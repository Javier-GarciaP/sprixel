//! Animations (tags) of a sprite: the no-overlap rule for new animations.

use super::animation::MAX_NEW_FRAMES;
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

// ---- Animations made in one step ----

use crate::history::History;

fn letter_frames(count: u32) -> (Project, SpriteRef) {
    // Frame n has the pixel `n` so that copies can be told apart.
    let (mut project, s) = project_with_frames(count);
    for n in 1..=count {
        let key = if n == 1 { 'A' } else { '.' };
        project
            .apply(&Operation::SetPixels {
                sprite: s.clone(),
                layer: 0,
                frame: n,
                pixels: vec![PixelChange { x: n % 2, y: 0, key }],
            })
            .unwrap();
    }
    (project, s)
}

fn run(project: &mut Project, history: &mut History, ops: Vec<Operation>) {
    history.apply(project, "tester", ops).unwrap();
}

// Checks the "Four frames from the base" scenario: a one-frame sprite gets 4
// copies of frame 1 and `walk` is a loop at 6 fps over frames 2 to 5, in ONE
// history entry, and one undo gives the sprite back.
#[test]
fn create_animation_copies_the_base() {
    let (mut project, s) = letter_frames(1);
    let before = project.clone();
    let mut history = History::new();
    let ops = create_animation(&project, &s, "walk", 4, 6, TagMode::Loop, 1, None).unwrap();
    run(&mut project, &mut history, ops);

    let sprite = project.find_sprite(&s).unwrap();
    assert_eq!(sprite.frame_count(), 5);
    for frame in &sprite.layers[0].frames {
        assert_eq!(frame, &sprite.layers[0].frames[0]);
    }
    assert_eq!(sprite.tags, vec![Tag { name: "walk".into(), from: 2, to: 5, fps: 6, mode: TagMode::Loop }]);
    assert_eq!(history.entries().len(), 1);
    history.undo(&mut project).unwrap();
    assert_eq!(project.root, before.root);
}

// Checks `at` and `base`: the copies go in the middle, the base can be any
// frame (even one that moves while the copies go in), and the animations after
// the new place move with their frames.
#[test]
fn create_animation_can_choose_the_base_and_the_place() {
    let (mut project, s) = letter_frames(3);
    add(&mut project, &s, tag("idle", 3, 3));
    // Base is frame 3 and the copies go before it, at 2.
    let ops = create_animation(&project, &s, "jump", 2, 8, TagMode::Once, 3, Some(2)).unwrap();
    project.apply_group(&ops).unwrap();

    let sprite = project.find_sprite(&s).unwrap();
    let frames = &sprite.layers[0].frames;
    assert_eq!(sprite.frame_count(), 5);
    assert_eq!(frames[1], frames[4], "copy of the old frame 3");
    assert_eq!(frames[2], frames[4]);
    let jump = sprite.tags.iter().find(|t| t.name == "jump").unwrap();
    assert_eq!((jump.from, jump.to, jump.mode), (2, 3, TagMode::Once));
    let idle = sprite.tags.iter().find(|t| t.name == "idle").unwrap();
    assert_eq!((idle.from, idle.to), (5, 5));
}

// Checks the refusals: the name exists (ignoring case), bad speed, bad base,
// a place inside another animation, and too many frames. Nothing changes.
#[test]
fn create_animation_refuses_bad_input() {
    let (mut project, s) = letter_frames(4);
    add(&mut project, &s, tag("walk", 1, 3));
    let before = project.clone();
    let code = |name: &str, n: usize, fps: u32, base: u32, at: Option<u32>| {
        create_animation(&project, &s, name, n, fps, TagMode::Loop, base, at).unwrap_err().code
    };
    assert_eq!(code("WALK", 2, 6, 1, None), "name_exists");
    assert_eq!(code("run", 2, 0, 1, None), "bad_fps");
    assert_eq!(code("run", 2, 6, 9, None), "bad_frame");
    assert_eq!(code("run", 2, 6, 1, Some(9)), "bad_frame");
    assert_eq!(code("run", 2, 6, 1, Some(2)), "animation_overlap");
    assert_eq!(code("run", 0, 6, 1, None), "bad_argument");
    assert_eq!(code("run", MAX_NEW_FRAMES + 1, 6, 1, None), "bad_argument");
    // Right after an animation is fine.
    create_animation(&project, &s, "run", 2, 6, TagMode::Loop, 1, Some(4)).unwrap();
    assert_eq!(project.root, before.root);
}

