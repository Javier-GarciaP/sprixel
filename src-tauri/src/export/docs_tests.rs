use super::sheet_tests::sheet_project;
use super::*;

const DOC: &str = include_str!("../../../docs/export-format.md");

/// The first ```json block of the page.
fn sample_in_doc() -> String {
    let text = DOC.replace("\r\n", "\n");
    let start = text.find("```json\n").expect("the page has a json sample") + 8;
    let end = start + text[start..].find("```").unwrap();
    text[start..end].to_string()
}

// Checks that the sample JSON of `docs/export-format.md` is what the code
// writes: the page and the exporter can not drift apart.
#[test]
fn the_documented_sample_is_what_the_export_writes() {
    let options = Options { scale: 1, spacing: 1, ..Options::default() };
    let files = build(&sheet_project(), &Scope::Project, &options, "kit").unwrap();
    let written = String::from_utf8(files[1].bytes.clone()).unwrap();
    assert_eq!(sample_in_doc(), written);
}

// Checks that every field the page lists in its table is in the real JSON,
// so a field added to the code must be documented.
#[test]
fn every_field_of_the_json_is_documented() {
    let options = Options { scale: 1, spacing: 1, ..Options::default() };
    let files = build(&sheet_project(), &Scope::Project, &options, "kit").unwrap();
    let json: serde_json::Value = serde_json::from_slice(&files[1].bytes).unwrap();
    let mut paths = Vec::new();
    for (section, value) in json.as_object().unwrap() {
        match value {
            serde_json::Value::Array(items) => {
                for key in items[0].as_object().unwrap().keys() {
                    paths.push(format!("{section}[].{key}"));
                }
            }
            serde_json::Value::Object(map) => {
                for key in map.keys() {
                    paths.push(format!("{section}.{key}"));
                }
            }
            _ => {}
        }
    }
    for path in paths {
        let (head, field) = path.rsplit_once('.').unwrap();
        // `from` and `to` share a row in the table.
        let row = if matches!(field, "from" | "to") { format!("`{head}.from`") } else { format!("`{path}`") };
        assert!(DOC.contains(&row), "{path} is not in the table of the page");
    }
}

// Checks that the page explains the engine mapping: the `SpriteFrames` of
// Godot and the clips of Unity, and that the code samples use real field names
// of the JSON.
#[test]
fn the_page_explains_godot_and_unity() {
    for text in ["### Godot 4 (`SpriteFrames`)", "### Unity", "AtlasTexture", "AnimationClip", "frames[].anchors"] {
        assert!(DOC.contains(text), "the page has no {text:?}");
    }
    let options = Options { scale: 1, spacing: 1, ..Options::default() };
    let files = build(&sheet_project(), &Scope::Project, &options, "kit").unwrap();
    let json: serde_json::Value = serde_json::from_slice(&files[1].bytes).unwrap();
    for field in ["meta", "frames", "animations"] {
        assert!(json.get(field).is_some());
    }
    for used in ["data.meta.image", "f.frame.x", "f.duration", "anim.fps", "anim.mode", "anim.from"] {
        assert!(DOC.contains(used));
    }
}
