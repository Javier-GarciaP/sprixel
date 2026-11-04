use super::sheet_tests::sheet_project;
use super::*;
use serde_json::{Value, json};

fn aseprite(scale: u32, spacing: u32) -> (Value, Vec<ExportFile>) {
    let options = Options { scale, spacing, json_format: JsonFormat::Aseprite, ..Options::default() };
    let files = build(&sheet_project(), &Scope::Project, &options, "kit").unwrap();
    (serde_json::from_slice(&files[1].bytes).unwrap(), files)
}

// Checks the shape Godot and Unity importers read: `frames` is an array with
// `filename`, `frame`, `duration` and `sourceSize`, and `meta` has the image
// name, the size and the scale as text (as Aseprite writes it).
#[test]
fn the_frames_and_meta_have_the_aseprite_shape() {
    let (json, files) = aseprite(1, 1);
    let frames = json["frames"].as_array().unwrap();
    assert_eq!(frames.len(), 4);
    assert_eq!(frames[1]["filename"], "hero_walk_1");
    assert_eq!(frames[1]["frame"], json!({ "x": 3, "y": 0, "w": 2, "h": 2 }));
    assert_eq!(frames[1]["duration"], 83);
    assert_eq!(frames[1]["sourceSize"], json!({ "w": 2, "h": 2 }));
    assert_eq!(frames[1]["spriteSourceSize"], json!({ "x": 0, "y": 0, "w": 2, "h": 2 }));
    assert_eq!(frames[1]["rotated"], false);
    assert_eq!(frames[1]["trimmed"], false);
    assert_eq!(json["meta"]["image"], "kit.png");
    assert_eq!(json["meta"]["size"], json!({ "w": 11, "h": 2 }));
    assert_eq!(json["meta"]["scale"], "1");
    assert_eq!(json["meta"]["format"], "RGBA8888");
    assert_eq!(files[0].name, "kit.png");
    assert_eq!(files[1].name, "kit.json");
}

// Checks that the animations become `frameTags` with positions in `frames`
// from 0, and that the tag of a project with several sprites says its sprite.
#[test]
fn animations_become_frame_tags() {
    let (json, _) = aseprite(1, 0);
    // The project has 2 sprites, so the tag name starts with the sprite.
    assert_eq!(json["meta"]["frameTags"], json!([{ "name": "hero_walk", "from": 1, "to": 2, "direction": "forward" }]));
}

// Checks that with one sprite the tag keeps its own name, and that the
// modes map to the Aseprite directions (`pingpong` stays, `once` is forward).
#[test]
fn one_sprite_keeps_the_tag_name_and_modes_map_to_directions() {
    let mut p = sheet_project();
    p.root.sprites.retain(|s| s.name == "hero");
    p.root.sprites[0].tags[0].mode = crate::document::TagMode::PingPong;
    let options = Options { json_format: JsonFormat::Aseprite, ..Options::default() };
    let files = build(&p, &Scope::Project, &options, "kit").unwrap();
    let json: Value = serde_json::from_slice(&files[1].bytes).unwrap();
    assert_eq!(json["meta"]["frameTags"][0], json!({ "name": "walk", "from": 0, "to": 1, "direction": "pingpong" }));
}

// Checks that the scale multiplies the rectangles and is written as text.
#[test]
fn the_scale_applies() {
    let (json, _) = aseprite(2, 0);
    assert_eq!(json["meta"]["scale"], "2");
    assert_eq!(json["frames"][0]["frame"]["w"], 4);
    assert_eq!(json["frames"][0]["sourceSize"], json!({ "w": 4, "h": 4 }));
}

// Checks that the PNG is the same for both JSON formats: only the JSON changes.
#[test]
fn the_png_does_not_depend_on_the_json_format() {
    let (_, aseprite_files) = aseprite(1, 1);
    let sprixel = build(&sheet_project(), &Scope::Project, &Options { spacing: 1, ..Options::default() }, "kit").unwrap();
    assert_eq!(aseprite_files[0].bytes, sprixel[0].bytes);
}

// Checks the documented sample of the Aseprite JSON is what the code writes.
#[test]
fn the_documented_aseprite_sample_is_what_the_export_writes() {
    let (_, files) = aseprite(1, 1);
    let written = String::from_utf8(files[1].bytes.clone()).unwrap();
    let doc = include_str!("../../../docs/export-format.md").replace("\r\n", "\n");
    let start = doc.find("```json aseprite\n").expect("the page has an aseprite sample") + 17;
    let end = start + doc[start..].find("```").unwrap();
    assert_eq!(&doc[start..end], written);
}

// Checks that the names of the options are the ones the tool and the screen use.
#[test]
fn json_format_names_are_parsed() {
    assert_eq!(JsonFormat::parse("sprixel"), Some(JsonFormat::Sprixel));
    assert_eq!(JsonFormat::parse("aseprite"), Some(JsonFormat::Aseprite));
    assert_eq!(JsonFormat::parse("hash"), None);
}
