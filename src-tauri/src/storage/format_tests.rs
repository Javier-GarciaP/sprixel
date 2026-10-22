use super::*;
use crate::document::{Color, Frame, Layer, Palette, Sprite, SpriteRef, Tag, TagMode};
use std::fs;
use std::path::PathBuf;

fn testdata(path: &str) -> String {
    let full = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src/storage/testdata/dungeon-kit")
        .join(path);
    // Git may turn line endings into CRLF on Windows; the format always uses LF.
    fs::read_to_string(full).unwrap().replace("\r\n", "\n")
}

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

fn frame(rows: [&str; 4]) -> Frame {
    Frame {
        pixels: rows.concat().into_bytes(),
    }
}

fn goblin() -> Sprite {
    Sprite {
        name: "goblin".to_string(),
        width: 4,
        height: 4,
        palette: "dungeon16".to_string(),
        frame_durations: vec![83, 120],
        layers: vec![Layer {
            name: "Layer 1".to_string(),
            frames: vec![
                frame(["..RR", ".RRR", "NPNP", "NPPN"]),
                frame(["RR..", "RRR.", "PNPN", "NPPN"]),
            ],
        }],
        tags: vec![Tag {
            name: "walk".to_string(),
            from: 1,
            to: 2,
            fps: 12,
            mode: TagMode::Loop,
        }],
    }
}

fn manifest() -> Manifest {
    Manifest {
        name: "Dungeon Kit".to_string(),
        active_palette: Some("dungeon16".to_string()),
    }
}

// Checks that the manifest is written exactly like the golden file.
#[test]
fn manifest_matches_the_golden_file() {
    assert_eq!(manifest_to_text(&manifest()), testdata("sprixel.json"));
}

// Checks that the palette file is written exactly like the golden file.
#[test]
fn palette_matches_the_golden_file() {
    assert_eq!(
        palette_to_text(&dungeon16()),
        testdata("palettes/dungeon16.palette.json")
    );
}

// Checks that the sprite file is written exactly like the golden file:
// one row string per line, so Git shows clean diffs.
#[test]
fn sprite_matches_the_golden_file() {
    assert_eq!(
        sprite_to_text(&goblin()),
        testdata("sprites/Personajes/goblin.sprite.json")
    );
}

// Checks that the golden files are read back into the same model.
#[test]
fn golden_files_are_read_back() {
    assert_eq!(manifest_from_text(&testdata("sprixel.json")).unwrap(), manifest());
    assert_eq!(
        palette_from_text("dungeon16", &testdata("palettes/dungeon16.palette.json")).unwrap(),
        dungeon16()
    );
    let text = testdata("sprites/Personajes/goblin.sprite.json");
    assert_eq!(sprite_from_text("goblin", &text, &dungeon16()).unwrap(), goblin());
}

// Checks that changing one pixel changes exactly one line of the saved file.
#[test]
fn changing_one_pixel_changes_one_line() {
    let before = sprite_to_text(&goblin());
    let mut changed = goblin();
    // Row 3 (index 2), column 2 (index 1) of frame 1: "NPNP" becomes "NANP".
    changed.layers[0].frames[0].pixels[2 * 4 + 1] = b'A';
    let after = sprite_to_text(&changed);

    let before_lines: Vec<&str> = before.lines().collect();
    let after_lines: Vec<&str> = after.lines().collect();
    assert_eq!(before_lines.len(), after_lines.len());
    let different: Vec<_> = before_lines
        .iter()
        .zip(&after_lines)
        .filter(|(a, b)| a != b)
        .collect();
    assert_eq!(different.len(), 1);
    assert!(different[0].0.contains("NPNP") && different[0].1.contains("NANP"));
}

// Checks that writing is deterministic: the same sprite always gives the same text.
#[test]
fn writing_twice_gives_the_same_text() {
    assert_eq!(sprite_to_text(&goblin()), sprite_to_text(&goblin()));
}

// Checks that a manifest without an active palette leaves that field out.
#[test]
fn manifest_without_active_palette() {
    let manifest = Manifest {
        name: "Empty".to_string(),
        active_palette: None,
    };
    let text = manifest_to_text(&manifest);
    assert!(!text.contains("activePalette"));
    assert_eq!(manifest_from_text(&text).unwrap(), manifest);
}

// Checks that a sprite file with rows of the wrong length is refused with a reason.
#[test]
fn a_row_with_the_wrong_length_is_refused() {
    let text = testdata("sprites/Personajes/goblin.sprite.json").replace("\"..RR\"", "\"..R\"");
    let error = sprite_from_text("goblin", &text, &dungeon16()).unwrap_err();
    assert!(error.contains("3") && error.contains("4"), "{error}");
}

// Checks that a sprite with the wrong number of rows is refused.
#[test]
fn a_frame_with_the_wrong_row_count_is_refused() {
    let text = testdata("sprites/Personajes/goblin.sprite.json").replace("          \"NPPN\"\n        ],\n        [", "        ],\n        [");
    assert!(sprite_from_text("goblin", &text, &dungeon16()).is_err());
}

// Checks that a key that is not in the palette is refused when reading.
#[test]
fn an_unknown_key_in_a_file_is_refused() {
    let text = testdata("sprites/Personajes/goblin.sprite.json").replace("\"..RR\"", "\"..RZ\"");
    let error = sprite_from_text("goblin", &text, &dungeon16()).unwrap_err();
    assert!(error.contains("unknown_key"), "{error}");
}

// Checks that text that is not JSON gives an error instead of a panic.
#[test]
fn invalid_json_is_refused() {
    assert!(sprite_from_text("goblin", "{ not json", &dungeon16()).is_err());
    assert!(palette_from_text("dungeon16", "").is_err());
    assert!(manifest_from_text("[]").is_err());
}

// Checks that a palette file with a bad key (here ".") is refused.
#[test]
fn a_palette_file_with_a_bad_key_is_refused() {
    let text = testdata("palettes/dungeon16.palette.json").replace("\"N\"", "\".\"");
    assert!(palette_from_text("dungeon16", &text).is_err());
}

// Checks that the files are written to the folder layout of the spec and read back.
#[test]
fn files_are_written_and_read_back_in_the_project_folder() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let sprite_ref = SpriteRef::new(&["Personajes"], "goblin");

    write_manifest(&DiskWriter, root, &manifest()).unwrap();
    write_palette(&DiskWriter, root, &dungeon16()).unwrap();
    write_sprite(&DiskWriter, root, &sprite_ref, &goblin()).unwrap();

    assert!(root.join("sprixel.json").is_file());
    assert!(root.join("palettes/dungeon16.palette.json").is_file());
    assert!(root.join("sprites/Personajes/goblin.sprite.json").is_file());
    assert_eq!(read_manifest(root).unwrap(), manifest());
    assert_eq!(read_palette(&palette_path(root, "dungeon16")).unwrap(), dungeon16());
    assert_eq!(read_sprite(root, &sprite_ref, &dungeon16()).unwrap(), goblin());
}

// Checks that reading a file that does not exist gives an error that names the path.
#[test]
fn reading_a_missing_file_names_the_path() {
    let dir = tempfile::tempdir().unwrap();
    let error = read_manifest(dir.path()).unwrap_err().to_string();
    assert!(error.contains("sprixel.json"), "{error}");
}
