use super::*;
use crate::document::{Color, Operation, Palette, PixelChange, SpriteRef};
use crate::ipc::project::{ProjectState, create_project, edit};

fn color(key: char, hex: &str) -> Color {
    Color {
        key,
        name: key.to_string(),
        hex: hex.to_string(),
    }
}

// A project "Kit" with the palette dungeon16 and the folder "Personajes"
// holding the sprite "goblin" (4x4, 2 frames, a tag "walk"). The folder
// "Enemigos" is empty and "orc" is in the root.
fn sample() -> (tempfile::TempDir, ProjectState) {
    let parent = tempfile::tempdir().unwrap();
    let state = ProjectState::default();
    create_project(&state, parent.path(), "Kit").unwrap();
    let goblin = SpriteRef::new(&["Personajes"], "goblin");
    edit(&state, |session| {
        session
            .apply(
                "user",
                vec![
                    Operation::CreatePalette {
                        palette: Palette {
                            name: "dungeon16".to_string(),
                            colors: vec![color('N', "#000000"), color('R', "#F04545")],
                        },
                        index: None,
                    },
                    Operation::CreateFolder { parent: vec![], name: "Personajes".to_string() },
                    Operation::CreateFolder { parent: vec![], name: "Enemigos".to_string() },
                    Operation::CreateSprite {
                        folder: vec!["Personajes".to_string()],
                        name: "goblin".to_string(),
                        width: 4,
                        height: 4,
                        palette: "dungeon16".to_string(),
                    },
                    Operation::AddFrame { sprite: goblin.clone(), duration_ms: 100 },
                    Operation::AddTag {
                        sprite: goblin.clone(),
                        tag: crate::document::Tag {
                            name: "walk".to_string(),
                            from: 1,
                            to: 2,
                            fps: 12,
                            mode: crate::document::TagMode::Loop,
                        },
                        index: None,
                    },
                    Operation::CreateSprite {
                        folder: vec![],
                        name: "orc".to_string(),
                        width: 8,
                        height: 8,
                        palette: "dungeon16".to_string(),
                    },
                ],
            )
            .unwrap();
        session
            .apply(
                "claude-code",
                vec![Operation::SetPixels {
                    sprite: goblin.clone(),
                    layer: 0,
                    frame: 1,
                    pixels: vec![
                        PixelChange { x: 0, y: 0, key: 'R' },
                        PixelChange { x: 1, y: 0, key: 'N' },
                    ],
                }],
            )
            .unwrap();
    })
    .unwrap();
    (parent, state)
}

// Checks that the tree lists the nested folders and the sprites with their
// size, frames and animations, so the UI can draw cards and details.
#[test]
fn the_tree_lists_folders_and_sprites() {
    let (_parent, state) = sample();

    let tree = project_tree(&state).unwrap();

    assert_eq!(tree.name, "Kit");
    let names: Vec<&str> = tree.root.folders.iter().map(|f| f.name.as_str()).collect();
    assert_eq!(names, ["Personajes", "Enemigos"]);
    let personajes = &tree.root.folders[0];
    assert_eq!(personajes.path, ["Personajes"]);
    let goblin = &personajes.sprites[0];
    assert_eq!((goblin.name.as_str(), goblin.width, goblin.height), ("goblin", 4, 4));
    assert_eq!(goblin.frames, 2);
    assert_eq!(goblin.palette, "dungeon16");
    assert_eq!(goblin.tags[0].name, "walk");
    assert_eq!((goblin.tags[0].from, goblin.tags[0].to, goblin.tags[0].fps), (1, 2, 12));
    assert_eq!(goblin.tags[0].mode, "loop");
    assert_eq!(tree.root.sprites[0].name, "orc");
    assert_eq!(tree.palettes[0].name, "dungeon16");
    assert_eq!(tree.palettes[0].colors, 2);
}

