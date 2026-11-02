//! Reads the colors of a palette file: `.gpl` (GIMP) and `.hex` (one hex
//! value per line). Every color gets the next free key and a name, which the
//! user can edit afterwards in the palette editor.

use super::error::DocError;
use super::model::{Color, MAX_COLORS, Palette};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportFormat {
    Gpl,
    Hex,
}

impl ImportFormat {
    /// The format of a file, from the end of its name (`.gpl` or `.hex`, in
    /// any letter case).
    pub fn of_file(name: &str) -> Option<ImportFormat> {
        match name.rsplit_once('.')?.1.to_ascii_lowercase().as_str() {
            "gpl" => Some(ImportFormat::Gpl),
            "hex" => Some(ImportFormat::Hex),
            _ => None,
        }
    }
}

fn bad(line: usize, reason: &str) -> DocError {
    DocError::new("bad_import", format!("line {line}: {reason}"))
}

/// The colors of the text: `(#RRGGBB, name in the file)`. A color that
/// repeats an earlier value is skipped.
fn read(format: ImportFormat, text: &str) -> Result<Vec<(String, Option<String>)>, DocError> {
    let mut found: Vec<(String, Option<String>)> = Vec::new();
    let mut add = |hex: String, name: Option<String>| {
        if !found.iter().any(|(h, _)| *h == hex) {
            found.push((hex, name));
        }
    };
    // A file saved on Windows can start with a byte order mark.
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    match format {
        ImportFormat::Hex => {
            for (i, line) in text.lines().enumerate() {
                let line = line.trim();
                if line.is_empty() || line.starts_with(';') {
                    continue;
                }
                let digits = line.strip_prefix('#').unwrap_or(line);
                if digits.len() != 6 || !digits.bytes().all(|b| b.is_ascii_hexdigit()) {
                    return Err(bad(i + 1, "expected one color like 1a1c2c"));
                }
                add(format!("#{}", digits.to_ascii_uppercase()), None);
            }
        }
        ImportFormat::Gpl => {
            let mut lines = text.lines().enumerate();
            match lines.next() {
                Some((_, first)) if first.trim() == "GIMP Palette" => {}
                _ => return Err(bad(1, "a GIMP palette starts with \"GIMP Palette\"")),
            }
            for (i, line) in lines {
                let line = line.trim();
                // The comments, and the header fields (`Name:`, `Columns:`): a
                // color line starts with a number, so a ":" in its name is fine.
                let starts_with_number = line
                    .split_whitespace()
                    .next()
                    .is_some_and(|p| p.chars().all(|c| c.is_ascii_digit()));
                if line.is_empty() || line.starts_with('#') || (!starts_with_number && line.contains(':')) {
                    continue;
                }
                let mut parts = line.split_whitespace();
                let mut channel = || -> Result<u8, DocError> {
                    parts
                        .next()
                        .and_then(|p| p.parse::<u8>().ok())
                        .ok_or_else(|| bad(i + 1, "expected three numbers from 0 to 255"))
                };
                let (r, g, b) = (channel()?, channel()?, channel()?);
                let name = parts.collect::<Vec<_>>().join(" ");
                add(
                    format!("#{r:02X}{g:02X}{b:02X}"),
                    (!name.is_empty()).then_some(name),
                );
            }
        }
    }
    if found.is_empty() {
        return Err(DocError::new("bad_import", "the file has no colors"));
    }
    Ok(found)
}

/// A new palette called `name` with the colors of the file. The keys are the
/// next free ones (`A` to `Z`, `a` to `z`, `0` to `9`, then the symbols). A
/// color is called as in the file, or by its value; a repeated name gets a
/// number. More than 93 colors fail with `palette_full`.
pub fn palette_from_file(name: &str, format: ImportFormat, text: &str) -> Result<Palette, DocError> {
    let read = read(format, text)?;
    if read.len() > MAX_COLORS {
        return Err(DocError::new(
            "palette_full",
            format!("the file has {} colors, a palette has at most {MAX_COLORS}", read.len()),
        ));
    }
    let mut palette = Palette {
        name: name.to_string(),
        colors: Vec::new(),
    };
    for (hex, file_name) in read {
        let key = palette.next_free_key().expect("at most 93 colors");
        let base = file_name.unwrap_or_else(|| hex[1..].to_string());
        let mut unique = base.clone();
        for n in 2.. {
            if !palette.colors.iter().any(|c| c.name.to_lowercase() == unique.to_lowercase()) {
                break;
            }
            unique = format!("{base} {n}");
        }
        palette.colors.push(Color { key, name: unique, hex });
    }
    Ok(palette)
}
