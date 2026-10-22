use super::*;
use crate::document::{Color, Operation, Palette, PixelChange, SpriteRef, Tag, TagMode};
use std::fs;
use std::path::PathBuf;

// The palette of the documentation example (same as the golden palette file).
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

fn read_normalized(path: PathBuf) -> String {
    // Git may turn line endings into CRLF on Windows; the format always uses LF.
    fs::read_to_string(path).unwrap().replace("\r\n", "\n")
}

fn golden(relative: &str) -> String {
    read_normalized(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src/storage/testdata/dungeon-kit")
            .join(relative),
    )
}

/// The operations that draw one frame of the goblin from rows of text.
fn draw_rows(frame: u32, rows: [&str; 4]) -> Operation {
    let mut pixels = Vec::new();
    for (y, row) in rows.iter().enumerate() {
        for (x, key) in row.chars().enumerate() {
            if key != '.' {
                pixels.push(PixelChange {
                    x: x as u32,
                    y: y as u32,
                    key,
                });
            }
        }
    }
    Operation::SetPixels {
        sprite: SpriteRef::new(&["Personajes"], "goblin"),
        layer: 0,
        frame,
        pixels,
    }
}

// Checks that the JSON examples in docs/architecture.md are exactly the files that
// the code writes for the same project (they also match the golden files), so the
// documentation cannot drift from the real format.
#[test]
fn the_examples_in_the_architecture_doc_are_real_files() {
    let parent = tempfile::tempdir().unwrap();
    let mut session = Session::create(parent.path(), "Dungeon Kit").unwrap();
    let goblin = SpriteRef::new(&["Personajes"], "goblin");
    let ops = vec![
        Operation::CreatePalette {
            palette: dungeon16(),
            index: None,
        },
        Operation::CreateFolder {
            parent: vec![],
            name: "Personajes".to_string(),
        },
        Operation::CreateSprite {
            folder: vec!["Personajes".to_string()],
            name: "goblin".to_string(),
            width: 4,
            height: 4,
            palette: "dungeon16".to_string(),
        },
        Operation::AddFrame {
            sprite: goblin.clone(),
            duration_ms: 120,
        },
        draw_rows(1, ["..RR", ".RRR", "NPNP", "NPPN"]),
        draw_rows(2, ["RR..", "RRR.", "PNPN", "NPPN"]),
        Operation::AddTag {
            sprite: goblin,
            tag: Tag {
                name: "walk".to_string(),
                from: 1,
                to: 2,
                fps: 12,
                mode: TagMode::Loop,
            },
            index: None,
        },
    ];
    session.apply("user", ops).unwrap();
    assert_eq!(session.flush(), SaveStatus::Saved);

    let doc = read_normalized(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../docs/architecture.md"));
    for relative in [
        "sprixel.json",
        "palettes/dungeon16.palette.json",
        "sprites/Personajes/goblin.sprite.json",
    ] {
        let written = read_normalized(session.root().join(relative));
        assert_eq!(written, golden(relative), "{relative} differs from its golden file");
        assert!(
            doc.contains(written.trim_end()),
            "the example for {relative} in docs/architecture.md is not the real file"
        );
    }
}
