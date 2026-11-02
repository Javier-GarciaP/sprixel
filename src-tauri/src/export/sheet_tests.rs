use super::*;
use crate::document::{Color, Folder, Frame, Palette, Project, Sprite, Tag, TagMode};

fn project() -> Project {
    let mut p = Project::new("Dungeon Kit");
    p.palettes.push(Palette {
        name: "main".to_string(),
        colors: vec![Color { key: 'R', name: "red".to_string(), hex: "#FF0000".to_string() }],
    });
    p
}

/// A 2x2 sprite with `frames` frames. Frame n has the key `R` in its first
/// pixel only when n is odd, so frames differ.
fn sprite(name: &str, frames: usize) -> Sprite {
    let mut s = Sprite::new(name, 2, 2, "main").unwrap();
    s.frame_durations = vec![100; frames];
    s.layers[0].frames = (0..frames)
        .map(|n| Frame { pixels: if n % 2 == 0 { b"R...".to_vec() } else { b"...R".to_vec() } })
        .collect();
    s
}

fn walk(from: u32, to: u32) -> Tag {
    Tag { name: "walk".to_string(), from, to, fps: 12, mode: TagMode::Loop }
}

fn sheet_project() -> Project {
    let mut p = project();
    let mut hero = sprite("hero", 3);
    hero.tags.push(walk(1, 2));
    hero.frame_durations[2] = 250;
    p.root.sprites.push(hero);
    p.root.sprites.push(sprite("blob", 1));
    p
}

fn options(scale: u32, spacing: u32) -> Options {
    Options { scale, spacing, ..Options::default() }
}

fn json_of(files: &[ExportFile]) -> String {
    String::from_utf8(files[1].bytes.clone()).unwrap()
}

// Checks the whole JSON against a golden file: frames ordered by sprite name
// ("blob" before "hero"), durations (the walk speed 12 fps = 83 ms, an own
// duration of 250 ms, and the 100 ms of a frame in no animation), the
// animation with positions in the sheet, and the meta block.
#[test]
fn json_matches_the_golden_file() {
    let files = build(&sheet_project(), &Scope::Project, &options(1, 1), "kit").unwrap();
    let golden = include_str!("testdata/sheet.json");
    assert_eq!(json_of(&files), golden);
}

// Checks the spec scenario: 14 frames of 8x8 with spacing 1 make a PNG of
// 71x17 and a JSON that lists 14 frames with their positions.
#[test]
fn fourteen_frames_make_a_71x17_sheet() {
    let mut p = project();
    let mut s = Sprite::new("goblin", 8, 8, "main").unwrap();
    s.frame_durations = vec![100; 14];
    s.layers[0].frames = vec![Frame::blank(8, 8); 14];
    p.root.folders.push(Folder { name: "Personajes".to_string(), folders: vec![], sprites: vec![s] });
    let scope = Scope::Folder(vec!["Personajes".to_string()]);
    let files = build(&p, &scope, &options(1, 1), "personajes").unwrap();
    assert_eq!(files[0].name, "personajes.png");
    assert_eq!(files[1].name, "personajes.json");
    let decoded = image::load_from_memory(&files[0].bytes).unwrap();
    assert_eq!((decoded.width(), decoded.height()), (71, 17));
    let json: serde_json::Value = serde_json::from_str(&json_of(&files)).unwrap();
    assert_eq!(json["frames"].as_array().unwrap().len(), 14);
    assert_eq!(json["frames"][8]["frame"], serde_json::json!({"x": 0, "y": 9, "w": 8, "h": 8}));
    assert_eq!(json["meta"]["size"], serde_json::json!({"w": 71, "h": 17}));
}

// Checks that the PNG has the frames at the places the JSON says, and that the
// space between frames is transparent.
#[test]
fn png_pixels_are_where_the_json_says() {
    let files = build(&sheet_project(), &Scope::Project, &options(1, 1), "kit").unwrap();
    let sheet = image::load_from_memory(&files[0].bytes).unwrap().to_rgba8();
    // blob frame 1 at x=0: red in its first pixel. hero frame 1 at x=3.
    assert_eq!(sheet.get_pixel(0, 0).0, [255, 0, 0, 255]);
    assert_eq!(sheet.get_pixel(2, 0).0, [0, 0, 0, 0]);
    assert_eq!(sheet.get_pixel(3, 0).0, [255, 0, 0, 255]);
    // hero frame 2 at x=6 has the red pixel in its last (right, bottom) corner.
    assert_eq!(sheet.get_pixel(7, 1).0, [255, 0, 0, 255]);
}

// Checks that the scale is applied to the sheet and the frames in the JSON
// (nearest-neighbor, 2x: a frame of 2x2 is 4x4 and the spacing is scaled too).
#[test]
fn scale_applies_to_the_sheet_and_the_json() {
    let files = build(&sheet_project(), &Scope::Project, &options(2, 1), "kit").unwrap();
    let json: serde_json::Value = serde_json::from_str(&json_of(&files)).unwrap();
    assert_eq!(json["meta"]["scale"], 2);
    assert_eq!(json["frames"][0]["frame"]["w"], 4);
    assert_eq!(json["frames"][1]["frame"]["x"], 6);
}

// Checks the "same result every time" requirement: two runs give the same bytes,
// also when the sprites were added in another order.
#[test]
fn two_exports_are_byte_identical() {
    let a = build(&sheet_project(), &Scope::Project, &options(1, 1), "kit").unwrap();
    let mut reversed = sheet_project();
    reversed.root.sprites.reverse();
    let b = build(&reversed, &Scope::Project, &options(1, 1), "kit").unwrap();
    assert_eq!(a, b);
    assert_eq!(a, build(&sheet_project(), &Scope::Project, &options(1, 1), "kit").unwrap());
}

// Checks that an empty scope is refused with `nothing_to_export`.
#[test]
fn an_empty_scope_is_nothing_to_export() {
    let error = build(&project(), &Scope::Project, &Options::default(), "kit").unwrap_err();
    assert_eq!(error.code, "nothing_to_export");
}

// Checks that wrong options are refused: scale 3 and spacing 17.
#[test]
fn wrong_options_are_refused() {
    let p = sheet_project();
    assert_eq!(build(&p, &Scope::Project, &options(3, 0), "kit").unwrap_err().code, "bad_options");
    assert_eq!(build(&p, &Scope::Project, &options(1, 17), "kit").unwrap_err().code, "bad_options");
}
