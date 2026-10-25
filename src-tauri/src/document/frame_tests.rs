use super::*;
use crate::history::History;

fn project_with_frames(count: usize) -> (Project, SpriteRef) {
    let mut project = Project::new("Kit");
    project
        .apply(&Operation::CreatePalette {
            palette: Palette {
                name: "main".to_string(),
                colors: "ABCDEF"
                    .chars()
                    .map(|key| Color {
                        key,
                        name: key.to_string(),
                        hex: "#112233".to_string(),
                    })
                    .collect(),
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
        project
            .apply(&Operation::AddFrame {
                sprite: sprite.clone(),
                duration_ms: 100,
            })
            .unwrap();
    }
    // Frame n starts with the n-th letter, so every frame is different.
    for n in 1..=count {
        project
            .apply(&Operation::SetPixels {
                sprite: sprite.clone(),
                layer: 0,
                frame: n as u32,
                pixels: vec![PixelChange {
                    x: 0,
                    y: 0,
                    key: (b'A' + n as u8 - 1) as char,
                }],
            })
            .unwrap();
    }
    (project, sprite)
}

fn tag(name: &str, from: u32, to: u32) -> Tag {
    Tag {
        name: name.to_string(),
        from,
        to,
        fps: 12,
        mode: TagMode::Loop,
    }
}

fn add_tag(project: &mut Project, sprite: &SpriteRef, name: &str, from: u32, to: u32) {
    project
        .apply(&Operation::AddTag {
            sprite: sprite.clone(),
            tag: tag(name, from, to),
            index: None,
        })
        .unwrap();
}

fn sprite(project: &Project) -> &Sprite {
    &project.root.sprites[0]
}

/// The first pixel of each frame: "ABCD" is the original order.
fn order(project: &Project) -> String {
    sprite(project).layers[0]
        .frames
        .iter()
        .map(|f| f.pixels[0] as char)
        .collect()
}

fn ranges(project: &Project) -> Vec<(String, u32, u32)> {
    sprite(project)
        .tags
        .iter()
        .map(|t| (t.name.clone(), t.from, t.to))
        .collect()
}

fn insert(sprite: &SpriteRef, at: u32, copy_of: Option<u32>) -> Operation {
    Operation::InsertFrame {
        sprite: sprite.clone(),
        at,
        copy_of,
        duration_ms: 100,
    }
}

fn mv(sprite: &SpriteRef, from: u32, to: u32) -> Operation {
    Operation::MoveFrame {
        sprite: sprite.clone(),
        from,
        to,
        tags: None,
    }
}

// Checks the "Add after the current frame" scenario: frame 2 of 4 is current,
// a blank frame is inserted at 3, so the sprite has 5 frames, the new one is
// frame 3 and the other frames keep their content.
#[test]
fn a_blank_frame_goes_after_the_current_one() {
    let (mut project, s) = project_with_frames(4);
    project.apply(&insert(&s, 3, None)).unwrap();
    assert_eq!(sprite(&project).frame_count(), 5);
    assert_eq!(order(&project), "AB.CD");
    assert_eq!(sprite(&project).frame_durations.len(), 5);
}

// Checks that a copy has the pixels and the duration of the frame it copies.
#[test]
fn a_copy_has_the_same_pixels_and_duration() {
    let (mut project, s) = project_with_frames(3);
    project
        .apply(&Operation::SetFrameDuration {
            sprite: s.clone(),
            frame: 2,
            duration_ms: 250,
        })
        .unwrap();
    project.apply(&insert(&s, 3, Some(2))).unwrap();
    assert_eq!(order(&project), "ABBC");
    assert_eq!(sprite(&project).frame_durations, vec![100, 250, 250, 100]);
}

// Checks bad positions: after the last frame plus one, 0, and a copy of a
// frame that does not exist.
#[test]
fn a_frame_cannot_go_out_of_range() {
    let (mut project, s) = project_with_frames(2);
    assert_eq!(project.apply(&insert(&s, 4, None)).unwrap_err().code, "bad_frame");
    assert_eq!(project.apply(&insert(&s, 0, None)).unwrap_err().code, "bad_frame");
    assert_eq!(project.apply(&insert(&s, 2, Some(9))).unwrap_err().code, "bad_frame");
    assert_eq!(sprite(&project).frame_count(), 2);
}

// Checks the tag ranges when a frame is inserted: inside a tag it grows,
// right after it nothing changes, before it the tag moves; with overlapping
// tags each one follows the rule on its own.
#[test]
fn tags_follow_an_inserted_frame_even_when_they_overlap() {
    let (mut project, s) = project_with_frames(5);
    add_tag(&mut project, &s, "walk", 1, 4);
    add_tag(&mut project, &s, "idle", 2, 2);
    add_tag(&mut project, &s, "last", 5, 5);

    project.apply(&insert(&s, 2, None)).unwrap();
    assert_eq!(
        ranges(&project),
        vec![
            ("walk".to_string(), 1, 5),
            ("idle".to_string(), 3, 3),
            ("last".to_string(), 6, 6)
        ]
    );

    // Right after the end of "walk" (frame 6): it does not grow.
    project.apply(&insert(&s, 6, None)).unwrap();
    assert_eq!(ranges(&project)[0], ("walk".to_string(), 1, 5));
    // At the very start, every tag moves.
    project.apply(&insert(&s, 1, None)).unwrap();
    assert_eq!(ranges(&project)[0], ("walk".to_string(), 2, 6));
}

// Checks that one undo takes back an inserted frame with the tag ranges as
// they were, and redo puts it back.
#[test]
fn undo_of_an_insert_restores_frames_and_tags() {
    let (mut project, s) = project_with_frames(4);
    add_tag(&mut project, &s, "walk", 1, 4);
    let before = project.clone();
    let mut history = History::new();
    history.apply(&mut project, "user", vec![insert(&s, 3, Some(1))]).unwrap();
    assert_eq!(ranges(&project)[0].2, 5);
    history.undo(&mut project).unwrap();
    assert_eq!(project.root, before.root);
    history.redo(&mut project).unwrap();
    assert_eq!(order(&project), "ABACD");
}

// Checks the "Delete the only frame" rule in the core and the tag ranges of a
// delete with overlapping tags; undo gives everything back.
#[test]
fn the_last_frame_stays_and_delete_shrinks_tags() {
    let (mut project, s) = project_with_frames(1);
    let delete = Operation::DeleteFrame { sprite: s.clone(), frame: 1 };
    assert_eq!(project.apply(&delete).unwrap_err().code, "last_frame");

    let (mut project, s) = project_with_frames(4);
    add_tag(&mut project, &s, "walk", 1, 4);
    add_tag(&mut project, &s, "idle", 2, 2);
    let before = project.clone();
    let mut history = History::new();
    history
        .apply(&mut project, "user", vec![Operation::DeleteFrame { sprite: s.clone(), frame: 2 }])
        .unwrap();
    // "idle" had only frame 2, so it goes; "walk" shrinks.
    assert_eq!(ranges(&project), vec![("walk".to_string(), 1, 3)]);
    history.undo(&mut project).unwrap();
    assert_eq!(project.root, before.root);
}

// Checks the "Move a frame" scenario: frame 4 goes before frame 2, the order
// is 1, 4, 2, 3 and each frame keeps its own pixels and duration.
#[test]
fn moving_a_frame_keeps_its_content() {
    let (mut project, s) = project_with_frames(4);
    project
        .apply(&Operation::SetFrameDuration { sprite: s.clone(), frame: 4, duration_ms: 400 })
        .unwrap();
    project.apply(&mv(&s, 4, 2)).unwrap();
    assert_eq!(order(&project), "ADBC");
    assert_eq!(sprite(&project).frame_durations, vec![100, 400, 100, 100]);
}

// Checks that tags keep the same frames after a move: a tag of the moved
// frame follows it, tags of the frames that were pushed move one place, and
// overlapping tags are each recomputed.
#[test]
fn tags_follow_the_frames_they_hold_when_one_moves() {
    let (mut project, s) = project_with_frames(4);
    add_tag(&mut project, &s, "all", 1, 4);
    add_tag(&mut project, &s, "mid", 2, 3);
    add_tag(&mut project, &s, "four", 4, 4);
    project.apply(&mv(&s, 4, 2)).unwrap();
    assert_eq!(
        ranges(&project),
        vec![
            ("all".to_string(), 1, 4),
            ("mid".to_string(), 3, 4),
            ("four".to_string(), 2, 2)
        ]
    );
}

// Checks the ambiguous case: a tag whose frames are no longer next to each
// other covers the smallest range that holds them all.
#[test]
fn a_tag_that_is_split_by_a_move_covers_all_its_frames() {
    let (mut project, s) = project_with_frames(4);
    add_tag(&mut project, &s, "walk", 1, 3);
    project.apply(&mv(&s, 2, 4)).unwrap();
    assert_eq!(order(&project), "ACDB");
    assert_eq!(ranges(&project), vec![("walk".to_string(), 1, 4)]);
}

// Checks that undo of a move gives back the frames and the exact tag ranges,
// also the ones that were widened.
#[test]
fn undo_of_a_move_restores_the_tags() {
    let (mut project, s) = project_with_frames(4);
    add_tag(&mut project, &s, "walk", 1, 3);
    let before = project.clone();
    let mut history = History::new();
    history.apply(&mut project, "user", vec![mv(&s, 2, 4)]).unwrap();
    history.undo(&mut project).unwrap();
    assert_eq!(project.root, before.root);
    history.redo(&mut project).unwrap();
    assert_eq!(order(&project), "ACDB");
}

// Checks that moving to the same place changes nothing and bad numbers fail.
#[test]
fn a_move_checks_its_frames() {
    let (mut project, s) = project_with_frames(3);
    let before = project.clone();
    project.apply(&mv(&s, 2, 2)).unwrap();
    assert_eq!(project.root, before.root);
    assert_eq!(project.apply(&mv(&s, 0, 2)).unwrap_err().code, "bad_frame");
    assert_eq!(project.apply(&mv(&s, 1, 9)).unwrap_err().code, "bad_frame");
}

// Checks the duration of a frame: set, undo, and 0 is refused.
#[test]
fn a_frame_duration_can_be_set_and_undone() {
    let (mut project, s) = project_with_frames(2);
    let mut history = History::new();
    let set = |ms| Operation::SetFrameDuration { sprite: s.clone(), frame: 2, duration_ms: ms };
    history.apply(&mut project, "user", vec![set(250)]).unwrap();
    assert_eq!(sprite(&project).frame_durations, vec![100, 250]);
    history.undo(&mut project).unwrap();
    assert_eq!(sprite(&project).frame_durations, vec![100, 100]);
    assert_eq!(project.apply(&set(0)).unwrap_err().code, "bad_duration");
}

// Checks editing a tag: change the range, speed and mode, rename it, and the
// undo; a name that another tag uses is refused, and so is a bad range.
#[test]
fn a_tag_can_be_edited_and_renamed() {
    let (mut project, s) = project_with_frames(4);
    add_tag(&mut project, &s, "walk", 1, 2);
    add_tag(&mut project, &s, "idle", 3, 3);
    let mut history = History::new();
    let edit = |name: &str, new: Tag| Operation::SetTag {
        sprite: s.clone(),
        name: name.to_string(),
        tag: new,
    };
    let mut run = tag("run", 1, 4);
    run.fps = 24;
    run.mode = TagMode::PingPong;
    history.apply(&mut project, "user", vec![edit("walk", run)]).unwrap();
    assert_eq!(sprite(&project).tags[0].name, "run");
    assert_eq!(sprite(&project).tags[0].fps, 24);
    history.undo(&mut project).unwrap();
    assert_eq!(sprite(&project).tags[0], tag("walk", 1, 2));

    assert_eq!(project.apply(&edit("walk", tag("IDLE", 1, 2))).unwrap_err().code, "tag_exists");
    assert_eq!(project.apply(&edit("walk", tag("walk", 3, 2))).unwrap_err().code, "bad_range");
    assert_eq!(project.apply(&edit("nope", tag("x", 1, 2))).unwrap_err().code, "not_found");
    // The same name with other values is fine.
    project.apply(&edit("walk", tag("Walk", 1, 3))).unwrap();
}

// Checks the "Default duration" scenario: at 12 fps a frame that has no
// duration of its own lasts about 83 ms; a frame with its own duration keeps
// it, and the speed 1 to 60 gives 1000 ms to 17 ms.
#[test]
fn the_speed_decides_the_duration_of_a_frame_without_one() {
    assert_eq!(frame_duration_ms(DEFAULT_FRAME_MS, 12), 83);
    assert_eq!(frame_duration_ms(DEFAULT_FRAME_MS, 1), 1000);
    assert_eq!(frame_duration_ms(DEFAULT_FRAME_MS, 60), 17);
    assert_eq!(frame_duration_ms(250, 12), 250);
}
