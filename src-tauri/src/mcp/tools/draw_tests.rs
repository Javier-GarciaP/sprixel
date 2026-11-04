//! Task 3.3: `apply_grid`, `set_pixels` and `draw_shape` as atomic actions.

use crate::ipc::project;
use crate::mcp::testing::{call, open_demo_project, revision, rows, setup};
use serde_json::{Value, json};

fn entries(host: &crate::mcp::testing::TestHost) -> usize {
    project::read(&host.project, |s| s.history.entries().len()).unwrap()
}

fn palette_keys(host: &crate::mcp::testing::TestHost) -> String {
    project::read(&host.project, |s| s.project.palette("main").unwrap().colors.iter().map(|c| c.key).collect()).unwrap()
}

const BLANK8: &str = "......../......../......../......../......../......../......../........";

/// An 8x8 grid for `slime`: a red square in the corner and a green dot.
const GRID8: &str = "frame 1\nRRRR....\nRRRR....\nRRRR....\nRRRR....\n........\n........\n......G.\n........\n";

// Checks the "Apply a grid" scenario: a valid 8x8 grid changes the frame,
// the call returns the new revision, it is ONE history entry by the client,
// and the UI is told.
#[test]
fn apply_grid_changes_the_frame_and_returns_the_revision() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    let before = revision(&host);

    let done = call(&core, &session, "apply_grid", json!({ "name": "slime", "grid": GRID8 })).unwrap();

    assert_eq!(rows(&host, &[], "slime", 1), "RRRR..../RRRR..../RRRR..../RRRR..../......../......../......G./........");
    assert_eq!(done["revision"], revision(&host));
    assert_eq!(revision(&host), before + 1);
    assert_eq!(done["pixels_changed"], 17);
    assert_eq!(entries(&host), 1);
    let author = project::read(&host.project, |s| s.history.entries()[0].author.clone()).unwrap();
    assert_eq!(author, "Claude Code");
    assert!(host.events.lock().unwrap().iter().any(|(e, _)| e == crate::ipc::browser::CHANGED_EVENT));
}

// Checks the "Apply a grid with a new color" scenario: the key Q is not in the
// palette, the legend gives it a color, and the frame and the palette change
// together, in one step; one undo removes both.
#[test]
fn apply_grid_with_a_legend_adds_the_color_in_the_same_step() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    let grid = format!("legend\n  Q teal #1FB5A8\n\n{}", GRID8.replace("RRRR", "QQQQ"));

    let done = call(&core, &session, "apply_grid", json!({ "name": "slime", "grid": grid })).unwrap();

    assert_eq!(done["colors_added"], 1);
    assert_eq!(palette_keys(&host), "RGQ");
    assert!(rows(&host, &[], "slime", 1).starts_with("QQQQ"));
    assert_eq!(entries(&host), 1);
    project::edit(&host.project, |s| s.undo().unwrap()).unwrap();
    assert_eq!(palette_keys(&host), "RG");
    assert_eq!(rows(&host, &[], "slime", 1), BLANK8);
}

// Checks the "Invalid grid" scenario: a row of the wrong length gives the
// error `row_length` with the line, every problem is listed, and nothing
// changes: not the pixels, not the palette, not the revision.
#[test]
fn an_invalid_grid_changes_nothing_and_says_where() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    let before = revision(&host);
    let grid = "legend\n  Q teal #1FB5A8\nframe 1\nRRRR....\nRRRR...\nRRRR....\nRRRR....\n........\n........\n......G.\n........\n";

    let error = call(&core, &session, "apply_grid", json!({ "name": "slime", "grid": grid })).unwrap_err();

    assert_eq!(error["code"], "row_length");
    assert_eq!(error["line"], 5);
    assert_eq!(error["row"], 2);
    assert!(error["message"].as_str().unwrap().contains("7 columns, expected 8"), "{error}");
    assert_eq!(rows(&host, &[], "slime", 1), BLANK8);
    assert_eq!(palette_keys(&host), "RG", "the legend color is not added either");
    assert_eq!(revision(&host), before);
    assert_eq!(entries(&host), 0);
}

