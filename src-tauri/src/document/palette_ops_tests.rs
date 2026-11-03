use super::*;
use crate::history::History;

fn color(key: char, name: &str, hex: &str) -> Color {
    Color {
        key,
        name: name.to_string(),
        hex: hex.to_string(),
    }
}

/// A project with the palette `main` (R red, B blue, G green), and two 2x2
/// sprites that use it: `goblin` ("RB" over "BG") and `orc` (all "R"). A
/// third sprite, `other`, uses another palette and is never touched.
fn project() -> Project {
    let mut project = Project::new("Kit");
    let colors = vec![
        color('R', "red", "#F04545"),
        color('B', "blue", "#2F6BFF"),
        color('G', "green", "#4ADE80"),
    ];
    let mut ops = vec![
        Operation::CreatePalette {
            palette: Palette {
                name: "main".to_string(),
                colors: colors.clone(),
            },
            index: None,
        },
        Operation::CreatePalette {
            palette: Palette {
                name: "second".to_string(),
                colors: vec![color('R', "red", "#000001")],
            },
            index: None,
        },
    ];
    for (name, palette) in [("goblin", "main"), ("orc", "main"), ("other", "second")] {
        ops.push(Operation::CreateSprite {
            folder: vec![],
            name: name.to_string(),
            width: 2,
            height: 2,
            palette: palette.to_string(),
        });
    }
    let all = |name: &str, keys: &str| Operation::SetPixels {
        sprite: SpriteRef::new(&[], name),
        layer: 0,
        frame: 1,
        pixels: keys
            .chars()
            .enumerate()
            .map(|(i, key)| PixelChange {
                x: (i % 2) as u32,
                y: (i / 2) as u32,
                key,
            })
            .collect(),
    };
    ops.push(all("goblin", "RBBG"));
    ops.push(all("orc", "RRRR"));
    ops.push(all("other", "RRRR"));
    for op in ops {
        project.apply(&op).unwrap();
    }
    project
}

fn keep(key: char) -> ColorEdit {
    let old = project();
    let color = old.palette("main").unwrap().colors.iter().find(|c| c.key == key).unwrap().clone();
    ColorEdit {
        from: Some(key),
        color,
    }
}

fn edit(from: char, to: Color) -> ColorEdit {
    ColorEdit { from: Some(from), color: to }
}

/// What each pixel of a sprite looks like: its hex, or `None` when transparent.
fn look(project: &Project, name: &str) -> Vec<Option<String>> {
    let sprite = project.find_sprite(&SpriteRef::new(&[], name)).unwrap();
    let palette = project.palette(&sprite.palette).unwrap();
    sprite.layers[0].frames[0]
        .pixels
        .iter()
        .map(|k| palette.colors.iter().find(|c| c.key == *k as char).map(|c| c.hex.clone()))
        .collect()
}

fn keys(project: &Project, name: &str) -> String {
    let sprite = project.find_sprite(&SpriteRef::new(&[], name)).unwrap();
    String::from_utf8(sprite.layers[0].frames[0].pixels.clone()).unwrap()
}

fn run(
    project: &mut Project,
    history: &mut History,
    colors: Vec<ColorEdit>,
    replacements: Vec<(char, char)>,
) -> Result<Vec<Target>, DocError> {
    let ops = project.palette_edit_operations("main", &colors, &replacements)?;
    history.apply(project, "user", ops)
}

// Checks the "Change a color" scenario: a new value for a color changes how
// every sprite that uses the key looks, and no pixel is rewritten.
#[test]
fn changing_a_value_changes_every_sprite_that_uses_it() {
    let mut p = project();
    let mut h = History::new();
    let red = edit('R', color('R', "red", "#D03030"));
    run(&mut p, &mut h, vec![red, keep('B'), keep('G')], vec![]).unwrap();
    assert_eq!(look(&p, "goblin")[0].as_deref(), Some("#D03030"));
    assert_eq!(look(&p, "orc")[3].as_deref(), Some("#D03030"));
    assert_eq!(keys(&p, "goblin"), "RBBG");
    // The sprite of the other palette is untouched.
    assert_eq!(look(&p, "other")[0].as_deref(), Some("#000001"));
}

