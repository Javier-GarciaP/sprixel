use super::*;
use crate::history::History;

fn palette(name: &str, keys: &str) -> Palette {
    Palette {
        name: name.to_string(),
        colors: keys
            .chars()
            .enumerate()
            .map(|(i, key)| Color {
                key,
                name: format!("color {key}"),
                // A different color for each key.
                hex: format!("#{:02X}{:02X}00", i / 256, i % 256),
            })
            .collect(),
    }
}

/// A project with the palette `main` (the given keys) and a 4x4 sprite.
fn project_with(keys: &str) -> (Project, SpriteRef) {
    let mut project = Project::new("Kit");
    for op in [
        Operation::CreatePalette {
            palette: palette("main", keys),
            index: None,
        },
        Operation::CreateSprite {
            folder: vec![],
            name: "goblin".to_string(),
            width: 4,
            height: 4,
            palette: "main".to_string(),
        },
    ] {
        project.apply(&op).unwrap();
    }
    (project, SpriteRef::new(&[], "goblin"))
}

fn change(x: u32, y: u32, color: Option<&str>) -> ColorChange {
    ColorChange {
        x,
        y,
        color: color.map(str::to_string),
    }
}

fn pixels(project: &Project) -> String {
    let sprite = &project.root.sprites[0];
    String::from_utf8(sprite.layers[0].frames[0].pixels.clone()).unwrap()
}

fn colors(project: &Project) -> Vec<(char, String, String)> {
    project.palettes[0]
        .colors
        .iter()
        .map(|c| (c.key, c.name.clone(), c.hex.clone()))
        .collect()
}

// Checks the "First color of a new project" scenario: a person draws the
// first pixel with #1F2A44 on an empty palette. The palette gets one color
// with the key A and the name 1F2A44, and the pixel uses A.
#[test]
fn the_first_color_gets_the_key_a() {
    let (mut project, sprite) = project_with("");
    let ops = project
        .stroke_operations(&sprite, 0, 1, &[change(1, 0, Some("#1F2A44"))])
        .unwrap();
    History::new().apply(&mut project, "user", ops).unwrap();
    assert_eq!(
        colors(&project),
        vec![('A', "1F2A44".to_string(), "#1F2A44".to_string())]
    );
    assert_eq!(&pixels(&project)[..4], ".A..");
}

// Checks the "A color that is not in the palette" scenario: the stroke uses
// the new color, the palette gets it with the next free key, and one Undo
// removes both the pixels and the color; one Redo brings both back.
#[test]
fn one_undo_removes_the_stroke_and_the_new_color() {
    let (mut project, sprite) = project_with("RB");
    let before = project.clone();
    let mut history = History::new();
    let ops = project
        .stroke_operations(
            &sprite,
            0,
            1,
            &[change(0, 0, Some("#102030")), change(1, 0, Some("#102030"))],
        )
        .unwrap();
    history.apply(&mut project, "user", ops).unwrap();
    assert_eq!(project.palettes[0].colors.len(), 3);
    assert_eq!(project.palettes[0].colors[2].key, 'A');
    assert_eq!(&pixels(&project)[..4], "AA..");
    assert_eq!(history.entries().len(), 1);

    history.undo(&mut project).unwrap();
    assert_eq!(project.palettes, before.palettes);
    assert_eq!(pixels(&project), "................");

    history.redo(&mut project).unwrap();
    assert_eq!(project.palettes[0].colors[2].hex, "#102030");
    assert_eq!(&pixels(&project)[..4], "AA..");
}

// Checks that a color already in the palette is used as it is, whatever its
// letter case, and that the transparent pixel is the dot.
#[test]
fn a_color_in_the_palette_is_reused_ignoring_case() {
    let (mut project, sprite) = project_with("RB");
    // The key B (the second color) has the hex #000100 in the test palette.
    let ops = project
        .stroke_operations(
            &sprite,
            0,
            1,
            &[change(0, 0, Some("#000100")), change(1, 0, None)],
        )
        .unwrap();
    assert_eq!(ops.len(), 1, "no AddColor is needed");
    History::new().apply(&mut project, "user", ops).unwrap();
    assert_eq!(&pixels(&project)[..2], "B.");
    assert_eq!(project.palettes[0].colors.len(), 2);

    let lower = project
        .stroke_operations(&sprite, 0, 1, &[change(2, 0, Some("#000100"))])
        .unwrap();
    assert_eq!(lower.len(), 1);
}

// Checks that two new colors in one stroke get two different keys, in order,
// and that the hex is stored in capital letters.
#[test]
fn two_new_colors_get_the_next_two_keys() {
    let (mut project, sprite) = project_with("");
    let ops = project
        .stroke_operations(
            &sprite,
            0,
            1,
            &[change(0, 0, Some("#aabbcc")), change(1, 0, Some("#112233"))],
        )
        .unwrap();
    History::new().apply(&mut project, "user", ops).unwrap();
    assert_eq!(project.palettes[0].colors[0].hex, "#AABBCC");
    assert_eq!(project.palettes[0].colors[0].name, "AABBCC");
    assert_eq!(project.palettes[0].colors[1].key, 'B');
    assert_eq!(&pixels(&project)[..2], "AB");
}

// Checks the "Next free key" scenario: when A to Z are used, the new color
// gets `a`.
#[test]
fn the_key_after_z_is_a_lowercase_a() {
    let (mut project, sprite) = project_with("ABCDEFGHIJKLMNOPQRSTUVWXYZ");
    let ops = project
        .stroke_operations(&sprite, 0, 1, &[change(0, 0, Some("#FFFFFF"))])
        .unwrap();
    History::new().apply(&mut project, "user", ops).unwrap();
    assert_eq!(project.palettes[0].colors[26].key, 'a');
}