// Checks that all the errors come together so one try can fix them all, and
// that an unknown key lists the keys that are valid.
#[test]
fn every_error_is_listed_and_unknown_keys_list_the_valid_keys() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    let grid = "frame 1\nRRRX....\nRRRR...\nRRRR....\nRRRR....\n........\n........\n........\n........\n";

    let error = call(&core, &session, "apply_grid", json!({ "name": "slime", "grid": grid })).unwrap_err();

    let codes: Vec<&str> = error["errors"].as_array().unwrap().iter().map(|e| e["code"].as_str().unwrap()).collect();
    assert!(codes.contains(&"unknown_key") && codes.contains(&"row_length"), "{codes:?}");
    assert_eq!(error["valid_keys"], json!(["R", "G", "."]));
}

// Checks writing a region: it can be smaller, `.` erases only with
// `skip_transparent` false (in a region it keeps by default, see erase_tests), and a region that does not fit is `region_out`.
#[test]
fn apply_grid_writes_regions() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    call(&core, &session, "apply_grid", json!({ "name": "slime", "grid": GRID8 })).unwrap();

    call(&core, &session, "apply_grid", json!({ "name": "slime", "x": 2, "y": 2, "skip_transparent": false, "grid": "frame 1\nG.\n.G\n" })).unwrap();
    let now = rows(&host, &[], "slime", 1);
    let all: Vec<&str> = now.split('/').collect();
    assert_eq!((all[2], all[3]), ("RRG.....", "RR.G...."), "`.` erased the pixels under it");
    call(&core, &session, "apply_grid", json!({ "name": "slime", "x": 0, "y": 0, "skip_transparent": true, "grid": "frame 1\n.G\n" })).unwrap();
    assert!(rows(&host, &[], "slime", 1).starts_with("RG"), "{}", rows(&host, &[], "slime", 1));
    call(&core, &session, "apply_grid", json!({ "name": "slime", "x": 0, "y": 0, "skip_transparent": false, "grid": "frame 1\n..\n" })).unwrap();
    assert!(rows(&host, &[], "slime", 1).starts_with(".."));

    let out = call(&core, &session, "apply_grid", json!({ "name": "slime", "x": 7, "y": 7, "grid": "frame 1\nRR\nRR\n" })).unwrap_err();
    assert_eq!(out["code"], "region_out");
    let whole = call(&core, &session, "apply_grid", json!({ "name": "slime", "grid": "frame 1\nRR\nRR\n" })).unwrap_err();
    assert_eq!(whole["code"], "row_count");
}

// Checks the `frame` argument for a grid that has only rows, a frame that
// does not exist, and a grid with no frame at all.
#[test]
fn apply_grid_accepts_rows_with_a_frame_argument() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    let rows_only = "palette: main\nlegend\n  R red #FF0000\nRR..\n.GG.\n....\n....\n";

    call(&core, &session, "apply_grid", json!({ "name": "goblin", "frame": 2, "grid": rows_only })).unwrap();
    assert_eq!(rows(&host, &["Characters"], "goblin", 2), "RR../.GG./..../....");
    assert_eq!(rows(&host, &["Characters"], "goblin", 1), "R.../..../..../...G", "frame 1 is not touched");

    let missing = call(&core, &session, "apply_grid", json!({ "name": "goblin", "frame": 5, "grid": rows_only })).unwrap_err();
    assert_eq!(missing["code"], "bad_frame");
    let no_frame = call(&core, &session, "apply_grid", json!({ "name": "goblin", "grid": "RR..\n" })).unwrap_err();
    assert_eq!(no_frame["code"], "no_frame");
}