// Checks duplicate: the copy has the same pixels, speed and mode, goes to the
// end, and has the new name; one undo removes it.
#[test]
fn duplicate_animation_copies_frames_and_speed() {
    let (mut project, s) = letter_frames(4);
    add(&mut project, &s, Tag { fps: 9, mode: TagMode::PingPong, ..tag("walk", 2, 3) });
    let before = project.clone();
    let mut history = History::new();
    let ops = duplicate_animation(&project, &s, "WALK", "walk2", None).unwrap();
    run(&mut project, &mut history, ops);

    let sprite = project.find_sprite(&s).unwrap();
    let frames = &sprite.layers[0].frames;
    assert_eq!(sprite.frame_count(), 6);
    assert_eq!((&frames[4], &frames[5]), (&frames[1], &frames[2]));
    let copy = sprite.tags.iter().find(|t| t.name == "walk2").unwrap();
    assert_eq!((copy.from, copy.to, copy.fps, copy.mode), (5, 6, 9, TagMode::PingPong));
    assert_eq!(duplicate_animation(&project, &s, "walk", "Walk2", None).unwrap_err().code, "name_exists");
    assert_eq!(duplicate_animation(&project, &s, "nope", "x", None).unwrap_err().code, "not_found");
    history.undo(&mut project).unwrap();
    assert_eq!(project.root, before.root);
}

// Checks that duplicating to a place before the source follows the source
// frames while they move.
#[test]
fn duplicate_animation_before_the_source() {
    let (mut project, s) = letter_frames(3);
    add(&mut project, &s, tag("walk", 2, 3));
    let ops = duplicate_animation(&project, &s, "walk", "back", Some(1)).unwrap();
    project.apply_group(&ops).unwrap();
    let sprite = project.find_sprite(&s).unwrap();
    let frames = &sprite.layers[0].frames;
    assert_eq!((&frames[0], &frames[1]), (&frames[3], &frames[4]));
    let walk = sprite.tags.iter().find(|t| t.name == "walk").unwrap();
    assert_eq!((walk.from, walk.to), (4, 5));
}

// Checks rename: only the name changes; a used name is refused; undo works.
#[test]
fn rename_animation_changes_only_the_name() {
    let (mut project, s) = letter_frames(4);
    add(&mut project, &s, tag("walk", 1, 2));
    add(&mut project, &s, tag("jump", 3, 4));
    let before = project.clone();
    let mut history = History::new();

    assert_eq!(rename_animation(&project, &s, "walk", "JUMP").unwrap_err().code, "name_exists");
    assert_eq!(rename_animation(&project, &s, "nope", "x").unwrap_err().code, "not_found");
    // A different case of its own name is allowed.
    let ops = rename_animation(&project, &s, "walk", "Walk").unwrap();
    run(&mut project, &mut history, ops);
    let ops = rename_animation(&project, &s, "walk", "stroll").unwrap();
    run(&mut project, &mut history, ops);

    let sprite = project.find_sprite(&s).unwrap();
    assert_eq!(sprite.tags[0], tag("stroll", 1, 2));
    history.undo(&mut project).unwrap();
    history.undo(&mut project).unwrap();
    assert_eq!(project.root, before.root);
}

// Checks delete: by default only the tag goes; with frames the frames go and
// later animations follow their frames; one undo gives everything back.
#[test]
fn delete_animation_keeps_or_removes_the_frames() {
    let (mut project, s) = letter_frames(6);
    add(&mut project, &s, tag("walk", 2, 3));
    add(&mut project, &s, tag("jump", 5, 6));
    let before = project.clone();
    let mut history = History::new();

    let ops = delete_animation(&project, &s, "walk", false).unwrap();
    run(&mut project, &mut history, ops);
    let sprite = project.find_sprite(&s).unwrap();
    assert_eq!((sprite.frame_count(), sprite.tags.len()), (6, 1));
    history.undo(&mut project).unwrap();

    let ops = delete_animation(&project, &s, "walk", true).unwrap();
    run(&mut project, &mut history, ops);
    let sprite = project.find_sprite(&s).unwrap();
    assert_eq!(sprite.frame_count(), 4);
    assert_eq!(sprite.tags, vec![tag("jump", 3, 4)]);
    assert_eq!(history.entries().len(), 1, "the whole delete is one entry");
    history.undo(&mut project).unwrap();
    assert_eq!(project.root, before.root);
    assert_eq!(delete_animation(&project, &s, "nope", false).unwrap_err().code, "not_found");
}

// Checks that deleting an animation that holds every frame is refused (a
// sprite keeps one frame) and nothing changes.
#[test]
fn delete_animation_cannot_remove_the_last_frame() {
    let (mut project, s) = letter_frames(2);
    add(&mut project, &s, tag("all", 1, 2));
    let before = project.clone();
    let ops = delete_animation(&project, &s, "all", true).unwrap();
    assert_eq!(project.apply_group(&ops).unwrap_err().code, "last_frame");
    assert_eq!(project.root, before.root);
}
