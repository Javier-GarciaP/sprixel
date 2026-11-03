//! Task 3.5: the `export` tool. Files stay inside the project, and the
//! export permission is respected.

use crate::ipc::project;
use crate::mcp::Permissions;
use crate::mcp::testing::{TestHost, call, open_demo_project, revision, setup};
use serde_json::json;
use std::path::PathBuf;

fn root(host: &TestHost) -> PathBuf {
    project::read(&host.project, |s| s.root().to_path_buf()).unwrap()
}

/// Every file in a folder, recursively, as sorted relative paths.
fn tree(folder: &std::path::Path) -> Vec<String> {
    fn walk(base: &std::path::Path, folder: &std::path::Path, out: &mut Vec<String>) {
        let Ok(entries) = std::fs::read_dir(folder) else { return };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(base, &path, out);
            } else {
                out.push(path.strip_prefix(base).unwrap().to_string_lossy().replace('\\', "/"));
            }
        }
    }
    let mut out = Vec::new();
    walk(folder, folder, &mut out);
    out.sort();
    out
}

// Checks the "Export a folder" scenario: the folder "Characters" as a sheet
// with JSON. The files are written inside the project, and the call returns
// their paths from the project folder. The project itself does not change.
#[test]
fn export_a_folder_as_a_sheet_writes_files_inside_the_project() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    let before = revision(&host);

    let done = call(&core, &session, "export", json!({ "scope": "folder", "folder": "Characters", "format": "sheet" })).unwrap();

    let files: Vec<String> = done["files"].as_array().unwrap().iter().map(|f| f.as_str().unwrap().to_string()).collect();
    assert_eq!(done["output"], "assets/sprites");
    assert_eq!(files.len(), 2);
    assert!(files.iter().any(|f| f.ends_with(".png")) && files.iter().any(|f| f.ends_with(".json")), "{files:?}");
    for file in &files {
        assert!(file.starts_with("assets/sprites/"), "{file}");
        assert!(root(&host).join(file).is_file(), "{file} exists inside the project");
    }
    let json_file = files.iter().find(|f| f.ends_with(".json")).unwrap();
    let text = std::fs::read_to_string(root(&host).join(json_file)).unwrap();
    assert!(text.contains("goblin"), "the JSON describes the sprite");
    assert!(!text.contains("slime"), "only the folder was exported");
    assert_eq!(revision(&host), before);
}

// Checks the PNG files format, the scope of sprites, a scale and a custom
// output folder.
#[test]
fn export_png_files_of_some_sprites_to_another_folder() {
    let (host, core, session) = setup();
    open_demo_project(&host);

    let done = call(&core, &session, "export", json!({
        "scope": "sprites", "sprites": ["Characters/goblin"], "format": "png", "scale": 2, "output": "out/art",
    })).unwrap();

    assert_eq!(done["output"], "out/art");
    let written = tree(&root(&host).join("out/art"));
    assert_eq!(written.len(), 2, "one PNG for each of the 2 frames: {written:?}");
    assert!(written.iter().all(|f| f.ends_with(".png") && f.contains("goblin")));
    let image = image::open(root(&host).join("out/art").join(&written[0])).unwrap();
    assert_eq!((image.width(), image.height()), (8, 8), "4x4 at scale 2");
}

// Checks the "Turn export off" scenario for this tool: with the permission
// off the call fails with `permission_denied` and no file is written.
#[test]
fn export_needs_the_export_permission() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    core.set_permissions(Permissions { export: false, ..core.permissions() }).unwrap();

    let error = call(&core, &session, "export", json!({})).unwrap_err();

    assert_eq!(error["code"], "permission_denied");
    assert_eq!(error["permission"], "export");
    assert!(!root(&host).join("assets").exists());
}

// Checks the main security rule: the output folder can never leave the
// project. `..`, absolute paths, drives, the reserved folder and a link that
// points outside are all refused, and nothing is written outside.
#[test]
fn export_cannot_write_outside_the_project() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    let outside = tempfile::tempdir().unwrap();
    let outside_path = outside.path().to_string_lossy().into_owned();
    let before_parent = tree(host.dir.path());

    for output in [
        "../escaped",
        "a/../../escaped",
        "..",
        &outside_path,
        &format!("{outside_path}/sub"),
        "/etc/sprixel-test",
        "\\\\server\\share",
        "C:\\Windows\\Temp",
        "D:escaped",
    ] {
        let error = call(&core, &session, "export", json!({ "output": output })).unwrap_err();
        assert!(["outside_project", "invalid_path"].contains(&error["code"].as_str().unwrap()), "{output}: {error}");
    }
    let reserved = call(&core, &session, "export", json!({ "output": ".sprixel/trash" })).unwrap_err();
    assert_eq!(reserved["code"], "invalid_path");
    assert_eq!(call(&core, &session, "export", json!({ "output": "" })).unwrap_err()["code"], "invalid_path");

    assert!(tree(outside.path()).is_empty(), "nothing was written outside");
    assert_eq!(tree(host.dir.path()), before_parent, "nothing new next to the project");
}

// Checks that a symbolic link inside the project that points outside cannot
// be used to write outside (Linux and macOS).
#[cfg(unix)]
#[test]
fn export_cannot_follow_a_link_out_of_the_project() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    let outside = tempfile::tempdir().unwrap();
    std::os::unix::fs::symlink(outside.path(), root(&host).join("link")).unwrap();

    for output in ["link", "link/deeper"] {
        let error = call(&core, &session, "export", json!({ "output": output })).unwrap_err();
        assert_eq!(error["code"], "outside_project", "{output}");
    }
    assert!(tree(outside.path()).is_empty());
}

// Checks the other errors: bad options, a scope with nothing to export, a
// folder that does not exist, a missing argument and an unknown sprite.
#[test]
fn export_errors_are_clear() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    call(&core, &session, "create_folder", json!({ "name": "Empty" })).unwrap();
    let run = |args| call(&core, &session, "export", args).unwrap_err()["code"].as_str().unwrap().to_string();

    assert_eq!(run(json!({ "scale": 3 })), "bad_options");
    assert_eq!(run(json!({ "spacing": 99 })), "bad_options");
    assert_eq!(run(json!({ "naming": "../x" })), "bad_options");
    assert_eq!(run(json!({ "naming": "a/b" })), "bad_options");
    assert_eq!(run(json!({ "scope": "folder", "folder": "Empty" })), "nothing_to_export");
    assert_eq!(run(json!({ "scope": "folder", "folder": "Nope" })), "not_found");
    assert_eq!(run(json!({ "scope": "folder" })), "bad_argument");
    assert_eq!(run(json!({ "scope": "sprites", "sprites": ["ghost"] })), "not_found");
    assert_eq!(run(json!({ "scope": "sprites" })), "bad_argument");
    assert_eq!(run(json!({ "scope": "everything" })), "bad_argument");
    assert_eq!(run(json!({ "format": "gif" })), "bad_argument");
    assert!(!root(&host).join("assets").exists(), "a failed export leaves no folder behind");
}