// Checks that a grid that changes nothing makes no history entry and does not
// change the revision.
#[test]
fn apply_grid_with_no_change() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    let blank = format!("frame 1\n{}\n", BLANK8.replace('/', "\n"));
    let before = revision(&host);

    let done = call(&core, &session, "apply_grid", json!({ "name": "slime", "grid": blank })).unwrap();

    assert_eq!(done["changed"], false);
    assert_eq!(revision(&host), before);
    assert_eq!(entries(&host), 0);
}

// Checks `set_pixels`: pixels are set, erased with `.`, in another frame,
// the answer has the count and the revision, and it is one history entry.
#[test]
fn set_pixels_sets_and_erases_pixels() {
    let (host, core, session) = setup();
    open_demo_project(&host);

    let done = call(&core, &session, "set_pixels", json!({ "name": "goblin", "folder": "Characters", "frame": 2, "pixels": [
        { "x": 1, "y": 1, "key": "G" }, { "x": 2, "y": 1, "key": "R" } ] })).unwrap();
    assert_eq!(done["pixels_set"], 2);
    assert_eq!(done["revision"], revision(&host));
    assert_eq!(rows(&host, &["Characters"], "goblin", 2), "..../.GR./..../....");
    assert_eq!(entries(&host), 1);

    call(&core, &session, "set_pixels", json!({ "name": "goblin", "frame": 2, "pixels": [{ "x": 1, "y": 1, "key": "." }] })).unwrap();
    assert_eq!(rows(&host, &["Characters"], "goblin", 2), "..../..R./..../....");
}

// Checks the atomic rule of `set_pixels`: one bad pixel in the list (an
// unknown key, a position outside, a bad number) and NO pixel is set. The
// unknown key lists the valid keys and says which pixel it was.
#[test]
fn set_pixels_is_all_or_nothing() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    let before = revision(&host);
    let run = |pixels: Value| call(&core, &session, "set_pixels", json!({ "name": "slime", "pixels": pixels })).unwrap_err();

    let unknown = run(json!([{ "x": 0, "y": 0, "key": "R" }, { "x": 1, "y": 0, "key": "R" }, { "x": 2, "y": 0, "key": "Z" }]));
    assert_eq!(unknown["code"], "unknown_key");
    assert_eq!(unknown["valid_keys"], json!(["R", "G", "."]));
    assert_eq!((unknown["index"].as_u64(), unknown["x"].as_u64()), (Some(2), Some(2)));
    let outside = run(json!([{ "x": 0, "y": 0, "key": "R" }, { "x": 8, "y": 0, "key": "R" }]));
    assert_eq!(outside["code"], "out_of_range");
    assert_eq!((outside["width"].as_u64(), outside["index"].as_u64()), (Some(8), Some(1)));
    assert_eq!(run(json!([{ "x": -1, "y": 0, "key": "R" }]))["code"], "bad_argument");
    assert_eq!(run(json!([{ "x": 0, "y": 0, "key": "RR" }]))["code"], "bad_argument");
    assert_eq!(run(json!("nope"))["code"], "bad_argument");
    assert_eq!(rows(&host, &[], "slime", 1), BLANK8);
    assert_eq!(revision(&host), before);
}

// Checks `draw_shape` for each shape, with the exact pixels.
#[test]
fn draw_shape_draws_lines_rectangles_ellipses_and_fills() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    let draw = |args: Value| {
        let mut full = json!({ "name": "slime" });
        full.as_object_mut().unwrap().extend(args.as_object().unwrap().clone());
        call(&core, &session, "draw_shape", full).unwrap()
    };

    draw(json!({ "shape": "line", "key": "R", "x0": 0, "y0": 0, "x1": 7, "y1": 7 }));
    assert_eq!(rows(&host, &[], "slime", 1), "R......./.R....../..R...../...R..../....R.../.....R../......R./.......R");

    draw(json!({ "shape": "rectangle", "key": "G", "x": 0, "y": 0, "width": 3, "height": 3 }));
    assert_eq!(&rows(&host, &[], "slime", 1)[..26], "GGG...../GRG...../GGG.....");

    let ellipse = draw(json!({ "shape": "ellipse", "key": "G", "x": 3, "y": 3, "width": 5, "height": 5, "filled": true }));
    assert_eq!(ellipse["pixels_changed"], 21);
    let all = rows(&host, &[], "slime", 1);
    let lower: Vec<&str> = all.split('/').skip(3).collect();
    assert_eq!(lower, vec!["...RGGG.", "...GGGGG", "...GGGGG", "...GGGGG", "....GGGR"]);
}