// Checks that the tree says who edited each sprite last, taken from the
// history of the session: an AI client for the goblin, the user for the orc.
#[test]
fn the_tree_says_who_edited_a_sprite_last() {
    let (_parent, state) = sample();

    let tree = project_tree(&state).unwrap();

    let goblin = &tree.root.folders[0].sprites[0];
    let editor = goblin.last_editor.as_ref().expect("an editor");
    assert_eq!(editor.author, "claude-code");
    assert!(editor.at_ms > 0);
    assert_eq!(tree.root.sprites[0].last_editor.as_ref().unwrap().author, "user");
}

// Checks that the tree carries the revision, so the UI can tell what is new.
#[test]
fn the_tree_carries_the_revision() {
    let (_parent, state) = sample();
    let before = project_tree(&state).unwrap().revision;

    edit(&state, |s| {
        s.apply("user", vec![Operation::CreateFolder { parent: vec![], name: "Items".to_string() }])
            .unwrap()
    });

    assert_eq!(project_tree(&state).unwrap().revision, before + 1);
}

// Checks that without an open project every query fails with `no_project`.
#[test]
fn queries_fail_without_a_project() {
    let state = ProjectState::default();

    assert_eq!(project_tree(&state).unwrap_err().code, "no_project");
    assert_eq!(sprite_frame(&state, &[], "x", 1).unwrap_err().code, "no_project");
}

// Checks that a frame is sent as rows of palette keys plus the colors, which
// is what the renderer needs to draw a thumbnail.
#[test]
fn a_frame_is_rows_of_keys_and_colors() {
    let (_parent, state) = sample();

    let frame = sprite_frame(&state, &["Personajes".to_string()], "goblin", 1).unwrap();

    assert_eq!((frame.width, frame.height), (4, 4));
    assert_eq!(frame.rows, ["RN..", "....", "....", "...."]);
    assert!(frame.colors.iter().any(|c| c.key == "R" && c.hex == "#F04545"));
}

// Checks the errors of a frame: a sprite that does not exist and a frame out of range.
#[test]
fn a_missing_sprite_or_frame_is_an_error() {
    let (_parent, state) = sample();

    assert_eq!(sprite_frame(&state, &[], "nope", 1).unwrap_err().code, "not_found");
    assert_eq!(
        sprite_frame(&state, &["Personajes".to_string()], "goblin", 3).unwrap_err().code,
        "bad_frame"
    );
}

// Checks that a change reports the new revision and what it touched (the
// data of the `project://changed` event), and that errors keep their code.
#[test]
fn a_change_reports_the_revision_and_the_touched_parts() {
    let (_parent, state) = sample();
    let before = project_tree(&state).unwrap().revision;

    let (_, event) = change(&state, |session| {
        let touched = session.apply(
            "user",
            vec![Operation::CreateFolder { parent: vec![], name: "Items".to_string() }],
        )?;
        Ok(((), touched))
    })
    .unwrap();

    assert_eq!(event.revision, before + 1);
    assert_eq!(event.touched, [TargetInfo { kind: "folder", path: vec!["Items".to_string()] }]);
    let failed = change(&state, |session| {
        let touched = session.apply(
            "user",
            vec![Operation::CreateFolder { parent: vec![], name: "items".to_string() }],
        )?;
        Ok(((), touched))
    });
    assert_eq!(failed.unwrap_err().code, "name_exists");
}

// Checks the wire format of the event and the errors the UI reads.
#[test]
fn the_event_and_the_errors_serialize_with_stable_names() {
    let event = ChangedEvent {
        revision: 7,
        touched: vec![TargetInfo { kind: "sprite", path: vec!["a".into(), "b".into()] }],
    };
    assert_eq!(
        serde_json::to_string(&event).unwrap(),
        r#"{"revision":7,"touched":[{"kind":"sprite","path":["a","b"]}]}"#
    );
    assert_eq!(
        serde_json::to_string(&CoreError::no_project()).unwrap(),
        r#"{"code":"no_project","message":"no project is open"}"#
    );
}
