use super::*;

/// A 4x3 sprite `hero` with 2 frames and 2 layers, with a tag, an own frame
/// duration and two anchors. Layer 1 frame 1 has `A` at 0,0 and `B` at 3,2.
fn setup() -> (Project, SpriteRef) {
    let mut project = Project::new("Kit");
    let ops = [
        Operation::CreatePalette { palette: Palette { name: "main".to_string(), colors: ['A', 'B', 'C'].iter().map(|k| Color { key: *k, name: k.to_string(), hex: "#112233".to_string() }).collect() }, index: None },
        Operation::CreateSprite { folder: vec![], name: "hero".to_string(), width: 4, height: 3, palette: "main".to_string() },
    ];
    for op in ops {
        project.apply(&op).unwrap();
    }
    let s = SpriteRef::new(&[], "hero");
    project.apply(&Operation::AddFrame { sprite: s.clone(), duration_ms: 100 }).unwrap();
    project.apply(&Operation::SetFrameDuration { sprite: s.clone(), frame: 2, duration_ms: 250 }).unwrap();
    project
        .apply(&Operation::AddTag {
            sprite: s.clone(),
            tag: Tag { name: "walk".to_string(), from: 1, to: 2, fps: 8, mode: TagMode::Loop },
            index: None,
        })
        .unwrap();
    for (name, x, y) in [("feet", 1, 2), ("head", 3, 0)] {
        let anchor = Anchor { name: name.to_string(), x, y, frame: None };
        project.apply(&Operation::SetAnchor { sprite: s.clone(), anchor, index: None }).unwrap();
    }
    let sprite = &mut project.root.sprites[0];
    sprite.layers.push(Layer { name: "Layer 2".to_string(), frames: vec![Frame::blank(4, 3), Frame::blank(4, 3)] });
    sprite.layers[0].frames[0].pixels[0] = b'A';
    sprite.layers[0].frames[0].pixels[11] = b'B';
    sprite.layers[1].frames[1].pixels[5] = b'C';
    (project, s)
}

fn resize(s: &SpriteRef, width: u32, height: u32, anchor: ResizeAnchor) -> Operation {
    Operation::ResizeSprite { sprite: s.clone(), width, height, anchor }
}

fn rows(project: &Project, layer: usize, frame: usize) -> Vec<String> {
    let sprite = &project.root.sprites[0];
    sprite.layers[layer].frames[frame]
        .pixels
        .chunks(sprite.width as usize)
        .map(|r| String::from_utf8(r.to_vec()).unwrap())
        .collect()
}

// Checks the spec scenario "Extend": a 4x3 sprite grows to 8x5 with the
// anchor `center`, the old pixels move by (2, 1) in every frame of every
// layer, and the tags, durations and palette do not change.
#[test]
fn extending_moves_the_content_and_keeps_the_rest() {
    let (mut p, s) = setup();
    p.apply(&resize(&s, 8, 5, ResizeAnchor::Center)).unwrap();

    let sprite = &p.root.sprites[0];
    assert_eq!((sprite.width, sprite.height), (8, 5));
    assert_eq!(rows(&p, 0, 0), vec!["........", "..A.....", "........", ".....B..", "........"]);
    assert_eq!(rows(&p, 1, 1)[2], "...C....");
    assert_eq!(sprite.layers[0].frames[1].pixels.len(), 40);
    assert_eq!(sprite.frame_durations, vec![100, 250]);
    assert_eq!(sprite.tags.len(), 1);
    assert_eq!(sprite.palette, "main");
    sprite.validate(p.palette("main").unwrap()).unwrap();
}

