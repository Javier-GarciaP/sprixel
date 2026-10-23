use super::*;

fn dungeon16() -> Palette {
    Palette {
        name: "dungeon16".to_string(),
        colors: "NPAR"
            .chars()
            .map(|key| Color {
                key,
                name: format!("color {key}"),
                hex: "#112233".to_string(),
            })
            .collect(),
    }
}

fn names(parts: &[&str]) -> Vec<String> {
    parts.iter().map(|p| p.to_string()).collect()
}

fn create_sprite(folder: &[&str], name: &str, w: u32, h: u32) -> Operation {
    Operation::CreateSprite {
        folder: names(folder),
        name: name.to_string(),
        width: w,
        height: h,
        palette: "dungeon16".to_string(),
    }
}

fn pixel(x: u32, y: u32, key: char) -> PixelChange {
    PixelChange { x, y, key }
}

fn set_pixels(sprite: &SpriteRef, pixels: Vec<PixelChange>) -> Operation {
    Operation::SetPixels {
        sprite: sprite.clone(),
        layer: 0,
        frame: 1,
        pixels,
    }
}

/// A project with the palette "dungeon16", the folder "Personajes" and a
/// 4x4 sprite "goblin" inside it.
fn project_with_goblin() -> (Project, SpriteRef) {
    let mut project = Project::new("Dungeon Kit");
    project
        .apply(&Operation::CreatePalette {
            palette: dungeon16(),
            index: None,
        })
        .unwrap();
    project
        .apply(&Operation::CreateFolder {
            parent: vec![],
            name: "Personajes".to_string(),
        })
        .unwrap();
    project
        .apply(&create_sprite(&["Personajes"], "goblin", 4, 4))
        .unwrap();
    (project, SpriteRef::new(&["Personajes"], "goblin"))
}

fn goblin(project: &Project) -> &Sprite {
    &project.root.folders[0].sprites[0]
}

/// The project without its revision, to compare content only.
fn content(project: &Project) -> Project {
    Project {
        revision: 0,
        ..project.clone()
    }
}

// Checks that a sprite is created with the palette it references.
#[test]
fn a_sprite_uses_a_palette() {
    let (project, _) = project_with_goblin();
    assert_eq!(goblin(&project).palette, "dungeon16");
    assert_eq!(goblin(&project).name, "goblin");
}

// Checks that a sprite cannot reference a palette that does not exist.
#[test]
fn a_sprite_cannot_use_a_missing_palette() {
    let (mut project, _) = project_with_goblin();
    let op = Operation::CreateSprite {
        folder: vec![],
        name: "orc".to_string(),
        width: 4,
        height: 4,
        palette: "nope".to_string(),
    };
    assert_eq!(project.apply(&op).unwrap_err().code, "palette_not_found");
}

// Checks that a 600x20 sprite is rejected with `too_big` and nothing changes.
#[test]
fn a_too_big_sprite_is_rejected_and_nothing_changes() {
    let (mut project, _) = project_with_goblin();
    let before = project.clone();
    let err = project.apply(&create_sprite(&[], "huge", 600, 20)).unwrap_err();
    assert_eq!(err.code, "too_big");
    assert_eq!(project, before);
}

// Checks that "Goblin" cannot be created in a folder that has "goblin".
#[test]
fn duplicate_names_are_rejected_ignoring_case() {
    let (mut project, _) = project_with_goblin();
    let before = project.clone();
    let err = project
        .apply(&create_sprite(&["personajes"], "Goblin", 4, 4))
        .unwrap_err();
    assert_eq!(err.code, "name_exists");
    assert_eq!(project, before);
}

// Checks that a sprite and a folder cannot share a name in the same folder.
#[test]
fn a_folder_and_a_sprite_cannot_share_a_name() {
    let (mut project, _) = project_with_goblin();
    let folder = Operation::CreateFolder {
        parent: names(&["Personajes"]),
        name: "GOBLIN".to_string(),
    };
    assert_eq!(project.apply(&folder).unwrap_err().code, "name_exists");
}

// Checks that names that are invalid on some system are rejected by the operations.
#[test]
fn invalid_names_are_rejected() {
    let (mut project, _) = project_with_goblin();
    assert_eq!(
        project.apply(&create_sprite(&[], "CON", 4, 4)).unwrap_err().code,
        "invalid_name"
    );
}

