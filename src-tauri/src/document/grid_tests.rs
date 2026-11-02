use super::*;
use super::grid::{Grid, write_grid};

const GOBLIN: &str = include_str!("testdata/goblin.grid");
const GOBLIN_WRITTEN: &str = include_str!("testdata/goblin_written.grid");

fn dungeon16() -> Palette {
    let color = |key, name: &str, hex: &str| Color {
        key,
        name: name.to_string(),
        hex: hex.to_string(),
    };
    Palette {
        name: "dungeon16".to_string(),
        colors: vec![
            color('N', "black", "#000000"),
            color('P', "skin", "#FFCC99"),
            color('A', "blue", "#2F6BFF"),
            color('R', "red", "#F04545"),
        ],
    }
}

// The goblin of the documentation, with the pixels of the first frame.
fn goblin() -> Sprite {
    let mut sprite = Sprite::new("goblin", 8, 8, "dungeon16").unwrap();
    let rows = [
        "....RRRR", "...RRRRR", "...NNNN.", "..NPNPPN", "..NPPPPN", "..NNNNNN", ".NAAAAN.",
        ".NA..AN.",
    ];
    sprite.layers[0].frames[0].pixels = rows.concat().into_bytes();
    sprite
}

// Checks "Parse the example": the goblin of the documentation gives the
// palette name, the legend and one frame of 8 rows of 8 pixels.
#[test]
fn the_documentation_example_parses() {
    let (grid, errors) = Grid::parse(GOBLIN);
    assert!(errors.is_empty(), "{errors:?}");
    assert_eq!(grid.palette.as_deref(), Some("dungeon16"));
    assert_eq!(grid.legend.len(), 5);
    assert_eq!(grid.legend[1].key, 'N');
    assert_eq!(grid.legend[1].name, "black");
    assert_eq!(grid.legend[1].hex.as_deref(), Some("#000000"));
    assert_eq!(grid.legend[0].hex, None);
    assert_eq!(grid.frames.len(), 1);
    assert_eq!(grid.frames[0].number, 1);
    assert_eq!(grid.frames[0].rows.len(), 8);
    let first: String = grid.frames[0].rows[0].cells.iter().collect();
    assert_eq!(first, "....RRRR");
    // The first row is on line 11 of the file.
    assert_eq!(grid.frames[0].rows[0].line, 11);
}

// Checks that blank lines and comments are ignored, keywords can be indented
// and the line ends of Windows are accepted.
#[test]
fn blank_lines_and_comments_are_ignored() {
    let text = "# my sprite\r\n\r\nframe 1\r\n# a comment\r\nRR\r\n\r\nRR\r\n";
    let (grid, errors) = Grid::parse(text);
    assert!(errors.is_empty(), "{errors:?}");
    assert_eq!(grid.palette, None);
    assert_eq!(grid.frames[0].rows.len(), 2);
    // The line numbers still count the ignored lines.
    assert_eq!(grid.frames[0].rows[1].line, 7);
}

// Checks that the key "#" is a color like any other: a row that starts with
// it is a row, and "# " (hash and space) is the comment.
#[test]
fn a_row_can_start_with_the_hash_key() {
    let (grid, errors) = Grid::parse("frame 1\n#.#\n# comment\n");
    assert!(errors.is_empty(), "{errors:?}");
    assert_eq!(grid.frames[0].rows.len(), 1);
    assert_eq!(grid.frames[0].rows[0].cells, vec!['#', '.', '#']);
}

// Checks that a row that begins with the word "frame" or "legend" is still a
// row when it is not a frame line.
#[test]
fn a_row_of_letters_is_not_a_keyword() {
    let (grid, errors) = Grid::parse("frame 1\nframe\nlegend\n");
    assert!(errors.is_empty(), "{errors:?}");
    assert_eq!(grid.frames[0].rows.len(), 2);
}

// Checks that several frames are read in order, with their numbers.
#[test]
fn several_frames_are_read() {
    let (grid, errors) = Grid::parse("frame 2\nRR\nframe 1\nBB\n");
    assert!(errors.is_empty(), "{errors:?}");
    let numbers: Vec<u32> = grid.frames.iter().map(|f| f.number).collect();
    assert_eq!(numbers, vec![2, 1]);
}

// Checks the writer against a golden file: the palette name, the legend of
// every color and the rows of the frame.
#[test]
fn the_writer_matches_the_golden_file() {
    assert_eq!(write_grid(&goblin(), &dungeon16(), Some(1)), GOBLIN_WRITTEN);
}

// Checks that what the writer gives is read back to the same rows, and that
// the example of the documentation and the written text have the same pixels.
#[test]
fn the_written_text_parses_back() {
    let (written, errors) = Grid::parse(&write_grid(&goblin(), &dungeon16(), Some(1)));
    assert!(errors.is_empty(), "{errors:?}");
    let (example, _) = Grid::parse(GOBLIN);
    assert_eq!(written.frames[0].rows.len(), example.frames[0].rows.len());
    for (a, b) in written.frames[0].rows.iter().zip(&example.frames[0].rows) {
        assert_eq!(a.cells, b.cells);
    }
}

