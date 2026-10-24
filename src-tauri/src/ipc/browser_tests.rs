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

fn names(folder: &crate::document::Folder) -> Vec<&str> {
    folder.sprites.iter().map(|s| s.name.as_str()).collect()
}

fn session_sprite(state: &ProjectState, folder: &[&str], name: &str) -> Option<crate::document::Sprite> {
    let at = SpriteRef::new(folder, name);
    crate::ipc::project::read(state, |s| s.project.find_sprite(&at).cloned()).unwrap()
}

// Checks the spec scenario "Custom size": 120x30 makes a sprite of that size
// in the chosen folder, and the answer says where it is so the UI can open it.
#[test]
fn a_sprite_can_have_any_size_up_to_512() {
    let (_parent, state) = sample();

    let (target, event) = create_sprite(
        &state,
        &["Enemigos".to_string()],
        "slime",
        120,
        30,
        Some("dungeon16"),
    )
    .unwrap();

    assert_eq!((target.folder, target.name.as_str()), (vec!["Enemigos".to_string()], "slime"));
    let sprite = session_sprite(&state, &["Enemigos"], "slime").unwrap();
    assert_eq!((sprite.width, sprite.height, sprite.frame_count()), (120, 30, 1));
    assert_eq!(sprite.palette, "dungeon16");
    assert_eq!(event.touched[0].kind, "sprite");
}

// Checks the spec scenario "No palette chosen": a project without palettes
// gets an empty palette `main` and the sprite uses it.
#[test]
fn without_a_palette_the_sprite_uses_a_new_main() {
    let parent = tempfile::tempdir().unwrap();
    let state = ProjectState::default();
    create_project(&state, parent.path(), "Kit").unwrap();

    create_sprite(&state, &[], "goblin", 16, 16, None).unwrap();

    let tree = project_tree(&state).unwrap();
    assert_eq!(tree.palettes.len(), 1);
    assert_eq!(tree.palettes[0].name, "main");
    assert_eq!(tree.root.sprites[0].palette, "main");
}

// Checks that creating the sprite and its `main` palette is one history entry:
// one undo removes both, and the palette is not left behind.
#[test]
fn creating_a_sprite_with_main_is_one_undo() {
    let parent = tempfile::tempdir().unwrap();
    let state = ProjectState::default();
    create_project(&state, parent.path(), "Kit").unwrap();
    create_sprite(&state, &[], "goblin", 16, 16, None).unwrap();

    crate::ipc::project::edit(&state, |s| s.undo()).unwrap().unwrap();

    let tree = project_tree(&state).unwrap();
    assert!(tree.root.sprites.is_empty());
    assert!(tree.palettes.is_empty());
}

// Checks that an existing `main` is reused, not created twice.
#[test]
fn an_existing_main_is_reused() {
    let parent = tempfile::tempdir().unwrap();
    let state = ProjectState::default();
    create_project(&state, parent.path(), "Kit").unwrap();
    create_sprite(&state, &[], "a", 8, 8, None).unwrap();

    create_sprite(&state, &[], "b", 8, 8, None).unwrap();

    assert_eq!(project_tree(&state).unwrap().palettes.len(), 1);
}

// Checks the errors of creating a sprite: a size above 512, a name already
// used in the folder and a palette that does not exist. A refused creation
// leaves nothing behind (not even the `main` palette).
#[test]
fn a_refused_sprite_changes_nothing() {
    let parent = tempfile::tempdir().unwrap();
    let state = ProjectState::default();
    create_project(&state, parent.path(), "Kit").unwrap();

    let too_big = create_sprite(&state, &[], "big", 513, 8, None).unwrap_err();
    assert_eq!(too_big.code, "too_big");
    assert!(project_tree(&state).unwrap().palettes.is_empty());

    create_sprite(&state, &[], "goblin", 8, 8, None).unwrap();
    let again = create_sprite(&state, &[], "GOBLIN", 8, 8, None).unwrap_err();
    assert_eq!(again.code, "name_exists");

    let missing = create_sprite(&state, &[], "x", 8, 8, Some("nope")).unwrap_err();
    assert_eq!(missing.code, "palette_not_found");
    assert_eq!(project_tree(&state).unwrap().root.sprites.len(), 1);
}