// Checks the "Rename the key R to H" scenario: the stored pixels of every
// sprite change from R to H, they look the same, and it is one undo step.
#[test]
fn changing_a_key_keeps_every_sprite_looking_the_same() {
    let mut p = project();
    let (goblin, orc) = (look(&p, "goblin"), look(&p, "orc"));
    let mut h = History::new();
    let red = edit('R', color('H', "red", "#F04545"));
    let touched = run(&mut p, &mut h, vec![red, keep('B'), keep('G')], vec![]).unwrap();
    assert_eq!(keys(&p, "goblin"), "HBBG");
    assert_eq!(keys(&p, "orc"), "HHHH");
    assert_eq!(keys(&p, "other"), "RRRR");
    assert_eq!(look(&p, "goblin"), goblin);
    assert_eq!(look(&p, "orc"), orc);
    // Storage writes the palette and the sprites that changed, not `other`.
    assert!(touched.contains(&Target::Palette("main".to_string())));
    assert!(touched.contains(&Target::Sprite(SpriteRef::new(&[], "goblin"))));
    assert!(touched.contains(&Target::Sprite(SpriteRef::new(&[], "orc"))));
    assert!(!touched.contains(&Target::Sprite(SpriteRef::new(&[], "other"))));
    // One entry: one undo brings back the old key everywhere.
    assert_eq!(h.entries().len(), 1);
    h.undo(&mut p).unwrap();
    assert_eq!(keys(&p, "goblin"), "RBBG");
    assert_eq!(keys(&p, "orc"), "RRRR");
    assert_eq!(p.palette("main").unwrap().colors[0].key, 'R');
    h.redo(&mut p).unwrap();
    assert_eq!(keys(&p, "orc"), "HHHH");
}

// Checks that two keys can swap places in one edit (R becomes B and B
// becomes R) without the pixels getting mixed up.
#[test]
fn two_keys_can_swap_in_one_edit() {
    let mut p = project();
    let mut h = History::new();
    let a = edit('R', color('B', "red", "#F04545"));
    let b = edit('B', color('R', "blue", "#2F6BFF"));
    run(&mut p, &mut h, vec![a, b, keep('G')], vec![]).unwrap();
    assert_eq!(keys(&p, "goblin"), "BRRG");
    assert_eq!(look(&p, "goblin")[0].as_deref(), Some("#F04545"));
}

// Checks that adding a color is part of the same edit and that a palette
// can grow up to the limit through the editor.
#[test]
fn a_new_color_is_added_in_the_same_step() {
    let mut p = project();
    let mut h = History::new();
    let new = ColorEdit {
        from: None,
        color: color('Y', "yellow", "#F5C542"),
    };
    run(&mut p, &mut h, vec![keep('R'), keep('B'), keep('G'), new], vec![]).unwrap();
    assert_eq!(p.palette("main").unwrap().colors.len(), 4);
    h.undo(&mut p).unwrap();
    assert_eq!(p.palette("main").unwrap().colors.len(), 3);
}

// Checks the "Remove a used color" scenario: removing blue (used by goblin)
// needs a replacement, and the pixels that used it take the replacement.
#[test]
fn removing_a_used_color_applies_the_replacement() {
    let mut p = project();
    let mut h = History::new();
    run(&mut p, &mut h, vec![keep('R'), keep('G')], vec![('B', 'G')]).unwrap();
    assert_eq!(keys(&p, "goblin"), "RGGG");
    assert_eq!(p.palette("main").unwrap().colors.len(), 2);
    // Undo gives back the color and the exact pixels that used it.
    h.undo(&mut p).unwrap();
    assert_eq!(keys(&p, "goblin"), "RBBG");
    assert_eq!(p.palette("main").unwrap().colors.len(), 3);
}

