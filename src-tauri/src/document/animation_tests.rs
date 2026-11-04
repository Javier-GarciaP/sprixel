//! Animations (tags) of a sprite: the no-overlap rule for new animations.

use super::*;

fn project_with_frames(count: u32) -> (Project, SpriteRef) {
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
        .apply(&Operation::CreateSprite {
            folder: vec![],
            name: "goblin".to_string(),
            width: 2,
            height: 1,
            palette: "main".to_string(),
        })
        .unwrap();
    let sprite = SpriteRef::new(&[], "goblin");
    for _ in 1..count {
        project.apply(&Operation::AddFrame { sprite: sprite.clone(), duration_ms: 100 }).unwrap();
    }
    (project, sprite)
}

fn tag(name: &str, from: u32, to: u32) -> Tag {
    Tag { name: name.to_string(), from, to, fps: 12, mode: TagMode::Loop }
}

fn add(project: &mut Project, sprite: &SpriteRef, tag: Tag) {
    project.apply(&Operation::AddTag { sprite: sprite.clone(), tag, index: None }).unwrap();
}

// Checks the "Overlap refused" scenario: a new animation that shares a frame
// with another one gets `animation_overlap`. Touching ranges are fine.
#[test]
fn a_new_animation_cannot_share_a_frame() {
    let (mut project, s) = project_with_frames(8);
    add(&mut project, &s, tag("walk", 2, 4));

    for (from, to) in [(1, 2), (4, 6), (3, 3), (1, 8)] {
        let error = project.check_new_animation(&s, &tag("run", from, to), None).unwrap_err();
        assert_eq!(error.code, "animation_overlap", "{from}-{to}");
    }
    project.check_new_animation(&s, &tag("run", 5, 6), None).unwrap();
    project.check_new_animation(&s, &tag("idle", 1, 1), None).unwrap();
}

// Checks that the other checks keep their codes and come first, so the app and
// the tools answer with the same error for the same input.
#[test]
fn the_other_errors_keep_their_codes() {
    let (mut project, s) = project_with_frames(4);
    add(&mut project, &s, tag("walk", 1, 2));

    let code = |t: Tag| project.check_new_animation(&s, &t, None).unwrap_err().code;
    assert_eq!(code(tag("x", 3, 9)), "bad_range");
    assert_eq!(code(Tag { fps: 0, ..tag("x", 3, 4) }), "bad_fps");
    // The same name wins over the overlap.
    assert_eq!(code(tag("WALK", 1, 2)), "tag_exists");
}

// Checks changing an animation: its own frames are not an overlap, a rename
// or a speed change of an old overlapping file works, and moving it onto
// another animation is refused.
#[test]
fn changing_an_animation_ignores_itself() {
    let (mut project, s) = project_with_frames(6);
    add(&mut project, &s, tag("walk", 1, 3));
    add(&mut project, &s, tag("jump", 4, 5));

    project.check_new_animation(&s, &tag("walk", 1, 3), Some("walk")).unwrap();
    project.check_new_animation(&s, &tag("stroll", 2, 3), Some("walk")).unwrap();
    let error = project.check_new_animation(&s, &tag("walk", 1, 4), Some("walk")).unwrap_err();
    assert_eq!(error.code, "animation_overlap");
}

// Checks that files that already overlap are kept as they are: the core still
// accepts such tags with `AddTag` (undo of a delete needs it), and renaming one
// without changing its frames is allowed.
#[test]
fn old_overlapping_files_are_kept() {
    let (mut project, s) = project_with_frames(6);
    add(&mut project, &s, tag("walk", 1, 4));
    add(&mut project, &s, tag("run", 3, 6));

    project.check_new_animation(&s, &tag("sprint", 3, 6), Some("run")).unwrap();
    let error = project.check_new_animation(&s, &tag("run", 2, 6), Some("run")).unwrap_err();
    assert_eq!(error.code, "animation_overlap");
    // Delete and undo brings the overlapping tag back.
    let mut history = crate::history::History::new();
    history.apply(&mut project, "user", vec![Operation::RemoveTag { sprite: s.clone(), name: "run".into() }]).unwrap();
    history.undo(&mut project).unwrap();
    assert_eq!(project.find_sprite(&s).unwrap().tags.len(), 2);
}