// Checks that a pixel with a key outside the palette is rejected and the sprite stays the same.
#[test]
fn a_pixel_with_an_unknown_key_is_rejected() {
    let (mut project, goblin_ref) = project_with_goblin();
    let before = project.clone();
    let err = project
        .apply(&set_pixels(&goblin_ref, vec![pixel(0, 0, 'X')]))
        .unwrap_err();
    assert_eq!(err.code, "unknown_key");
    assert_eq!(project, before);
}

// Checks that a pixel outside the sprite is rejected.
#[test]
fn a_pixel_outside_the_sprite_is_rejected() {
    let (mut project, goblin_ref) = project_with_goblin();
    let err = project
        .apply(&set_pixels(&goblin_ref, vec![pixel(4, 0, 'N')]))
        .unwrap_err();
    assert_eq!(err.code, "out_of_range");
}

// Checks that setting pixels changes them and that the inverse puts the old ones back,
// also when the same pixel is set twice in one operation.
#[test]
fn set_pixels_and_its_inverse() {
    let (mut project, goblin_ref) = project_with_goblin();
    let before = content(&project);
    let op = set_pixels(
        &goblin_ref,
        vec![pixel(1, 2, 'N'), pixel(1, 2, 'P'), pixel(3, 3, 'R')],
    );
    let applied = project.apply(&op).unwrap();
    let pixels = &goblin(&project).layers[0].frames[0].pixels;
    assert_eq!(pixels[2 * 4 + 1], b'P');
    assert_eq!(pixels[15], b'R');
    project.apply(&applied.inverse).unwrap();
    assert_eq!(content(&project), before);
}

// Checks that an operation is found ignoring case and reports the real names it touched.
#[test]
fn operations_find_things_ignoring_case_and_report_real_names() {
    let (mut project, _) = project_with_goblin();
    let wrong_case = SpriteRef::new(&["PERSONAJES"], "GOBLIN");
    let applied = project
        .apply(&set_pixels(&wrong_case, vec![pixel(0, 0, 'N')]))
        .unwrap();
    assert_eq!(
        applied.touched,
        vec![Target::Sprite(SpriteRef::new(&["Personajes"], "goblin"))]
    );
}

// Checks that the revision grows by one per applied operation and does not move on errors.
#[test]
fn the_revision_grows_by_one_and_not_on_errors() {
    let (mut project, goblin_ref) = project_with_goblin();
    let start = project.revision;
    project
        .apply(&set_pixels(&goblin_ref, vec![pixel(0, 0, 'N')]))
        .unwrap();
    project
        .apply(&set_pixels(&goblin_ref, vec![pixel(1, 0, 'N')]))
        .unwrap();
    assert_eq!(project.revision, start + 2);
    let _ = project.apply(&set_pixels(&goblin_ref, vec![pixel(0, 0, 'X')]));
    assert_eq!(project.revision, start + 2);
}

// Checks that deleting a sprite and applying its inverse gives the same project,
// with the sprite back in the same position.
#[test]
fn delete_sprite_and_its_inverse() {
    let (mut project, goblin_ref) = project_with_goblin();
    project
        .apply(&create_sprite(&["Personajes"], "orc", 4, 4))
        .unwrap();
    let before = content(&project);
    let applied = project
        .apply(&Operation::DeleteSprite { sprite: goblin_ref })
        .unwrap();
    assert_eq!(project.root.folders[0].sprites.len(), 1);
    project.apply(&applied.inverse).unwrap();
    assert_eq!(content(&project), before);
}

