use super::*;
use crate::document::{Color, Operation, Palette};
use crate::ipc::project::{ProjectState, create_project, edit};

// A project "Kit" with the sprite "goblin" (4x4, 2 frames) in the folder
// "Personajes" and the empty folder "Enemigos".
fn sample() -> (tempfile::TempDir, ProjectState) {
    let parent = tempfile::tempdir().unwrap();
    let state = ProjectState::default();
    create_project(&state, parent.path(), "Kit").unwrap();
    edit(&state, |session| {
        session
            .apply(
                "user",
                vec![
                    Operation::CreatePalette {
                        palette: Palette {
                            name: "main".to_string(),
                            colors: vec![Color {
                                key: 'R',
                                name: "red".to_string(),
                                hex: "#FF0000".to_string(),
                            }],
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
                        palette: "main".to_string(),
                    },
                    Operation::AddFrame {
                        sprite: SpriteRef::new(&["Personajes"], "goblin"),
                        duration_ms: 100,
                    },
                ],
            )
            .unwrap();
    });
    (parent, state)
}

fn folder(name: &str) -> ScopeArg {
    ScopeArg::Folder { path: vec![name.to_string()] }
}

fn options() -> OptionsArg {
    OptionsArg { scale: 1, spacing: 1, naming: None }
}

// Checks the counts the tab shows for a folder and for the empty one.
#[test]
fn summary_counts_sprites_frames_and_files() {
    let (_dir, state) = sample();
    let s = summary(&state, &folder("Personajes"), &FormatArg::Sheet, &options()).unwrap();
    assert_eq!((s.sprites, s.frames), (1, 2));
    assert_eq!(s.files, ["personajes.png", "personajes.json"]);
    assert_eq!((s.sheet_width, s.sheet_height), (9, 4));
    let empty = summary(&state, &folder("Enemigos"), &FormatArg::Sheet, &options()).unwrap();
    assert_eq!((empty.sprites, empty.frames), (0, 0));
}

// Checks that the preview has the 8 byte header and the pixels after it.
#[test]
fn preview_has_a_size_header_and_pixels() {
    let (_dir, state) = sample();
    let bytes = preview(&state, &folder("Personajes"), &options()).unwrap();
    assert_eq!(u32::from_le_bytes(bytes[0..4].try_into().unwrap()), 9);
    assert_eq!(u32::from_le_bytes(bytes[4..8].try_into().unwrap()), 4);
    assert_eq!(bytes.len(), 8 + 9 * 4 * 4);
}

// Checks the export: the files are in the project folder, and the answer says
// the folder and the files.
#[test]
fn run_writes_the_files_in_the_project() {
    let (parent, state) = sample();
    let written = run(&state, &folder("Personajes"), &FormatArg::Sheet, &options(), "assets/sprites")
        .unwrap();
    assert_eq!(written.output, "assets/sprites");
    assert_eq!(written.files, ["personajes.png", "personajes.json"]);
    let out = parent.path().join("Kit").join("assets").join("sprites");
    assert!(out.join("personajes.png").is_file());
    assert!(out.join("personajes.json").is_file());
}

// Checks the PNG files format through the same path.
#[test]
fn run_writes_png_files() {
    let (parent, state) = sample();
    let written = run(&state, &folder("Personajes"), &FormatArg::Png, &options(), "out").unwrap();
    assert_eq!(written.files, ["goblin_1.png", "goblin_2.png"]);
    assert!(parent.path().join("Kit/out/goblin_2.png").is_file());
}

// Checks the spec scenario "path outside the project" through the core, and
// that an empty scope is refused with `nothing_to_export`.
#[test]
fn run_refuses_outside_paths_and_empty_scopes() {
    let (parent, state) = sample();
    let error = run(&state, &folder("Personajes"), &FormatArg::Sheet, &options(), "../../etc").unwrap_err();
    assert_eq!(error.code, "outside_project");
    assert!(!parent.path().join("etc").exists());
    let error = run(&state, &folder("Enemigos"), &FormatArg::Sheet, &options(), "out").unwrap_err();
    assert_eq!(error.code, "nothing_to_export");
}

// Checks that everything needs an open project.
#[test]
fn without_a_project_there_is_no_export() {
    let state = ProjectState::default();
    let error = summary(&state, &ScopeArg::Project, &FormatArg::Sheet, &options()).unwrap_err();
    assert_eq!(error.code, "no_project");
}
