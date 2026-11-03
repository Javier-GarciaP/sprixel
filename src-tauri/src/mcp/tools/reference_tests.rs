//! Task 4.1: `import_reference`. The PNG files are made by the tests in the
//! temp project folder.

use crate::ipc::project;
use crate::mcp::testing::{TestHost, call, open_demo_project, revision, rows, setup};
use serde_json::json;
use std::path::PathBuf;

fn root(host: &TestHost) -> PathBuf {
    project::read(&host.project, |s| s.root().to_path_buf()).unwrap()
}

/// Saves a picture as `references/<name>`; `pixel(x, y)` gives its RGBA.
fn png(host: &TestHost, name: &str, w: u32, h: u32, pixel: impl Fn(u32, u32) -> [u8; 4]) -> PathBuf {
    let folder = root(host).join("references");
    std::fs::create_dir_all(&folder).unwrap();
    let image = image::RgbaImage::from_fn(w, h, |x, y| image::Rgba(pixel(x, y)));
    let path = folder.join(name);
    image.save(&path).unwrap();
    path
}

const RED: [u8; 4] = [255, 0, 0, 255];
const GREEN: [u8; 4] = [0, 255, 0, 255];

fn import(core: &crate::mcp::Core, session: &str, extra: serde_json::Value) -> Result<serde_json::Value, serde_json::Value> {
    let mut args = json!({ "name": "slime" });
    args.as_object_mut().unwrap().extend(extra.as_object().unwrap().clone());
    call(core, session, "import_reference", args)
}

// Checks the "Import" and "Nearest color" scenarios: palette colors give their
// keys, a color that is close to red gives R (and adds nothing), and a pixel
// with alpha under 128 is transparent.
#[test]
fn a_picture_is_written_with_the_palette_keys() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    png(&host, "a.png", 4, 4, |x, _| match x {
        0 => RED,
        1 => [200, 30, 20, 255],
        2 => GREEN,
        _ => [10, 10, 10, 127],
    });

    let done = import(&core, &session, json!({ "file": "a.png" })).unwrap();

    assert_eq!(done["colors_added"], 0);
    assert_eq!(rows(&host, &[], "slime", 1), "RRG...../RRG...../RRG...../RRG...../......../......../......../........");
}

// Checks the "Add colors" scenario: with `max_colors` 1 the most used new
// color joins the palette in the same history entry (one undo removes both);
// without it the nearest palette color is used.
#[test]
fn max_colors_adds_the_most_used_new_colors() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    let blue = [0, 0, 255, 255];
    let purple = [120, 0, 120, 255];
    png(&host, "b.png", 2, 2, |x, y| if x == 0 && y == 0 { purple } else { blue });

    let none = import(&core, &session, json!({ "file": "b.png" })).unwrap();
    assert_eq!(none["colors_added"], 0);
    let entries = project::read(&host.project, |s| s.history.entries().len()).unwrap();

    let done = import(&core, &session, json!({ "file": "b.png", "max_colors": 1 })).unwrap();

    assert_eq!(done["colors_added"], 1);
    let keys: String = project::read(&host.project, |s| s.project.palette("main").unwrap().colors.iter().map(|c| c.key).collect()).unwrap();
    assert_eq!(keys, "RGA");
    let hex = project::read(&host.project, |s| s.project.palette("main").unwrap().colors[2].hex.clone()).unwrap();
    assert_eq!(hex, "#0000FF", "the blue is the most used color");
    assert!(rows(&host, &[], "slime", 1).starts_with("RA....../AA......"), "purple has no new color, it takes the first nearest one");
    let after = project::read(&host.project, |s| s.history.entries().len()).unwrap();
    assert_eq!(after, entries + 1, "colors and pixels are one entry");
    project::edit(&host.project, |s| s.undo().unwrap()).unwrap();
    let keys: String = project::read(&host.project, |s| s.project.palette("main").unwrap().colors.iter().map(|c| c.key).collect()).unwrap();
    assert_eq!(keys, "RG");
}

// Checks the "Scale down" scenario: an 8x8 picture with a red top left
// quarter becomes 4x4 with `width` and `height`, at the place `x`, `y`; and
// giving only `width` keeps the proportion.
#[test]
fn a_picture_can_be_scaled_down_and_placed() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    png(&host, "big.png", 8, 8, |x, y| if x < 4 && y < 4 { RED } else { GREEN });

    let done = import(&core, &session, json!({ "file": "big.png", "width": 4, "height": 4, "x": 2, "y": 1 })).unwrap();

    assert_eq!((done["width"].as_u64(), done["height"].as_u64()), (Some(4), Some(4)));
    assert_eq!(rows(&host, &[], "slime", 1), "......../..RRGG../..RRGG../..GGGG../..GGGG../......../......../........");

    png(&host, "wide.png", 8, 4, |_, _| RED);
    let wide = import(&core, &session, json!({ "file": "wide.png", "width": 4 })).unwrap();
    assert_eq!((wide["width"].as_u64(), wide["height"].as_u64()), (Some(4), Some(2)));
}