// Checks that cropping with a corner anchor cuts the other side, and that
// undo brings back the old size, ALL pixels (also the ones that were cut) and
// the anchors exactly. Redo (applying the operation again) gives the same result.
#[test]
fn cropping_can_be_undone_exactly() {
    let (mut p, s) = setup();
    let before = p.clone();
    let applied = p.apply(&resize(&s, 2, 2, ResizeAnchor::TopLeft)).unwrap();
    let cropped = p.clone();
    assert_eq!(rows(&p, 0, 0), vec!["A.", ".."]);

    p.apply(&applied.inverse).unwrap();
    assert_eq!(p.root, before.root, "the project is the same, pixels and anchors included");

    p.apply(&resize(&s, 2, 2, ResizeAnchor::TopLeft)).unwrap();
    assert_eq!(p.root, cropped.root, "redo gives the same result");
}

// Checks the spec scenario "Anchors": they move with the content, the ones
// that end outside are removed, and undo puts the removed one back.
#[test]
fn anchors_follow_the_content_and_undo_restores_them() {
    let (mut p, s) = setup();
    // 4x3 -> 3x3 with `right`: everything shifts by -1 in x. head (3,0) -> (2,0), feet (1,2) -> (0,2).
    let applied = p.apply(&resize(&s, 3, 3, ResizeAnchor::Right)).unwrap();
    let names: Vec<_> = p.root.sprites[0].anchors.iter().map(|a| (a.name.as_str(), a.x, a.y)).collect();
    assert_eq!(names, vec![("feet", 0, 2), ("head", 2, 0)]);

    // 3x3 -> 2x3 with `right` again: feet (0,2) falls outside.
    let second = p.apply(&resize(&s, 2, 3, ResizeAnchor::Right)).unwrap();
    let names: Vec<_> = p.root.sprites[0].anchors.iter().map(|a| a.name.as_str()).collect();
    assert_eq!(names, vec!["head"]);

    p.apply(&second.inverse).unwrap();
    assert_eq!(p.root.sprites[0].anchors.len(), 2);
    p.apply(&applied.inverse).unwrap();
    let back: Vec<_> = p.root.sprites[0].anchors.iter().map(|a| (a.name.as_str(), a.x, a.y)).collect();
    assert_eq!(back, vec![("feet", 1, 2), ("head", 3, 0)]);
}

// Checks the spec scenario "Limits": a side of 0 or more than 512, an unknown
// sprite and the same size are refused, and nothing changes, not even the revision.
#[test]
fn bad_sizes_are_refused_and_change_nothing() {
    let (mut p, s) = setup();
    let before = p.clone();
    for (w, h, code) in [(0, 3, "bad_size"), (4, 0, "bad_size"), (513, 3, "too_big"), (4, 600, "too_big"), (4, 3, "bad_size")] {
        let error = p.apply(&resize(&s, w, h, ResizeAnchor::Center)).unwrap_err();
        assert_eq!(error.code, code, "{w}x{h}");
    }
    let ghost = SpriteRef::new(&[], "ghost");
    assert_eq!(p.apply(&resize(&ghost, 8, 8, ResizeAnchor::Center)).unwrap_err().code, "not_found");
    assert_eq!(p, before);
}

// Checks that the limits themselves are allowed: 1x1 and 512x512.
#[test]
fn the_limits_are_allowed() {
    let (mut p, s) = setup();
    p.apply(&resize(&s, 512, 512, ResizeAnchor::TopLeft)).unwrap();
    assert_eq!(p.root.sprites[0].layers[0].frames[0].pixels.len(), 512 * 512);
    p.apply(&resize(&s, 1, 1, ResizeAnchor::TopLeft)).unwrap();
    assert_eq!(rows(&p, 0, 0), vec!["A"]);
}

// Checks that the operation is one step in a group, and that the whole group
// is undone with its inverses (the way the history does).
#[test]
fn it_works_in_a_group() {
    let (mut p, s) = setup();
    let before = p.root.clone();
    let group = p.apply_group(&[resize(&s, 6, 6, ResizeAnchor::BottomRight), resize(&s, 2, 2, ResizeAnchor::Top)]).unwrap();
    assert_eq!(group.touched.len(), 2);
    for inverse in &group.inverses {
        p.apply(inverse).unwrap();
    }
    assert_eq!(p.root, before);
}