// Checks the name check the dialogs use: empty, a character that is not
// allowed, and a name that a folder or a sprite already uses (ignoring case).
#[test]
fn check_name_gives_a_stable_code() {
    let (_parent, state) = sample();
    let check = |parent: &[&str], name: &str| {
        let parent: Vec<String> = parent.iter().map(|p| p.to_string()).collect();
        check_name(&state, &parent, name, None).unwrap()
    };

    assert_eq!(check(&[], ""), Some("empty"));
    assert_eq!(check(&[], "  "), Some("empty"));
    assert_eq!(check(&[], "a/b"), Some("invalid_name"));
    assert_eq!(check(&[], "enemigos"), Some("folder_exists"));
    assert_eq!(check(&[], "ORC"), Some("sprite_exists"));
    assert_eq!(check(&["Personajes"], "orc"), None);
    assert_eq!(check(&[], "Items"), None);
}

// Checks that the item being renamed does not clash with itself (so a
// rename that only changes the case is allowed), but other names still do.
#[test]
fn check_name_ignores_the_item_being_renamed() {
    let (_parent, state) = sample();
    let check = |name: &str, current: &str| {
        check_name(&state, &[], name, Some(current)).unwrap()
    };

    assert_eq!(check("Orc", "orc"), None);
    assert_eq!(check("enemigos", "orc"), Some("folder_exists"));
}

// Checks the spec scenario "Create a folder": the folder appears in the tree
// and in the project's `sprites/` folder once saved; the same name twice is refused.
#[test]
fn a_folder_is_created_in_the_root_or_inside_another() {
    let (parent, state) = sample();

    let (path, _) = create_folder(&state, &[], "Items").unwrap();
    create_folder(&state, &["Items".to_string()], "Potions").unwrap();

    assert_eq!(path, ["Items"]);
    let tree = project_tree(&state).unwrap();
    assert_eq!(tree.root.folders[2].folders[0].path, ["Items", "Potions"]);
    crate::ipc::project::close_project(&state);
    assert!(parent.path().join("Kit/sprites/Items/Potions").is_dir());
}

// Checks that a folder name already used is refused with `name_exists`.
#[test]
fn a_folder_name_in_use_is_refused() {
    let (_parent, state) = sample();

    let error = create_folder(&state, &[], "enemigos").unwrap_err();

    assert_eq!(error.code, "name_exists");
    assert_eq!(names(&crate::ipc::project::read(&state, |s| s.project.root.clone()).unwrap()), ["orc"]);
}

// Checks the spec scenario "Rename with F2": the sprite shows with its new
// name and, once saved, its file has the new name. History and content stay.
#[test]
fn a_renamed_sprite_keeps_its_content_and_changes_its_file() {
    let (parent, state) = sample();
    crate::ipc::project::close_project(&state);
    let again = crate::ipc::project::open_project(&state, &parent.path().join("Kit")).unwrap();
    assert_eq!(again.name, "Kit");

    let (target, event) =
        rename_sprite(&state, &["Personajes".to_string()], "goblin", "orc2").unwrap();

    assert_eq!(target.name, "orc2");
    assert_eq!(event.touched.len(), 2);
    let sprite = session_sprite(&state, &["Personajes"], "orc2").unwrap();
    assert_eq!(sprite.frame_count(), 2);
    assert!(session_sprite(&state, &["Personajes"], "goblin").is_none());
    crate::ipc::project::close_project(&state);
    let folder = parent.path().join("Kit/sprites/Personajes");
    assert!(folder.join("orc2.sprite.json").is_file());
    assert!(!folder.join("goblin.sprite.json").exists());
}

// Checks that renaming to a name in use is refused with `name_exists` and nothing changes.
#[test]
fn a_rename_to_a_used_name_is_refused() {
    let (_parent, state) = sample();
    let before = project_tree(&state).unwrap().revision;

    let error = rename_sprite(&state, &[], "orc", "ENEMIGOS").unwrap_err();

    assert_eq!(error.code, "name_exists");
    assert_eq!(project_tree(&state).unwrap().revision, before);
}

// Checks that a folder is renamed with its sprites, and that the new path is returned.
#[test]
fn a_folder_is_renamed_with_its_sprites() {
    let (_parent, state) = sample();

    let (path, _) = rename_folder(&state, &["Personajes".to_string()], "Heroes").unwrap();

    assert_eq!(path, ["Heroes"]);
    let tree = project_tree(&state).unwrap();
    assert_eq!(tree.root.folders[0].path, ["Heroes"]);
    assert_eq!(tree.root.folders[0].sprites[0].folder, ["Heroes"]);
}