// Checks that with no frame given the writer writes every frame, numbered
// from 1, and that a frame that does not exist writes nothing.
#[test]
fn the_writer_can_write_all_the_frames() {
    let mut sprite = Sprite::new("s", 2, 1, "dungeon16").unwrap();
    sprite.frame_durations.push(100);
    sprite.layers[0].frames.push(Frame { pixels: b"RN".to_vec() });
    let text = write_grid(&sprite, &dungeon16(), None);
    assert!(text.contains("frame 1\n..\n\nframe 2\nRN\n"), "{text}");
    assert!(!write_grid(&sprite, &dungeon16(), Some(3)).contains("frame"));
}

// ---- Validation (task 1.2) ----

fn check(text: &str, sprite: &Sprite, options: GridOptions) -> Vec<GridError> {
    match Grid::check(text, sprite, &dungeon16(), &options) {
        Ok(_) => Vec::new(),
        Err(errors) => errors,
    }
}

fn whole() -> GridOptions {
    GridOptions::default()
}

// A text for an 8x8 sprite with the rows given; row 7 can be changed by a test.
fn goblin_with(rows: &[&str]) -> String {
    format!("frame 1\n{}\n", rows.join("\n"))
}

const OK_ROWS: [&str; 8] = [
    "....RRRR", "...RRRRR", "...NNNN.", "..NPNPPN", "..NPPPPN", "..NNNNNN", ".NAAAAN.", ".NA..AN.",
];

// Checks that the documentation example has no error against its sprite.
#[test]
fn a_valid_grid_has_no_errors() {
    assert_eq!(check(GOBLIN, &goblin(), whole()), vec![]);
}

// Checks "Short row": the error `row_length` with the exact message of the
// documentation, the line (15) and the row (7).
#[test]
fn a_short_row_is_row_length() {
    let mut rows = OK_ROWS;
    rows[6] = ".NAAAAN";
    // The text starts with the 8 lines of the header, so row 7 is on line 15.
    let text = format!("palette: dungeon16\n\nlegend\n  . transparent\n\n# note\n\n{}", goblin_with(&rows));
    let errors = check(&text, &goblin(), whole());
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert_eq!(errors[0].code, "row_length");
    assert_eq!(errors[0].message, "line 15 (row 7 of frame 1): 7 columns, expected 8");
    assert_eq!((errors[0].line, errors[0].row), (Some(15), Some(7)));
}

// Checks `row_count`: a frame with 7 rows on an 8 row sprite.
#[test]
fn a_missing_row_is_row_count() {
    let errors = check(&goblin_with(&OK_ROWS[..7]), &goblin(), whole());
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert_eq!(errors[0].code, "row_count");
    assert_eq!(errors[0].message, "frame 1 has 7 rows, expected 8");
}

// Checks `unknown_key` and its position: line, row and column, with the
// message of the documentation. The keys are case sensitive.
#[test]
fn a_key_that_is_not_in_the_palette_is_unknown_key() {
    let mut rows = OK_ROWS;
    rows[3] = "..XPNPPN";
    let errors = check(&goblin_with(&rows), &goblin(), whole());
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert_eq!(errors[0].code, "unknown_key");
    assert_eq!(errors[0].message, "line 5 (row 4, column 3): key \"X\" is not in palette dungeon16");
    assert_eq!((errors[0].line, errors[0].row, errors[0].column), (Some(5), Some(4), Some(3)));
    rows[3] = "..nPNPPN";
    assert_eq!(check(&goblin_with(&rows), &goblin(), whole())[0].code, "unknown_key");
}

// Checks `no_frame`: a text with no frame block, and an empty text.
#[test]
fn a_text_without_a_frame_is_no_frame() {
    let errors = check("palette: dungeon16\n", &goblin(), whole());
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].code, "no_frame");
    assert_eq!(errors[0].message, "no \"frame\" block found");
    assert_eq!(check("", &goblin(), whole())[0].code, "no_frame");
}

// Checks `bad_frame`: frame 5 on a sprite with 4 frames says how many it
// has. Frame 0 and a frame written twice are bad frames too.
#[test]
fn a_frame_that_does_not_exist_is_bad_frame() {
    let mut sprite = goblin();
    for _ in 0..3 {
        sprite.frame_durations.push(100);
        sprite.layers[0].frames.push(Frame::blank(8, 8));
    }
    let errors = check(&format!("frame 5\n{}\n", OK_ROWS.join("\n")), &sprite, whole());
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert_eq!(errors[0].code, "bad_frame");
    assert_eq!(errors[0].message, "frame 5 does not exist; the sprite has 4 frames");
    assert_eq!(check("frame 0\n", &sprite, whole())[0].code, "bad_frame");
    assert_eq!(check("frame 1\nframe 1\n", &sprite, whole())[0].code, "bad_frame");
}