// Checks the full order of keys: letters, digits, then the other printable
// characters, never the dot or the space, 93 in all.
#[test]
fn the_keys_come_in_the_documented_order() {
    let mut palette = Palette {
        name: "main".to_string(),
        colors: vec![],
    };
    let mut order = String::new();
    while let Some(key) = palette.next_free_key() {
        order.push(key);
        palette.colors.push(Color {
            key,
            name: key.to_string(),
            hex: "#000000".to_string(),
        });
    }
    assert_eq!(order.chars().count(), MAX_COLORS);
    assert!(order.starts_with("ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!"));
    assert!(!order.contains('.') && !order.contains(' '));
    assert!(palette.validate().is_ok());
}

// Checks the "The palette is full" scenario in the core: with 93 colors a
// new color fails with `palette_full` and the project and the history stay
// as they are, while a color already in the palette still works.
#[test]
fn a_full_palette_refuses_a_new_color() {
    let all: String = {
        let mut p = Palette {
            name: "x".to_string(),
            colors: vec![],
        };
        while let Some(key) = p.next_free_key() {
            p.colors.push(Color {
                key,
                name: String::new(),
                hex: "#000000".to_string(),
            });
        }
        p.colors.iter().map(|c| c.key).collect()
    };
    let (mut project, sprite) = project_with(&all);
    let before = project.clone();
    let mut history = History::new();

    let error = project
        .stroke_operations(&sprite, 0, 1, &[change(0, 0, Some("#FEDCBA"))])
        .unwrap_err();
    assert_eq!(error.code, "palette_full");
    assert_eq!(project, before);
    assert!(history.entries().is_empty());

    // The hex of the first color of the test palette is #000000.
    let ops = project
        .stroke_operations(&sprite, 0, 1, &[change(0, 0, Some("#000000"))])
        .unwrap();
    history.apply(&mut project, "user", ops).unwrap();
    assert_eq!(project.palettes[0].colors.len(), 93);
}

// Checks that adding a color at the operation level also stops at 93 and
// leaves the palette unchanged.
#[test]
fn add_color_fails_with_palette_full() {
    let (mut project, _) = project_with("");
    for i in 0..93u32 {
        let key = project.palettes[0].next_free_key().unwrap();
        project
            .apply(&Operation::AddColor {
                palette: "main".to_string(),
                color: Color {
                    key,
                    name: format!("c{i}"),
                    hex: "#000000".to_string(),
                },
                index: None,
            })
            .unwrap();
    }
    let before = project.clone();
    let error = project
        .apply(&Operation::AddColor {
            palette: "main".to_string(),
            color: Color {
                key: '.',
                name: "x".to_string(),
                hex: "#000000".to_string(),
            },
            index: None,
        })
        .unwrap_err();
    assert_eq!(error.code, "palette_full");
    assert_eq!(project, before);
}

// Checks that a color with a key that is already used, or a key that is not
// allowed, is refused by AddColor and nothing changes.
#[test]
fn add_color_validates_the_color() {
    let (mut project, _) = project_with("R");
    let before = project.clone();
    let add = |key: char, hex: &str| Operation::AddColor {
        palette: "main".to_string(),
        color: Color {
            key,
            name: "x".to_string(),
            hex: hex.to_string(),
        },
        index: None,
    };
    assert_eq!(project.apply(&add('R', "#000000")).unwrap_err().code, "bad_palette");
    assert_eq!(project.apply(&add('.', "#000000")).unwrap_err().code, "bad_palette");
    assert_eq!(project.apply(&add('Z', "red")).unwrap_err().code, "bad_palette");
    assert_eq!(project, before);
}

// Checks that RemoveColor refuses a color that a sprite still uses, so an
// undo that comes in the wrong order can never leave a pixel without color.
#[test]
fn a_color_in_use_cannot_be_removed() {
    let (mut project, sprite) = project_with("");
    let ops = project
        .stroke_operations(&sprite, 0, 1, &[change(0, 0, Some("#123456"))])
        .unwrap();
    History::new().apply(&mut project, "user", ops).unwrap();
    let before = project.clone();
    let error = project
        .apply(&Operation::RemoveColor {
            palette: "main".to_string(),
            key: 'A',
        })
        .unwrap_err();
    assert_eq!(error.code, "color_in_use");
    assert_eq!(project, before);
}

// Checks that a color that is not a #RRGGBB value is refused before anything
// is built.
#[test]
fn a_bad_color_is_refused() {
    let (project, sprite) = project_with("");
    for bad in ["red", "#12345", "123456", "#GGGGGG"] {
        let error = project
            .stroke_operations(&sprite, 0, 1, &[change(0, 0, Some(bad))])
            .unwrap_err();
        assert_eq!(error.code, "bad_color", "{bad}");
    }
}

// Checks that a stroke on a sprite that does not exist is refused.
#[test]
fn a_missing_sprite_is_refused() {
    let (project, _) = project_with("");
    let error = project
        .stroke_operations(&SpriteRef::new(&[], "ghost"), 0, 1, &[])
        .unwrap_err();
    assert_eq!(error.code, "not_found");
}

// Checks that a color added in the middle of a palette by an inverse comes
// back at the same place (undo of a removal keeps the order).
#[test]
fn removing_and_restoring_a_color_keeps_its_place() {
    let (mut project, _) = project_with("RGB");
    let removed = project
        .apply(&Operation::RemoveColor {
            palette: "main".to_string(),
            key: 'G',
        })
        .unwrap();
    project.apply(&removed.inverse).unwrap();
    let keys: String = project.palettes[0].colors.iter().map(|c| c.key).collect();
    assert_eq!(keys, "RGB");
}