// Checks the spec scenario "Duplicate twice": "goblin copy" and "goblin copy
// 2" exist with the same content, each duplicate in the same folder.
#[test]
fn duplicating_twice_gives_copy_and_copy_2() {
    let (_parent, state) = sample();
    let folder = vec!["Personajes".to_string()];

    let (first, _) = duplicate_sprite(&state, &folder, "goblin").unwrap();
    let (second, _) = duplicate_sprite(&state, &folder, "goblin").unwrap();

    assert_eq!((first.name.as_str(), second.name.as_str()), ("goblin copy", "goblin copy 2"));
    let original = session_sprite(&state, &["Personajes"], "goblin").unwrap();
    for name in ["goblin copy", "goblin copy 2"] {
        let copy = session_sprite(&state, &["Personajes"], name).unwrap();
        assert_eq!(crate::document::Sprite { name: "goblin".to_string(), ..copy }, original);
    }
}

// Checks that a folder with the name "x copy" also counts as taken for a duplicate.
#[test]
fn a_duplicate_name_avoids_folders_too() {
    let (_parent, state) = sample();
    create_folder(&state, &[], "orc copy").unwrap();

    let (copy, _) = duplicate_sprite(&state, &[], "orc").unwrap();

    assert_eq!(copy.name, "orc copy 2");
}

// Checks that undo takes back a rename (the command and the history agree).
#[test]
fn undo_takes_back_a_rename() {
    let (_parent, state) = sample();
    rename_sprite(&state, &[], "orc", "ogre").unwrap();

    crate::ipc::project::edit(&state, |s| s.undo()).unwrap().unwrap();

    assert!(session_sprite(&state, &[], "orc").is_some());
    assert!(session_sprite(&state, &[], "ogre").is_none());
}

fn trash_has_a_sprite_file(root: &std::path::Path) -> bool {
    fn walk(dir: &std::path::Path) -> bool {
        let Ok(entries) = std::fs::read_dir(dir) else { return false };
        entries.flatten().any(|e| {
            let path = e.path();
            if path.is_dir() {
                walk(&path)
            } else {
                path.to_string_lossy().ends_with(".sprite.json")
            }
        })
    }
    walk(&root.join(".sprixel/trash"))
}

// Checks the spec scenario "Delete and undo": the sprite goes to the trash
// (its file is kept in `.sprixel/trash/`, not erased), and a project opened
// again does not have it.
#[test]
fn a_deleted_sprite_goes_to_the_trash() {
    let (parent, state) = sample();
    let root = parent.path().join("Kit");
    crate::ipc::project::edit(&state, |s| s.flush()).unwrap();
    assert!(root.join("sprites/Personajes/goblin.sprite.json").is_file());
    let target = SpriteTarget { folder: vec!["Personajes".to_string()], name: "goblin".to_string() };

    let (revision, event) = delete_sprites(&state, &[target]).unwrap();

    assert_eq!(revision, event.revision);
    assert!(session_sprite(&state, &["Personajes"], "goblin").is_none());
    crate::ipc::project::close_project(&state);
    assert!(!root.join("sprites/Personajes/goblin.sprite.json").exists());
    assert!(trash_has_a_sprite_file(&root));
    crate::ipc::project::open_project(&state, &root).unwrap();
    assert!(session_sprite(&state, &["Personajes"], "goblin").is_none());
}

// Checks that undo restores the content of the deleted sprite, and that redo deletes it again.
#[test]
fn undo_and_redo_bring_back_and_remove_the_sprite() {
    let (_parent, state) = sample();
    let before = session_sprite(&state, &["Personajes"], "goblin").unwrap();
    let target = SpriteTarget { folder: vec!["Personajes".to_string()], name: "goblin".to_string() };
    delete_sprites(&state, &[target]).unwrap();

    let (_, event) = undo(&state).unwrap();
    assert_eq!(event.touched[0].path, ["Personajes", "goblin"]);
    assert_eq!(session_sprite(&state, &["Personajes"], "goblin"), Some(before));

    redo(&state).unwrap();
    assert!(session_sprite(&state, &["Personajes"], "goblin").is_none());
}

