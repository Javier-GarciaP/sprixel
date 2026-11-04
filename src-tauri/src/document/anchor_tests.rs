use super::*;

/// A 4x3 sprite `hero` with 3 frames.
fn setup() -> (Project, SpriteRef) {
    let mut project = Project::new("Kit");
    let ops = [
        Operation::CreatePalette {
            palette: Palette { name: "main".to_string(), colors: Vec::new() },
            index: None,
        },
        Operation::CreateSprite { folder: vec![], name: "hero".to_string(), width: 4, height: 3, palette: "main".to_string() },
    ];
    for op in ops {
        project.apply(&op).unwrap();
    }
    let s = SpriteRef::new(&[], "hero");
    for _ in 0..2 {
        project.apply(&Operation::AddFrame { sprite: s.clone(), duration_ms: 100 }).unwrap();
    }
    (project, s)
}

fn anchor(name: &str, x: u32, y: u32, frame: Option<u32>) -> Anchor {
    Anchor { name: name.to_string(), x, y, frame }
}

fn set(s: &SpriteRef, a: Anchor) -> Operation {
    Operation::SetAnchor { sprite: s.clone(), anchor: a, index: None }
}

fn anchors(project: &Project) -> Vec<(String, u32, u32, Option<u32>)> {
    project.root.sprites[0].anchors.iter().map(|a| (a.name.clone(), a.x, a.y, a.frame)).collect()
}

fn entry(name: &str, x: u32, y: u32, frame: Option<u32>) -> (String, u32, u32, Option<u32>) {
    (name.to_string(), x, y, frame)
}

// Checks that an anchor is added, that the same name and frame replaces it
// (the name ignores case), that another frame is another anchor, and that undo
// brings back each step.
#[test]
fn set_anchor_adds_replaces_and_undoes() {
    let (mut p, s) = setup();
    let added = p.apply(&set(&s, anchor("feet", 1, 2, None))).unwrap();
    let replaced = p.apply(&set(&s, anchor("FEET", 3, 0, None))).unwrap();
    assert_eq!(anchors(&p), vec![entry("FEET", 3, 0, None)]);
    p.apply(&set(&s, anchor("feet", 0, 0, Some(2)))).unwrap();
    assert_eq!(anchors(&p).len(), 2, "the frame one is a different anchor");

    p.apply(&replaced.inverse).unwrap();
    assert_eq!(anchors(&p)[0], entry("feet", 1, 2, None));
    p.apply(&added.inverse).unwrap();
    assert_eq!(anchors(&p), vec![entry("feet", 0, 0, Some(2))]);
}

// Checks the rules: inside the sprite, a frame that exists, a name that is
// valid. A refused anchor changes nothing.
#[test]
fn an_anchor_must_fit_the_sprite() {
    let (mut p, s) = setup();
    let revision = p.revision;
    assert_eq!(p.apply(&set(&s, anchor("a", 4, 0, None))).unwrap_err().code, "bad_anchor");
    assert_eq!(p.apply(&set(&s, anchor("a", 0, 3, None))).unwrap_err().code, "bad_anchor");
    assert_eq!(p.apply(&set(&s, anchor("a", 0, 0, Some(4)))).unwrap_err().code, "bad_frame");
    assert_eq!(p.apply(&set(&s, anchor("a", 0, 0, Some(0)))).unwrap_err().code, "bad_frame");
    assert_eq!(p.apply(&set(&s, anchor("  ", 0, 0, None))).unwrap_err().code, "bad_anchor");
    assert_eq!(p.revision, revision);
    assert!(anchors(&p).is_empty());
}