// Checks that a picture that does not fit in the sprite is refused with
// `region_out` and nothing changes.
#[test]
fn a_picture_that_does_not_fit_is_refused() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    png(&host, "big.png", 9, 2, |_, _| RED);
    let before = revision(&host);

    let error = import(&core, &session, json!({ "file": "big.png" })).unwrap_err();

    assert_eq!(error["code"], "region_out");
    assert_eq!(revision(&host), before);
}

// Checks the "Not allowed path" scenario: `..`, absolute paths, other kinds
// of files, a file that is too big, a picture that is too big, a missing file
// and a missing folder. Each gives a code, and nothing changes.
#[test]
fn files_outside_the_rules_are_refused() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    png(&host, "ok.png", 2, 2, |_, _| RED);
    let outside = root(&host).join("secret.png");
    image::RgbaImage::new(2, 2).save(&outside).unwrap();
    let references = root(&host).join("references");
    std::fs::write(references.join("note.txt"), b"hello").unwrap();
    std::fs::write(references.join("fake.png"), b"this is not a png").unwrap();
    std::fs::write(references.join("huge.png"), vec![0u8; 2 * 1024 * 1024 + 1]).unwrap();
    png(&host, "wide.png", 513, 1, |_, _| RED);
    let before = revision(&host);
    let code = |file: &str| import(&core, &session, json!({ "file": file })).unwrap_err()["code"].as_str().unwrap().to_string();

    assert_eq!(code("../secret.png"), "invalid_path");
    assert_eq!(code("sub/../../secret.png"), "invalid_path");
    assert_eq!(code(outside.to_str().unwrap()), "invalid_path");
    assert_eq!(code("/etc/passwd"), "invalid_path");
    assert_eq!(code("note.txt"), "invalid_path");
    assert_eq!(code("fake.png"), "bad_image");
    assert_eq!(code("huge.png"), "too_big");
    assert_eq!(code("wide.png"), "too_big");
    assert_eq!(code("missing.png"), "not_found");
    assert_eq!(revision(&host), before);

    std::fs::remove_dir_all(&references).unwrap();
    assert_eq!(code("ok.png"), "not_found", "no references folder");
}

// Checks that a symbolic link is never followed, not even to a real PNG
// outside the folder, and not when the whole folder is a link.
#[cfg(unix)]
#[test]
fn symbolic_links_are_refused() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    let outside = root(&host).join("secret.png");
    image::RgbaImage::new(2, 2).save(&outside).unwrap();
    png(&host, "ok.png", 2, 2, |_, _| RED);
    std::os::unix::fs::symlink(&outside, root(&host).join("references").join("link.png")).unwrap();
    let before = revision(&host);

    let error = import(&core, &session, json!({ "file": "link.png" })).unwrap_err();
    assert_eq!(error["code"], "invalid_path");

    let elsewhere = tempfile::tempdir().unwrap();
    image::RgbaImage::new(2, 2).save(elsewhere.path().join("x.png")).unwrap();
    std::fs::remove_dir_all(root(&host).join("references")).unwrap();
    std::os::unix::fs::symlink(elsewhere.path(), root(&host).join("references")).unwrap();
    let folder_link = import(&core, &session, json!({ "file": "x.png" })).unwrap_err();
    assert_eq!(folder_link["code"], "invalid_path");
    assert_eq!(revision(&host), before);
}

// Checks that the call is a normal write: one entry, a replay event for the UI
// and the permission "Edit pixels" is needed.
#[test]
fn the_import_is_a_normal_write() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    png(&host, "a.png", 2, 2, |_, _| RED);

    let done = import(&core, &session, json!({ "file": "a.png" })).unwrap();

    assert_eq!(done["changed"], true);
    assert_eq!(done["sprite"], "slime");
    let events = host.events.lock().unwrap();
    let replay = events.iter().rev().find(|(e, _)| e == super::write::OPS_EVENT).expect("a replay event");
    assert_eq!(replay.1["tool"], "import_reference");
    drop(events);
}
