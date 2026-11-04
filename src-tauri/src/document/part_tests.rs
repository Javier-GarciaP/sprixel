//! Named parts: rectangles of a sprite that the region tools can use by name.

use super::*;
use crate::history::History;

fn project_with_sprite() -> (Project, SpriteRef) {
    let mut project = Project::new("Kit");
    project
        .apply(&Operation::CreatePalette {
            palette: Palette {
                name: "main".to_string(),
                colors: vec![Color { key: 'A', name: "a".to_string(), hex: "#112233".to_string() }],
            },
            index: None,
        })
        .unwrap();
    project
        .apply(&Operation::CreateSprite { folder: vec![], name: "goat".to_string(), width: 16, height: 8, palette: "main".to_string() })
        .unwrap();
    (project, SpriteRef::new(&[], "goat"))
}

fn part(name: &str, x: u32, y: u32, width: u32, height: u32) -> Part {
    Part { name: name.to_string(), x, y, width, height }
}

fn set(sprite: &SpriteRef, part: Part) -> Operation {
    Operation::SetPart { sprite: sprite.clone(), part, index: None }
}

// Checks that a new sprite has no parts (old projects open without them) and
// that SetPart adds a part and a second SetPart with the same name (any case)
// replaces it and keeps its place.
#[test]
fn parts_are_added_and_replaced_by_name() {
    let (mut project, s) = project_with_sprite();
    assert!(project.find_sprite(&s).unwrap().parts.is_empty());

    project.apply(&set(&s, part("head", 0, 0, 4, 4))).unwrap();
    project.apply(&set(&s, part("tail", 12, 0, 4, 4))).unwrap();
    project.apply(&set(&s, part("HEAD", 1, 1, 5, 5))).unwrap();

    assert_eq!(project.find_sprite(&s).unwrap().parts, vec![part("HEAD", 1, 1, 5, 5), part("tail", 12, 0, 4, 4)]);
}

// Checks the refusals: a part outside the sprite (`region_out`), an empty
// size or a bad name (`bad_part`), too many parts, and a sprite that is not
// there. Nothing changes.
#[test]
fn bad_parts_are_refused() {
    let (mut project, s) = project_with_sprite();
    let before = project.clone();
    let code = |project: &mut Project, p: Part| project.apply(&set(&s, p)).unwrap_err().code;

    assert_eq!(code(&mut project, part("a", 10, 0, 7, 2)), "region_out");
    assert_eq!(code(&mut project, part("a", 0, 7, 2, 2)), "region_out");
    assert_eq!(code(&mut project, part("a", 0, 0, 0, 2)), "bad_part");
    assert_eq!(code(&mut project, part("  ", 0, 0, 2, 2)), "bad_part");
    assert_eq!(code(&mut project, part(&"x".repeat(65), 0, 0, 2, 2)), "bad_part");
    for n in 0..MAX_PARTS {
        project.apply(&set(&s, part(&format!("p{n}"), 0, 0, 1, 1))).unwrap();
    }
    assert_eq!(code(&mut project, part("one-more", 0, 0, 1, 1)), "too_many_parts");
    // Replacing a part still works when the sprite is full.
    project.apply(&set(&s, part("P0", 1, 1, 2, 2))).unwrap();
    let ghost = SpriteRef::new(&[], "ghost");
    assert_eq!(project.apply(&set(&ghost, part("a", 0, 0, 1, 1))).unwrap_err().code, "not_found");
    assert_eq!(before.find_sprite(&s).unwrap().parts.len(), 0);
}

// Checks undo and redo: undo of a new part removes it, undo of a replace puts
// the old rectangle back, undo of a remove puts the part back in its place.
#[test]
fn parts_can_be_undone() {
    let (mut project, s) = project_with_sprite();
    let mut history = History::new();
    history.apply(&mut project, "tester", vec![set(&s, part("head", 0, 0, 4, 4))]).unwrap();
    history.apply(&mut project, "tester", vec![set(&s, part("tail", 12, 0, 4, 4))]).unwrap();
    let two = project.clone();

    history.apply(&mut project, "tester", vec![set(&s, part("head", 2, 2, 3, 3))]).unwrap();
    history.undo(&mut project).unwrap();
    assert_eq!(project.root, two.root);

    history.apply(&mut project, "tester", vec![Operation::RemovePart { sprite: s.clone(), name: "HEAD".into() }]).unwrap();
    assert_eq!(project.find_sprite(&s).unwrap().parts, vec![part("tail", 12, 0, 4, 4)]);
    history.undo(&mut project).unwrap();
    assert_eq!(project.root, two.root, "the head is back in first place");
    history.undo(&mut project).unwrap();
    history.undo(&mut project).unwrap();
    assert!(project.find_sprite(&s).unwrap().parts.is_empty());
    assert_eq!(project.apply(&Operation::RemovePart { sprite: s.clone(), name: "nope".into() }).unwrap_err().code, "not_found");
}