// Checks `RemoveAnchor`: it removes the right one, a missing one is
// `not_found`, and undo puts it back in the same place.
#[test]
fn remove_anchor_and_undo() {
    let (mut p, s) = setup();
    p.apply(&set(&s, anchor("a", 0, 0, None))).unwrap();
    p.apply(&set(&s, anchor("b", 1, 1, None))).unwrap();
    p.apply(&set(&s, anchor("c", 2, 2, None))).unwrap();
    let before = anchors(&p);
    let removed = p.apply(&Operation::RemoveAnchor { sprite: s.clone(), name: "B".to_string(), frame: None }).unwrap();
    assert_eq!(anchors(&p).len(), 2);
    p.apply(&removed.inverse).unwrap();
    assert_eq!(anchors(&p), before);
    let missing = Operation::RemoveAnchor { sprite: s.clone(), name: "b".to_string(), frame: Some(1) };
    assert_eq!(p.apply(&missing).unwrap_err().code, "not_found");
}

// Checks the "Frames change" scenario: deleting a frame drops its anchors and
// shifts the later ones; undoing the delete brings everything back exactly.
#[test]
fn deleting_a_frame_moves_anchors_and_undo_restores_them() {
    let (mut p, s) = setup();
    p.apply(&set(&s, anchor("hand", 1, 1, Some(2)))).unwrap();
    p.apply(&set(&s, anchor("hand", 2, 2, Some(3)))).unwrap();
    p.apply(&set(&s, anchor("feet", 0, 0, None))).unwrap();
    let before = anchors(&p);

    let deleted = p.apply(&Operation::DeleteFrame { sprite: s.clone(), frame: 2 }).unwrap();
    assert_eq!(anchors(&p), vec![entry("hand", 2, 2, Some(2)), entry("feet", 0, 0, None)]);

    p.apply(&deleted.inverse).unwrap();
    assert_eq!(anchors(&p), before);
}

// Checks that inserting a frame pushes the anchors of later frames, and that
// the undo of the insert brings them back.
#[test]
fn inserting_a_frame_shifts_anchors() {
    let (mut p, s) = setup();
    p.apply(&set(&s, anchor("hand", 1, 1, Some(1)))).unwrap();
    p.apply(&set(&s, anchor("hand", 2, 2, Some(2)))).unwrap();
    let inserted = p.apply(&Operation::InsertFrame { sprite: s.clone(), at: 2, copy_of: None, duration_ms: 100 }).unwrap();
    assert_eq!(anchors(&p), vec![entry("hand", 1, 1, Some(1)), entry("hand", 2, 2, Some(3))]);
    p.apply(&inserted.inverse).unwrap();
    assert_eq!(anchors(&p), vec![entry("hand", 1, 1, Some(1)), entry("hand", 2, 2, Some(2))]);
}

// Checks that anchors follow their frame when it moves, and that undo puts
// them back.
#[test]
fn moving_a_frame_moves_its_anchors() {
    let (mut p, s) = setup();
    p.apply(&set(&s, anchor("a", 0, 0, Some(1)))).unwrap();
    p.apply(&set(&s, anchor("b", 1, 1, Some(2)))).unwrap();
    p.apply(&set(&s, anchor("c", 2, 2, Some(3)))).unwrap();
    let before = anchors(&p);
    let moved = p.apply(&Operation::MoveFrame { sprite: s.clone(), from: 1, to: 3, anchors: None, tags: None }).unwrap();
    assert_eq!(anchors(&p), vec![entry("a", 0, 0, Some(3)), entry("b", 1, 1, Some(1)), entry("c", 2, 2, Some(2))]);
    p.apply(&moved.inverse).unwrap();
    assert_eq!(anchors(&p), before);
}

// Checks that a duplicated sprite keeps its anchors.
#[test]
fn a_duplicate_keeps_the_anchors() {
    let (mut p, s) = setup();
    p.apply(&set(&s, anchor("feet", 1, 2, None))).unwrap();
    p.apply(&Operation::DuplicateSprite { sprite: s.clone(), name: "hero2".to_string() }).unwrap();
    assert_eq!(p.root.sprites[1].anchors, p.root.sprites[0].anchors);
}