// Checks `too_big`: a grid wider than 512 is refused with its size.
#[test]
fn a_grid_over_512_is_too_big() {
    let sprite = Sprite::new("s", 512, 512, "dungeon16").unwrap();
    let row = ".".repeat(600);
    let errors = check(&format!("frame 1\n{row}\n"), &sprite, GridOptions { region: Some((0, 0)), skip_transparent: false });
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert_eq!(errors[0].code, "too_big");
    assert_eq!(errors[0].message, "grid is 600x1; maximum is 512x512");
}

// Checks `region_out`: an 8x8 grid at (10, 2) on a 16x8 sprite, with the
// message of the documentation; a grid that fits exactly is accepted.
#[test]
fn a_region_that_does_not_fit_is_region_out() {
    let sprite = Sprite::new("s", 16, 8, "dungeon16").unwrap();
    let at = |x, y| GridOptions { region: Some((x, y)), skip_transparent: false };
    let errors = check(&goblin_with(&OK_ROWS), &sprite, at(10, 2));
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert_eq!(errors[0].code, "region_out");
    assert_eq!(errors[0].message, "region at (10, 2) of size 8x8 does not fit in 16x8");
    assert_eq!(check(&goblin_with(&OK_ROWS), &sprite, at(8, 0)), vec![]);
}

// Checks that a grid with no region must be the size of the sprite, and that
// with a region the grid may be smaller.
#[test]
fn only_a_region_can_be_smaller_than_the_sprite() {
    let sprite = Sprite::new("s", 16, 16, "dungeon16").unwrap();
    assert_eq!(check(&goblin_with(&OK_ROWS), &sprite, whole())[0].code, "row_count");
    let region = GridOptions { region: Some((0, 0)), skip_transparent: false };
    assert_eq!(check(&goblin_with(&OK_ROWS), &sprite, region), vec![]);
}

// Checks `palette_full`: the legend adds a color to a palette of 93 colors.
#[test]
fn a_legend_color_in_a_full_palette_is_palette_full() {
    let mut palette = dungeon16();
    palette.colors = ('!'..='~')
        .filter(|c| *c != '.')
        .map(|key| Color { key, name: format!("c{}", key as u32), hex: "#112233".to_string() })
        .collect();
    assert_eq!(palette.colors.len(), 93);
    // All 93 keys are used, so a free key cannot exist in a real palette.
    // Here the key "Q" is swapped for a space to leave it free: the check of
    // the limit still has to say `palette_full`.
    palette.colors.retain(|c| c.key != 'Q');
    palette.colors.push(Color { key: ' ', name: "x".to_string(), hex: "#000000".to_string() });
    assert_eq!(palette.colors.len(), 93);
    let text = "legend\n  Q teal #1FB5A8\nframe 1\n..\n";
    let sprite = Sprite::new("s", 2, 1, "dungeon16").unwrap();
    let errors = Grid::check(text, &sprite, &palette, &whole()).unwrap_err();
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert_eq!(errors[0].code, "palette_full");
    assert_eq!(errors[0].line, Some(2));
}

// Checks `bad_legend`: a bad key, a bad color, a missing name, a color for
// ".", a key twice and a name that another color already has.
#[test]
fn a_wrong_legend_line_is_bad_legend() {
    let sprite = Sprite::new("s", 2, 1, "dungeon16").unwrap();
    for line in [
        "QQ teal #1FB5A8",
        "Q teal #1FB5",
        "Q #1FB5A8",
        "Q",
        ". clear #FFFFFF",
        "Q red #1FB5A8",
    ] {
        let errors = check(&format!("legend\n  {line}\nframe 1\n..\n"), &sprite, whole());
        assert_eq!(errors.len(), 1, "{line}: {errors:?}");
        assert_eq!(errors[0].code, "bad_legend", "{line}");
        assert_eq!(errors[0].line, Some(2), "{line}");
    }
    let twice = check("legend\n  Q teal #1FB5A8\n  Q blue2 #1FB5A9\nframe 1\n..\n", &sprite, whole());
    assert_eq!(twice[0].code, "bad_legend");
    assert_eq!(twice[0].line, Some(3));
}

// Checks "Several errors": two bad rows give two errors together, so an AI
// can fix them in one try.
#[test]
fn all_the_errors_are_reported_together() {
    let mut rows = OK_ROWS;
    rows[1] = "...RRRR";
    rows[4] = "..NPXPPN";
    let errors = check(&goblin_with(&rows), &goblin(), whole());
    let codes: Vec<&str> = errors.iter().map(|e| e.code).collect();
    assert_eq!(codes, vec!["row_length", "unknown_key"]);
}
