use super::*;
use super::import::{ImportFormat, palette_from_file};

const SIXTEEN: &str = include_str!("testdata/sixteen.hex");
const SAMPLE: &str = include_str!("testdata/sample.gpl");

// Checks the "Import a hex file" scenario: a `.hex` file with 16 lines makes
// a palette with 16 colors, each with a unique key and a name.
#[test]
fn a_hex_file_with_16_lines_makes_16_colors() {
    let palette = palette_from_file("sixteen", ImportFormat::Hex, SIXTEEN).unwrap();
    assert_eq!(palette.colors.len(), 16);
    let keys: String = palette.colors.iter().map(|c| c.key).collect();
    assert_eq!(keys, "ABCDEFGHIJKLMNOP");
    assert_eq!(palette.colors[0].hex, "#1A1C2C");
    // The name of a color from a hex file is its value, so it is unique.
    assert_eq!(palette.colors[0].name, "1A1C2C");
    // The result passes the same rules as a palette made by hand.
    palette.validate().unwrap();
    check_colors(&palette.colors).unwrap();
}

// Checks that a hex file may have `#`, capital letters, blank lines,
// comments with ";" and Windows line ends, and that a repeated value is
// skipped.
#[test]
fn a_hex_file_can_be_a_little_untidy() {
    let text = "\u{feff}; my colors\r\n#FF0000\r\n\r\n00ff00\r\nff0000\r\n";
    let palette = palette_from_file("p", ImportFormat::Hex, text).unwrap();
    let hexes: Vec<&str> = palette.colors.iter().map(|c| c.hex.as_str()).collect();
    assert_eq!(hexes, vec!["#FF0000", "#00FF00"]);
}

// Checks that a line that is not a color is an error that names the line,
// and that an empty file is an error too.
#[test]
fn a_bad_hex_file_names_the_line() {
    let error = palette_from_file("p", ImportFormat::Hex, "ff0000\nnope\n").unwrap_err();
    assert_eq!(error.code, "bad_import");
    assert!(error.message.contains("line 2"), "{}", error.message);
    assert_eq!(palette_from_file("p", ImportFormat::Hex, "ff0000\n12345\n").unwrap_err().code, "bad_import");
    assert_eq!(palette_from_file("p", ImportFormat::Hex, "\n\n").unwrap_err().code, "bad_import");
}

// Checks a GIMP palette: the header fields and the comments are skipped, the
// names of the file are used, a color with no name is named by its value, and
// a repeated value is skipped.
#[test]
fn a_gpl_file_gives_colors_with_their_names() {
    let palette = palette_from_file("Sample", ImportFormat::Gpl, SAMPLE).unwrap();
    let got: Vec<(char, &str, &str)> = palette
        .colors
        .iter()
        .map(|c| (c.key, c.name.as_str(), c.hex.as_str()))
        .collect();
    assert_eq!(
        got,
        vec![
            ('A', "Night", "#1A1C2C"),
            ('B', "Red", "#F04545"),
            ('C', "Green", "#4ADE80"),
            ('D', "000000", "#000000"),
        ]
    );
}

// Checks that two colors with the same name in a `.gpl` file get different
// names, and that a file that is not a GIMP palette is refused.
#[test]
fn gpl_names_are_made_unique_and_the_header_is_required() {
    let text = "GIMP Palette\nName: x\n255 0 0 Sky\n0 0 255 sky\n1 2 3 Dark: blue\n";
    let palette = palette_from_file("p", ImportFormat::Gpl, text).unwrap();
    assert_eq!(palette.colors[0].name, "Sky");
    assert_eq!(palette.colors[1].name, "sky 2");
    // A ":" in the name of a color is part of the name.
    assert_eq!(palette.colors[2].name, "Dark: blue");
    assert_eq!(palette_from_file("p", ImportFormat::Gpl, "255 0 0\n").unwrap_err().code, "bad_import");
    let error = palette_from_file("p", ImportFormat::Gpl, "GIMP Palette\n300 0 0 x\n").unwrap_err();
    assert_eq!(error.code, "bad_import");
    assert!(error.message.contains("line 2"));
}

// Checks the limit: 94 different colors do not fit, and nothing is made.
#[test]
fn more_than_93_colors_do_not_fit() {
    let text: String = (0..94).map(|i| format!("{:06x}\n", i + 1)).collect();
    let error = palette_from_file("p", ImportFormat::Hex, &text).unwrap_err();
    assert_eq!(error.code, "palette_full");
    let ok: String = (0..93).map(|i| format!("{:06x}\n", i + 1)).collect();
    assert_eq!(palette_from_file("p", ImportFormat::Hex, &ok).unwrap().colors.len(), 93);
}

// Checks that the format comes from the end of the file name, in any letter
// case, and other files are not palettes.
#[test]
fn the_format_comes_from_the_file_name() {
    assert_eq!(ImportFormat::of_file("a.gpl"), Some(ImportFormat::Gpl));
    assert_eq!(ImportFormat::of_file("B.HEX"), Some(ImportFormat::Hex));
    assert_eq!(ImportFormat::of_file("c.png"), None);
    assert_eq!(ImportFormat::of_file("hex"), None);
}