// Checks that deleting a folder with sprites and applying the inverse gives the same project.
#[test]
fn delete_folder_and_its_inverse() {
    let (mut project, _) = project_with_goblin();
    let before = content(&project);
    let applied = project
        .apply(&Operation::DeleteFolder {
            path: names(&["personajes"]),
        })
        .unwrap();
    assert!(project.root.folders.is_empty());
    assert_eq!(
        applied.touched,
        vec![Target::Folder(names(&["Personajes"]))]
    );
    project.apply(&applied.inverse).unwrap();
    assert_eq!(content(&project), before);
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

fn add_tag(sprite: &SpriteRef, tag: Tag) -> Operation {
    Operation::AddTag {
        sprite: sprite.clone(),
        tag,
        index: None,
    }
}

fn with_four_frames(project: &mut Project, sprite: &SpriteRef) {
    for _ in 0..3 {
        project
            .apply(&Operation::AddFrame {
                sprite: sprite.clone(),
                duration_ms: 80,
            })
            .unwrap();
    }
}

// Checks that a tag with frames 3 to 8 on a sprite with 4 frames is rejected with `bad_range`.
#[test]
fn a_tag_outside_the_frames_is_rejected() {
    let (mut project, goblin_ref) = project_with_goblin();
    with_four_frames(&mut project, &goblin_ref);
    let err = project.apply(&add_tag(&goblin_ref, tag("run", 3, 8))).unwrap_err();
    assert_eq!(err.code, "bad_range");
    assert!(goblin(&project).tags.is_empty());
}

// Checks that tag names are unique in a sprite, ignoring case.
#[test]
fn tag_names_must_be_unique() {
    let (mut project, goblin_ref) = project_with_goblin();
    with_four_frames(&mut project, &goblin_ref);
    project.apply(&add_tag(&goblin_ref, tag("walk", 1, 2))).unwrap();
    let err = project.apply(&add_tag(&goblin_ref, tag("WALK", 3, 4))).unwrap_err();
    assert_eq!(err.code, "tag_exists");
}

// Checks that adding a tag and removing it (the inverse) restores the project.
#[test]
fn add_tag_and_its_inverse() {
    let (mut project, goblin_ref) = project_with_goblin();
    with_four_frames(&mut project, &goblin_ref);
    let before = content(&project);
    let applied = project.apply(&add_tag(&goblin_ref, tag("walk", 1, 4))).unwrap();
    project.apply(&applied.inverse).unwrap();
    assert_eq!(content(&project), before);
}

// Checks that deleting a frame inside a tag shrinks the tag, moves later tags
// and removes a tag left without frames; the inverse brings everything back.
#[test]
fn deleting_a_frame_shrinks_the_tags() {
    let (mut project, goblin_ref) = project_with_goblin();
    with_four_frames(&mut project, &goblin_ref);
    project.apply(&add_tag(&goblin_ref, tag("walk", 1, 3))).unwrap();
    project.apply(&add_tag(&goblin_ref, tag("hit", 2, 2))).unwrap();
    project.apply(&add_tag(&goblin_ref, tag("idle", 4, 4))).unwrap();
    let before = content(&project);

    let applied = project
        .apply(&Operation::DeleteFrame {
            sprite: goblin_ref.clone(),
            frame: 2,
        })
        .unwrap();
    let sprite = goblin(&project);
    assert_eq!(sprite.frame_count(), 3);
    // "walk" lost one frame, "hit" had only frame 2 so it is gone, "idle" moved.
    assert_eq!(sprite.tags.len(), 2);
    assert_eq!((sprite.tags[0].from, sprite.tags[0].to), (1, 2));
    assert_eq!(sprite.tags[1].name, "idle");
    assert_eq!((sprite.tags[1].from, sprite.tags[1].to), (3, 3));

    project.apply(&applied.inverse).unwrap();
    assert_eq!(content(&project), before);
}

// Checks that the last frame of a sprite cannot be deleted.
#[test]
fn the_last_frame_cannot_be_deleted() {
    let (mut project, goblin_ref) = project_with_goblin();
    let err = project
        .apply(&Operation::DeleteFrame {
            sprite: goblin_ref,
            frame: 1,
        })
        .unwrap_err();
    assert_eq!(err.code, "last_frame");
}

// Checks that deleting a frame keeps the pixels of the other frames, and the inverse restores its pixels.
#[test]
fn delete_frame_keeps_other_frames_and_restores_its_own() {
    let (mut project, goblin_ref) = project_with_goblin();
    with_four_frames(&mut project, &goblin_ref);
    for (frame, key) in [(1, 'N'), (2, 'P'), (3, 'A')] {
        project
            .apply(&Operation::SetPixels {
                sprite: goblin_ref.clone(),
                layer: 0,
                frame,
                pixels: vec![pixel(0, 0, key)],
            })
            .unwrap();
    }
    let applied = project
        .apply(&Operation::DeleteFrame {
            sprite: goblin_ref,
            frame: 2,
        })
        .unwrap();
    let frames = &goblin(&project).layers[0].frames;
    assert_eq!((frames[0].pixels[0], frames[1].pixels[0]), (b'N', b'A'));
    project.apply(&applied.inverse).unwrap();
    assert_eq!(goblin(&project).layers[0].frames[1].pixels[0], b'P');
}

// Checks that a palette used by a sprite cannot be deleted, and an unused one can (and comes back).
#[test]
fn a_used_palette_cannot_be_deleted() {
    let (mut project, _) = project_with_goblin();
    let err = project
        .apply(&Operation::DeletePalette {
            name: "dungeon16".to_string(),
        })
        .unwrap_err();
    assert_eq!(err.code, "palette_in_use");

    let mut other = dungeon16();
    other.name = "other".to_string();
    project
        .apply(&Operation::CreatePalette {
            palette: other,
            index: Some(0),
        })
        .unwrap();
    let before = content(&project);
    let applied = project
        .apply(&Operation::DeletePalette {
            name: "OTHER".to_string(),
        })
        .unwrap();
    project.apply(&applied.inverse).unwrap();
    assert_eq!(content(&project), before);
}

// Checks that a palette name that already exists (ignoring case) is rejected.
#[test]
fn palette_names_are_unique() {
    let (mut project, _) = project_with_goblin();
    let mut same = dungeon16();
    same.name = "DUNGEON16".to_string();
    let err = project
        .apply(&Operation::CreatePalette {
            palette: same,
            index: None,
        })
        .unwrap_err();
    assert_eq!(err.code, "name_exists");
}

// Checks that a group is all or nothing: 41 changes where the 41st is invalid
// apply none of them, and the revision does not move.
#[test]
fn a_group_with_an_invalid_41st_change_applies_nothing() {
    let (mut project, goblin_ref) = project_with_goblin();
    let before = project.clone();
    let mut ops: Vec<Operation> = (0..40)
        .map(|i| set_pixels(&goblin_ref, vec![pixel(i % 4, i / 4 % 4, 'N')]))
        .collect();
    ops.push(set_pixels(&goblin_ref, vec![pixel(0, 0, 'X')]));

    let err = project.apply_group(&ops).unwrap_err();

    assert_eq!(err.code, "unknown_key");
    assert_eq!(project, before);
}

// Checks that one operation with 41 pixels where the last is invalid changes no pixel.
#[test]
fn one_operation_with_an_invalid_41st_pixel_applies_nothing() {
    let (mut project, goblin_ref) = project_with_goblin();
    let before = project.clone();
    let mut pixels: Vec<PixelChange> = (0..40).map(|i| pixel(i % 4, i / 4 % 4, 'N')).collect();
    pixels.push(pixel(0, 0, 'X'));

    assert!(project.apply(&set_pixels(&goblin_ref, pixels)).is_err());
    assert_eq!(project, before);
}

// Checks that a failing group also undoes steps that changed the structure
// (a delete, a create), not only pixels.
#[test]
fn a_failing_group_rolls_back_structural_changes() {
    let (mut project, goblin_ref) = project_with_goblin();
    let before = project.clone();
    let ops = vec![
        create_sprite(&["Personajes"], "orc", 4, 4),
        Operation::DeleteSprite { sprite: goblin_ref },
        create_sprite(&["Personajes"], "ORC", 4, 4),
    ];

    assert_eq!(project.apply_group(&ops).unwrap_err().code, "name_exists");
    assert_eq!(project, before);
}

// Checks that a good group raises the revision by exactly 1 and its inverses undo all of it.
#[test]
fn a_good_group_is_one_revision_and_can_be_undone_together() {
    let (mut project, goblin_ref) = project_with_goblin();
    let before = content(&project);
    let start = project.revision;
    let ops: Vec<Operation> = (0..3)
        .map(|i| set_pixels(&goblin_ref, vec![pixel(i, 0, 'N')]))
        .collect();

    let applied = project.apply_group(&ops).unwrap();

    assert_eq!(project.revision, start + 1);
    for inverse in &applied.inverses {
        project.apply(inverse).unwrap();
    }
    assert_eq!(content(&project), before);
}
