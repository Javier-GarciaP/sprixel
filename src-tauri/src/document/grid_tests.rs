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
