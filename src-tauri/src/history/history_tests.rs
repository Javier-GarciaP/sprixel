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
        .apply(&mut project, USER, vec![draw(&sprite, 0, 'N')])
        .unwrap();
    history
        .apply(&mut project, "claude-code", vec![draw(&sprite, 1, 'N')])
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
        .apply(&mut project, USER, vec![draw(&sprite, 0, 'X')])
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
        .apply(&mut project, USER, vec![draw(&sprite, 2, 'N')])
        .unwrap();

    let entry = &history.entries()[0];
    for op in &entry.inverse {
        project.apply(op).unwrap();
    }
    assert_eq!(project.root, before.root);
}

// Checks that a group of changes becomes one history entry and one revision.
#[test]
fn a_group_is_one_entry_and_one_revision() {
    let (mut project, sprite) = project_with_sprite();
    let mut history = History::new();
    let start = project.revision;
    let ops = (0..5).map(|x| draw(&sprite, x % 4, 'N')).collect();

    history.apply(&mut project, "claude-code", ops).unwrap();

    assert_eq!(history.entries().len(), 1);
    assert_eq!(history.entries()[0].forward.len(), 5);
    assert_eq!(project.revision, start + 1);
}

// Checks that an AI call whose last step is invalid leaves the project and the history untouched.
#[test]
fn a_failed_group_leaves_project_and_history_untouched() {
    let (mut project, sprite) = project_with_sprite();
    let mut history = History::new();
    let before = project.clone();
    let mut ops: Vec<Operation> = (0..40).map(|i| draw(&sprite, i % 4, 'N')).collect();
    ops.push(draw(&sprite, 0, 'X'));

    assert!(history.apply(&mut project, "claude-code", ops).is_err());

    assert_eq!(project, before);
    assert!(history.entries().is_empty());
}

// Checks that an action with no operations adds no entry and no revision.
#[test]
fn an_empty_action_is_ignored() {
    let (mut project, _) = project_with_sprite();
    let mut history = History::new();
    let start = project.revision;
    history.apply(&mut project, USER, vec![]).unwrap();
    assert!(history.entries().is_empty());
    assert_eq!(project.revision, start);
}

fn pixels_of(project: &Project) -> Vec<u8> {
    project.root.sprites[0].layers[0].frames[0].pixels.clone()
}

// Checks that undoing an AI action returns the project to the state before it,
// and that the entry can be redone.
#[test]
fn undo_an_ai_action_and_redo_it() {
    let (mut project, sprite) = project_with_sprite();
    let mut history = History::new();
    let before = pixels_of(&project);
    history
        .apply(&mut project, "claude-code", vec![draw(&sprite, 0, 'N')])
        .unwrap();
    let after = pixels_of(&project);

    history.undo(&mut project).unwrap();
    assert_eq!(pixels_of(&project), before);
    assert!(history.can_redo());

    history.redo(&mut project).unwrap();
    assert_eq!(pixels_of(&project), after);
    assert!(!history.can_redo());
}

// Checks that undo goes back one entry at a time, newest first, for any author.
#[test]
fn undo_goes_in_order_for_any_author() {
    let (mut project, sprite) = project_with_sprite();
    let mut history = History::new();
    history
        .apply(&mut project, USER, vec![draw(&sprite, 0, 'N')])
        .unwrap();
    let first = pixels_of(&project);
    history
        .apply(&mut project, "claude-code", vec![draw(&sprite, 1, 'N')])
        .unwrap();

    history.undo(&mut project).unwrap();
    assert_eq!(pixels_of(&project), first);
    history.undo(&mut project).unwrap();
    assert_eq!(pixels_of(&project), vec![b'.'; 16]);
}