// Checks that undo after saving moves the file out of the trash to its place.
#[test]
fn undo_after_the_save_moves_the_file_back_from_the_trash() {
    let (parent, state) = sample();
    let root = parent.path().join("Kit");
    let target = SpriteTarget { folder: vec!["Personajes".to_string()], name: "goblin".to_string() };
    crate::ipc::project::edit(&state, |s| s.flush()).unwrap();
    delete_sprites(&state, &[target]).unwrap();
    crate::ipc::project::edit(&state, |s| s.flush()).unwrap();
    assert!(!root.join("sprites/Personajes/goblin.sprite.json").exists());

    undo(&state).unwrap();
    crate::ipc::project::edit(&state, |s| s.flush()).unwrap();

    assert!(root.join("sprites/Personajes/goblin.sprite.json").is_file());
    assert!(!trash_has_a_sprite_file(&root));
}

// Checks that several sprites are deleted as one step: one undo brings all back.
#[test]
fn deleting_several_sprites_is_one_undo() {
    let (_parent, state) = sample();
    let targets = [
        SpriteTarget { folder: vec!["Personajes".to_string()], name: "goblin".to_string() },
        SpriteTarget { folder: vec![], name: "orc".to_string() },
    ];

    delete_sprites(&state, &targets).unwrap();
    assert!(session_sprite(&state, &[], "orc").is_none());
    undo(&state).unwrap();

    assert!(session_sprite(&state, &[], "orc").is_some());
    assert!(session_sprite(&state, &["Personajes"], "goblin").is_some());
}

// Checks that a delete of a sprite that does not exist is refused and the
// sprites before it in the same call stay (all or none).
#[test]
fn a_refused_delete_changes_nothing() {
    let (_parent, state) = sample();
    let targets = [
        SpriteTarget { folder: vec![], name: "orc".to_string() },
        SpriteTarget { folder: vec![], name: "ghost".to_string() },
    ];

    let error = delete_sprites(&state, &targets).unwrap_err();

    assert_eq!(error.code, "not_found");
    assert!(session_sprite(&state, &[], "orc").is_some());
}

// Checks that undo and redo with nothing to do answer with a stable code.
#[test]
fn undo_and_redo_with_nothing_to_do_have_codes() {
    let parent = tempfile::tempdir().unwrap();
    let state = ProjectState::default();
    create_project(&state, parent.path(), "Kit").unwrap();

    assert_eq!(undo(&state).unwrap_err().code, "nothing_to_undo");
    assert_eq!(redo(&state).unwrap_err().code, "nothing_to_redo");
}

// Checks the spec scenario "Move to a folder": the sprite is in the new
// folder (and in the tree) and no longer in the old one.
#[test]
fn a_sprite_is_moved_to_a_folder() {
    let (_parent, state) = sample();
    let orc = SpriteTarget { folder: vec![], name: "orc".to_string() };

    let (_, event) = move_sprites(&state, &[orc], &["Enemigos".to_string()]).unwrap();

    assert!(session_sprite(&state, &["Enemigos"], "orc").is_some());
    assert!(session_sprite(&state, &[], "orc").is_none());
    assert_eq!(event.touched.len(), 2);
}

// Checks that several sprites move as one step (one undo), and that a name that
// exists in the destination refuses the whole move with `name_exists`.
#[test]
fn several_sprites_move_together_or_not_at_all() {
    let (_parent, state) = sample();
    create_sprite(&state, &["Enemigos".to_string()], "goblin", 4, 4, None).unwrap();
    let both = [
        SpriteTarget { folder: vec![], name: "orc".to_string() },
        SpriteTarget { folder: vec!["Personajes".to_string()], name: "goblin".to_string() },
    ];

    let error = move_sprites(&state, &both, &["Enemigos".to_string()]).unwrap_err();

    assert_eq!(error.code, "name_exists");
    assert!(session_sprite(&state, &[], "orc").is_some());

    let one = [both[0].clone()];
    let two = [SpriteTarget { folder: vec!["Personajes".to_string()], name: "hero".to_string() }];
    create_sprite(&state, &["Personajes".to_string()], "hero", 4, 4, None).unwrap();
    move_sprites(&state, &one, &["Enemigos".to_string()]).unwrap();
    move_sprites(&state, &two, &["Enemigos".to_string()]).unwrap();
    undo(&state).unwrap();
    assert!(session_sprite(&state, &["Personajes"], "hero").is_some());
    assert!(session_sprite(&state, &["Enemigos"], "orc").is_some());
}