// Checks that the replacement can also be transparent (".").
#[test]
fn a_used_color_can_be_replaced_by_transparent() {
    let mut p = project();
    let mut h = History::new();
    run(&mut p, &mut h, vec![keep('R'), keep('G')], vec![('B', '.')]).unwrap();
    assert_eq!(keys(&p, "goblin"), "R..G");
}

// Checks that removing a used color without a replacement is refused and
// nothing changes; a color that no sprite uses needs none.
#[test]
fn a_used_color_cannot_be_removed_without_a_replacement() {
    let mut p = project();
    let before = p.clone();
    let error = p
        .palette_edit_operations("main", &[keep('R'), keep('G')], &[])
        .unwrap_err();
    assert_eq!(error.code, "replacement_needed");
    assert_eq!(p, before);
    // Green is used by goblin; blue is not used by orc, but is by goblin. Add
    // a color nobody uses, then remove it freely.
    let mut h = History::new();
    let extra = ColorEdit { from: None, color: color('Y', "yellow", "#F5C542") };
    run(&mut p, &mut h, vec![keep('R'), keep('B'), keep('G'), extra], vec![]).unwrap();
    run(&mut p, &mut h, vec![keep('R'), keep('B'), keep('G')], vec![]).unwrap();
    assert_eq!(p.palette("main").unwrap().colors.len(), 3);
}

// Checks that a replacement must be a color that stays in the palette.
#[test]
fn a_replacement_must_stay_in_the_palette() {
    let p = project();
    let error = p
        .palette_edit_operations("main", &[keep('R')], &[('B', 'G'), ('G', 'R')])
        .unwrap_err();
    assert_eq!(error.code, "bad_replacement");
}

// Checks that a bad edit changes nothing: a duplicate key and an unknown
// original key are refused.
#[test]
fn a_bad_edit_is_refused() {
    let p = project();
    let twice = edit('R', color('B', "red", "#F04545"));
    let error = p
        .palette_edit_operations("main", &[twice, keep('B'), keep('G')], &[])
        .unwrap_err();
    assert_eq!(error.code, "duplicate_key");
    let ghost = edit('Z', color('Z', "ghost", "#000000"));
    let error = p.palette_edit_operations("main", &[ghost], &[]).unwrap_err();
    assert_eq!(error.code, "not_found");
    let error = p.palette_edit_operations("nope", &[], &[]).unwrap_err();
    assert_eq!(error.code, "not_found");
}

// Checks that an edit that changes nothing gives no operation, so it makes
// no history entry.
#[test]
fn an_edit_with_no_change_does_nothing() {
    let p = project();
    let ops = p
        .palette_edit_operations("main", &[keep('R'), keep('B'), keep('G')], &[])
        .unwrap();
    assert!(ops.is_empty());
}

// Checks that the rewrite keeps every frame of every layer, not only the
// first one.
#[test]
fn every_frame_and_layer_is_rewritten() {
    let mut p = project();
    let goblin = SpriteRef::new(&[], "goblin");
    p.apply(&Operation::AddFrame { sprite: goblin.clone(), duration_ms: 100 }).unwrap();
    p.apply(&Operation::SetPixels {
        sprite: goblin.clone(),
        layer: 0,
        frame: 2,
        pixels: vec![PixelChange { x: 0, y: 0, key: 'R' }],
    })
    .unwrap();
    let mut h = History::new();
    let red = edit('R', color('H', "red", "#F04545"));
    run(&mut p, &mut h, vec![red, keep('B'), keep('G')], vec![]).unwrap();
    let sprite = p.find_sprite(&goblin).unwrap();
    assert_eq!(sprite.layers[0].frames[1].pixels[0], b'H');
}