// Checks the bucket: it fills the connected area only, and filling with the
// key that is already there changes nothing (no history entry).
#[test]
fn draw_shape_fill_and_no_change() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    call(&core, &session, "draw_shape", json!({ "name": "slime", "shape": "line", "key": "R", "x0": 4, "y0": 0, "x1": 4, "y1": 7 })).unwrap();

    let filled = call(&core, &session, "draw_shape", json!({ "name": "slime", "shape": "fill", "key": "G", "x": 0, "y": 0 })).unwrap();

    assert_eq!(filled["pixels_changed"], 32);
    assert_eq!(rows(&host, &[], "slime", 1).split('/').next().unwrap(), "GGGGR...");
    let before = (revision(&host), entries(&host));
    let same = call(&core, &session, "draw_shape", json!({ "name": "slime", "shape": "fill", "key": "G", "x": 0, "y": 0 })).unwrap();
    assert_eq!(same["changed"], false);
    assert_eq!((revision(&host), entries(&host)), before);
}

// Checks the errors of `draw_shape`: a shape that does not fit, a key that is
// not in the palette, a bad shape name and missing numbers. Nothing changes.
#[test]
fn draw_shape_errors_change_nothing() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    let before = revision(&host);
    let run = |args: Value| {
        let mut full = json!({ "name": "slime", "key": "R" });
        full.as_object_mut().unwrap().extend(args.as_object().unwrap().clone());
        call(&core, &session, "draw_shape", full).unwrap_err()
    };

    assert_eq!(run(json!({ "shape": "rectangle", "x": 5, "y": 5, "width": 4, "height": 4 }))["code"], "out_of_range");
    assert_eq!(run(json!({ "shape": "line", "x0": 0, "y0": 0, "x1": 9, "y1": 0 }))["code"], "out_of_range");
    assert_eq!(run(json!({ "shape": "fill", "x": 8, "y": 0 }))["code"], "out_of_range");
    assert_eq!(run(json!({ "shape": "circle" }))["code"], "bad_argument");
    assert_eq!(run(json!({ "shape": "ellipse", "x": 0, "y": 0, "width": 0, "height": 2 }))["code"], "bad_argument");
    assert_eq!(run(json!({ "shape": "fill", "key": "Z", "x": 0, "y": 0 }))["code"], "unknown_key");
    assert_eq!(run(json!({ "shape": "line" }))["code"], "bad_argument");
    assert_eq!(revision(&host), before);
}

// Checks that a drawing tool needs the "Edit pixels" permission.
#[test]
fn drawing_tools_need_the_edit_permission() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    core.set_permissions(crate::mcp::Permissions { edit: false, ..core.permissions() }).unwrap();

    for (tool, args) in [
        ("apply_grid", json!({ "name": "slime", "grid": GRID8 })),
        ("set_pixels", json!({ "name": "slime", "pixels": [] })),
        ("draw_shape", json!({ "name": "slime", "shape": "fill", "key": "R", "x": 0, "y": 0 })),
    ] {
        assert_eq!(call(&core, &session, tool, args).unwrap_err()["code"], "permission_denied", "{tool}");
    }
    assert_eq!(rows(&host, &[], "slime", 1), BLANK8);
}
