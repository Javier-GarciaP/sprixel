use super::*;
use crate::document::{Color, Palette, PixelChange, SpriteRef};

fn project_with_sprite() -> (Project, SpriteRef) {
    let mut project = Project::new("Dungeon Kit");
    let palette = Palette {
        name: "dungeon16".to_string(),
        colors: vec![Color {
            key: 'N',
            name: "black".to_string(),
            hex: "#000000".to_string(),
        }],
    };
    project
        .apply(&Operation::CreatePalette {
            palette,
            index: None,
        })
        .unwrap();
    project
        .apply(&Operation::CreateSprite {
            folder: vec![],
            name: "goblin".to_string(),
            width: 4,
            height: 4,
            palette: "dungeon16".to_string(),
        })
        .unwrap();
    (project, SpriteRef::new(&[], "goblin"))
}

fn draw(sprite: &SpriteRef, x: u32, key: char) -> Operation {
    Operation::SetPixels {
        sprite: sprite.clone(),
        layer: 0,
        frame: 1,
        pixels: vec![PixelChange { x, y: 0, key }],
    }
}

// Checks that two operations raise the revision by exactly 2 and each entry keeps its author.
#[test]
fn revision_grows_and_each_entry_keeps_its_author() {
    let (mut project, sprite) = project_with_sprite();
    let mut history = History::new();
    let start = project.revision;

    history
        .apply(&mut project, USER, draw(&sprite, 0, 'N'))
        .unwrap();
    history
        .apply(&mut project, "claude-code", draw(&sprite, 1, 'N'))
        .unwrap();

    assert_eq!(project.revision, start + 2);
    let authors: Vec<&str> = history.entries().iter().map(|e| e.author.as_str()).collect();
    assert_eq!(authors, ["user", "claude-code"]);
}

// Checks that a rejected operation changes nothing: not the project, not the revision, not the history.
#[test]
fn a_failed_operation_leaves_everything_unchanged() {
    let (mut project, sprite) = project_with_sprite();
    let mut history = History::new();
    let before = project.clone();

    let err = history
        .apply(&mut project, USER, draw(&sprite, 0, 'X'))
        .unwrap_err();

    assert_eq!(err.code, "unknown_key");
    assert_eq!(project, before);
    assert!(history.entries().is_empty());
}

// Checks that every entry stores the inverse that undoes it.
#[test]
fn an_entry_stores_the_inverse() {
    let (mut project, sprite) = project_with_sprite();
    let mut history = History::new();
    let before = project.clone();
    history
        .apply(&mut project, USER, draw(&sprite, 2, 'N'))
        .unwrap();

    let entry = &history.entries()[0];
    for op in &entry.inverse {
        project.apply(op).unwrap();
    }
    assert_eq!(project.root, before.root);
}