// Checks that a new entry after an undo clears the redo entries.
#[test]
fn a_new_entry_clears_redo() {
    let (mut project, sprite) = project_with_sprite();
    let mut history = History::new();
    history
        .apply(&mut project, USER, vec![draw(&sprite, 0, 'N')])
        .unwrap();
    history.undo(&mut project).unwrap();
    assert!(history.can_redo());

    history
        .apply(&mut project, USER, vec![draw(&sprite, 1, 'N')])
        .unwrap();

    assert!(!history.can_redo());
    assert_eq!(history.redo(&mut project).unwrap_err().code, "nothing_to_redo");
}

// Checks the errors when there is nothing to undo or redo.
#[test]
fn undo_and_redo_with_an_empty_history_fail() {
    let (mut project, _) = project_with_sprite();
    let mut history = History::new();
    assert_eq!(history.undo(&mut project).unwrap_err().code, "nothing_to_undo");
    assert_eq!(history.redo(&mut project).unwrap_err().code, "nothing_to_redo");
}

// Checks that undo and redo are changes too: they raise the revision.
#[test]
fn undo_and_redo_raise_the_revision() {
    let (mut project, sprite) = project_with_sprite();
    let mut history = History::new();
    history
        .apply(&mut project, USER, vec![draw(&sprite, 0, 'N')])
        .unwrap();
    let revision = project.revision;
    history.undo(&mut project).unwrap();
    assert_eq!(project.revision, revision + 1);
    history.redo(&mut project).unwrap();
    assert_eq!(project.revision, revision + 2);
}

// Checks that undoing a delete returns the sprite and reports what changed, so storage can restore the file.
#[test]
fn undo_a_delete_restores_the_sprite() {
    let (mut project, sprite) = project_with_sprite();
    let mut history = History::new();
    history
        .apply(&mut project, USER, vec![Operation::DeleteSprite { sprite: sprite.clone() }])
        .unwrap();
    assert!(project.root.sprites.is_empty());

    let touched = history.undo(&mut project).unwrap();

    assert_eq!(project.root.sprites.len(), 1);
    assert_eq!(touched, vec![Target::Sprite(sprite)]);
}

// Checks that "undo last AI action" undoes the latest entry when an AI made it.
#[test]
fn undo_last_ai_action_when_it_is_the_latest() {
    let (mut project, sprite) = project_with_sprite();
    let mut history = History::new();
    history
        .apply(&mut project, USER, vec![draw(&sprite, 0, 'N')])
        .unwrap();
    let before_ai = pixels_of(&project);
    history
        .apply(&mut project, "claude-code", vec![draw(&sprite, 1, 'N')])
        .unwrap();

    history.undo_last_ai_action(&mut project).unwrap();

    assert_eq!(pixels_of(&project), before_ai);
    assert!(history.can_redo());
}

// Checks that if the user drew after the AI action the request fails with
// `newer_changes` and nothing is undone.
#[test]
fn undo_last_ai_action_fails_after_a_newer_user_change() {
    let (mut project, sprite) = project_with_sprite();
    let mut history = History::new();
    history
        .apply(&mut project, "claude-code", vec![draw(&sprite, 0, 'N')])
        .unwrap();
    history
        .apply(&mut project, USER, vec![draw(&sprite, 1, 'N')])
        .unwrap();
    let before = project.clone();

    let err = history.undo_last_ai_action(&mut project).unwrap_err();

    assert_eq!(err.code, "newer_changes");
    assert_eq!(project, before);
    assert_eq!(history.entries().len(), 2);
}

// Checks that without any AI action in the history the request fails with `no_ai_action`.
#[test]
fn undo_last_ai_action_without_ai_entries_fails() {
    let (mut project, sprite) = project_with_sprite();
    let mut history = History::new();
    assert_eq!(
        history.undo_last_ai_action(&mut project).unwrap_err().code,
        "no_ai_action"
    );
    history
        .apply(&mut project, USER, vec![draw(&sprite, 0, 'N')])
        .unwrap();
    assert_eq!(
        history.undo_last_ai_action(&mut project).unwrap_err().code,
        "no_ai_action"
    );
}
