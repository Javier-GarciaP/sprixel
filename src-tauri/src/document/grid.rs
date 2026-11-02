//! The text grid: a sprite written as plain text, one character per pixel.
//! This module reads it (`Grid::parse`) and writes it (`write_grid`). The
//! format is in `docs/text-grid-format.md`. Plain Rust, no Tauri types.

use super::model::{Palette, Sprite, TRANSPARENT};
use serde::Serialize;

/// A problem in a grid. `line` is the line of the text, `row` the row inside
/// its frame and `column` the character inside the row, all counted from 1
/// and given only when they apply.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GridError {
    pub code: &'static str,
    pub message: String,
    pub line: Option<usize>,
    pub row: Option<usize>,
    pub column: Option<usize>,
}

impl GridError {
    pub(super) fn new(code: &'static str, message: String) -> GridError {
        GridError { code, message, line: None, row: None, column: None }
    }

    pub(super) fn at(mut self, line: usize) -> GridError {
        self.line = Some(line);
        self
    }
}

/// One line of the `legend` block: `<key> <name> [#hex]`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LegendLine {
    pub line: usize,
    pub key: char,
    pub name: String,
    /// `#RRGGBB` in capital letters, when the line has a color.
    pub hex: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridRow {
    pub line: usize,
    pub cells: Vec<char>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridFrame {
    /// The line of the `frame` keyword.
    pub line: usize,
    pub number: u32,
    pub rows: Vec<GridRow>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Grid {
    pub palette: Option<String>,
    pub legend: Vec<LegendLine>,
    pub frames: Vec<GridFrame>,
}

enum Part {
    Header,
    Legend,
    Frame,
}

impl Grid {
    /// Reads a text grid. It never stops at the first problem: it gives what
    /// it could read and all the errors of the text itself (`no_frame`,
    /// `bad_frame` and `bad_legend`). Checking the pixels against a sprite is
    /// a separate step.
    pub fn parse(text: &str) -> (Grid, Vec<GridError>) {
        let mut grid = Grid::default();
        let mut errors = Vec::new();
        let mut part = Part::Header;
        for (index, raw) in text.trim_start_matches('\u{feff}').lines().enumerate() {
            let number = index + 1;
            let line = raw.trim();
            if line.is_empty() || is_comment(line, &part) {
                continue;
            }
            if let Some(frame) = frame_header(line) {
                part = Part::Frame;
                match frame {
                    Ok(n) if grid.frames.iter().any(|f| f.number == n) => errors.push(
                        GridError::new("bad_frame", format!("line {number}: frame {n} appears twice"))
                            .at(number),
                    ),
                    Ok(n) => grid.frames.push(GridFrame { line: number, number: n, rows: Vec::new() }),
                    Err(message) => errors.push(GridError::new("bad_frame", format!("line {number}: {message}")).at(number)),
                }
                continue;
            }
            match part {
                Part::Frame => {
                    // Rows of an unreadable header go nowhere.
                    if let Some(frame) = grid.frames.last_mut().filter(|f| errors.iter().all(|e| e.line != Some(f.line))) {
                        frame.rows.push(GridRow { line: number, cells: line.chars().collect() });
                    }
                }
                Part::Legend => match legend_line(number, line) {
                    Ok(entry) if grid.legend.iter().any(|l| l.key == entry.key) => errors.push(
                        GridError::new("bad_legend", format!("line {number}: key {:?} is in the legend twice", entry.key))
                            .at(number),
                    ),
                    Ok(entry) => grid.legend.push(entry),
                    Err(error) => errors.push(error),
                },
                Part::Header => {
                    if let Some(name) = line.strip_prefix("palette:") {
                        let name = name.trim();
                        if name.is_empty() {
                            errors.push(GridError::new("no_frame", format!("line {number}: \"palette:\" needs a name")).at(number));
                        } else {
                            grid.palette = Some(name.to_string());
                        }
                    } else if line == "legend" {
                        part = Part::Legend;
                    } else {
                        errors.push(
                            GridError::new(
                                "no_frame",
                                format!("line {number}: expected \"palette:\", \"legend\" or \"frame\", found {line:?}"),
                            )
                            .at(number),
                        );
                    }
                }
            }
        }
        if grid.frames.is_empty() {
            errors.push(GridError::new("no_frame", "no \"frame\" block found".to_string()));
        }
        (grid, errors)
    }
}

// A comment is a line that starts with `#`. Inside a frame, "#" is also a
// color key, so there only "#" alone or "# " followed by text is a comment.
// In the legend, a line that has the form of a legend line for the key "#"
// (`# name #RRGGBB`) is a legend line.
fn is_comment(line: &str, part: &Part) -> bool {
    if !line.starts_with('#') {
        return false;
    }
    let spaced = line.len() == 1 || line.as_bytes()[1] == b' ' || line.as_bytes()[1] == b'\t';
    match part {
        Part::Frame => spaced,
        Part::Legend => !(spaced && matches!(legend_line(0, line), Ok(LegendLine { hex: Some(_), .. }))),
        Part::Header => true,
    }
}

// `frame <n>`: `None` when the line is not a frame line (a row can be the
// word "frame"), an error message when the number is not good.
fn frame_header(line: &str) -> Option<Result<u32, String>> {
    let rest = line.strip_prefix("frame")?;
    if !rest.starts_with(|c: char| c.is_whitespace()) {
        return None;
    }
    let mut words = rest.split_whitespace();
    Some(match (words.next().map(str::parse::<u32>), words.next()) {
        (Some(Ok(0)), None) => Err("frames are numbered from 1".to_string()),
        (Some(Ok(n)), None) => Ok(n),
        _ => Err(format!("expected \"frame <number>\", found {line:?}")),
    })
}

fn legend_line(number: usize, line: &str) -> Result<LegendLine, GridError> {
    let bad = |message: String| GridError::new("bad_legend", format!("line {number}: {message}")).at(number);
    let mut words: Vec<&str> = line.split_whitespace().collect();
    let key_word = words.remove(0);
    let mut chars = key_word.chars();
    let key = match (chars.next(), chars.next()) {
        (Some(c), None) if c.is_ascii_graphic() => c,
        _ => return Err(bad(format!("{key_word:?} is not a key, a key is one printable character"))),
    };
    let mut hex = None;
    if words.len() > 1 && words.last().is_some_and(|w| w.starts_with('#')) {
        let word = words.pop().unwrap();
        let valid = word.len() == 7 && word[1..].bytes().all(|b| b.is_ascii_hexdigit());
        if !valid {
            return Err(bad(format!("{word:?} is not a color, expected #RRGGBB")));
        }
        hex = Some(word.to_ascii_uppercase());
    }
    if words.is_empty() {
        return Err(bad(format!("the legend line for {key:?} needs a name")));
    }
    if key == TRANSPARENT && hex.is_some() {
        return Err(bad("\".\" is always transparent, it cannot have a color".to_string()));
    }
    Ok(LegendLine { line: number, key, name: words.join(" "), hex })
}

/// Writes a sprite as a text grid: the palette name, a legend with every color
/// of the palette and the rows of the frames. `frame` is one frame number
/// (from 1), or `None` for all the frames. Only the first layer is written
/// (version 1 shows one layer).
pub fn write_grid(sprite: &Sprite, palette: &Palette, frame: Option<u32>) -> String {
    let mut out = format!("palette: {}\n\nlegend\n  {TRANSPARENT} transparent\n", palette.name);
    let width = palette.colors.iter().map(|c| c.name.chars().count()).max().unwrap_or(0) + 2;
    for color in &palette.colors {
        let pad = width - color.name.chars().count();
        out.push_str(&format!("  {} {}{}{}\n", color.key, color.name, " ".repeat(pad), color.hex));
    }
    let frames = &sprite.layers[0].frames;
    for (index, pixels) in frames.iter().enumerate() {
        let number = index as u32 + 1;
        if frame.is_some_and(|wanted| wanted != number) {
            continue;
        }
        out.push_str(&format!("\nframe {number}\n"));
        for row in pixels.pixels.chunks(sprite.width as usize) {
            out.push_str(&String::from_utf8_lossy(row));
            out.push('\n');
        }
    }
    out
}
