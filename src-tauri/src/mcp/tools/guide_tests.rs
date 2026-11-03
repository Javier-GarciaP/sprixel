//! Spec `agent-live-design`, task 2.1: the tool `get_guide` and the guides.

use super::docs::{ANIMATION_GUIDE, DESIGN_GUIDE, PALETTE_GUIDE};
use crate::mcp::testing::{call, core_with, open_demo_project, revision, setup};
use crate::mcp::{Permissions, tools};
use serde_json::json;
use std::collections::BTreeSet;

const DOC: &str = include_str!("../../../../docs/ai-and-mcp.md");

// Checks the "Known topic" scenario: each of the three topics gives its text.
#[test]
fn each_topic_gives_its_guide() {
    let (_host, core, session) = setup();

    let animation = call(&core, &session, "get_guide", json!({ "topic": "animation" })).unwrap();
    assert_eq!(animation["guide"], ANIMATION_GUIDE);
    assert_eq!(animation["topic"], "animation");
    assert_eq!(call(&core, &session, "get_guide", json!({ "topic": "design" })).unwrap()["guide"], DESIGN_GUIDE);
    assert_eq!(call(&core, &session, "get_guide", json!({ "topic": "palette" })).unwrap()["guide"], PALETTE_GUIDE);
}

// Checks the "Unknown topic" scenario: the error says what is wrong and lists
// the three valid topics, so the agent can pick one.
#[test]
fn an_unknown_topic_lists_the_valid_ones() {
    let (_host, core, session) = setup();

    let error = call(&core, &session, "get_guide", json!({ "topic": "colour" })).unwrap_err();

    assert_eq!(error["code"], "unknown_topic");
    let message = error["message"].as_str().unwrap();
    for topic in ["design", "animation", "palette"] {
        assert!(message.contains(topic), "{message}");
    }
    assert_eq!(error["topics"], json!(["design", "animation", "palette"]));
    assert_eq!(call(&core, &session, "get_guide", json!({})).unwrap_err()["code"], "bad_argument");
}

// Checks that the tool needs no permission (even with every permission off)
// and does not change the project: same revision, no history entry.
#[test]
fn the_guide_needs_no_permission_and_changes_nothing() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    core.set_permissions(Permissions { create: false, edit: false, export: false, delete: false, open_project: false, ..core.permissions() }).unwrap();
    let before = revision(&host);
    let entries = crate::ipc::project::read(&host.project, |s| s.history.entries().len()).unwrap();

    call(&core, &session, "get_guide", json!({ "topic": "design" })).unwrap();

    assert_eq!(revision(&host), before);
    assert_eq!(crate::ipc::project::read(&host.project, |s| s.history.entries().len()).unwrap(), entries);
}

// Checks the content the spec asks for: the design guide teaches a small
// palette with ramps, one light direction, silhouette first, flat fills, two or
// three tones, selective outline, no stray pixels, the review loop with a look
// at the image, and region by region for big sprites.
#[test]
fn the_design_guide_has_the_rules() {
    let text = DESIGN_GUIDE.to_lowercase();
    for part in [
        "small palette", "ramp", "one light direction", "silhouette first", "flat fill", "two or three tones",
        "selective outline", "stray pixels", "render_preview", "look at", "region by region",
    ] {
        assert!(text.contains(part), "the design guide does not say {part:?}");
    }
}

// Checks the animation guide: key poses first, the same palette in every frame
// and a loop that closes.
#[test]
fn the_animation_guide_has_the_rules() {
    let text = ANIMATION_GUIDE.to_lowercase();
    for part in ["key poses first", "same palette in every frame", "loop closes", "create_tag"] {
        assert!(text.contains(part), "the animation guide does not say {part:?}");
    }
}

/// The codes of the table of errors in the page.
fn documented_codes() -> BTreeSet<String> {
    let text = DOC.replace("\r\n", "\n");
    let start = text.find("### Errors").expect("the errors section");
    let end = start + text[start..].find("\n## ").expect("the next section");
    text[start..end]
        .lines()
        .filter_map(|line| line.strip_prefix("| "))
        .filter_map(|row| row.split(" | ").next())
        .flat_map(|cell| cell.split('`').skip(1).step_by(2).map(str::to_string).collect::<Vec<_>>())
        .collect()
}

/// Every word between backticks in a text.
pub(super) fn quoted_words(text: &str) -> Vec<String> {
    text.split('`').skip(1).step_by(2).map(str::to_string).collect()
}

// Checks the "Names are real" scenario: every tool, argument and error code
// that a guide names between backticks exists, so a guide cannot send an
// agent to a tool that is not there.
#[test]
fn every_name_in_the_guides_is_real() {
    let (_host, core) = core_with(tools::all());
    let mut known: BTreeSet<String> = documented_codes();
    for tool in &core.tools {
        known.insert(tool.name.to_string());
        if let Some(properties) = tool.schema["properties"].as_object() {
            known.extend(properties.keys().cloned());
        }
    }
    // Words of the text grid and of the animation modes.
    known.extend(["legend", "loop"].map(String::from));
    for (name, text) in [("design", DESIGN_GUIDE), ("animation", ANIMATION_GUIDE), ("palette", PALETTE_GUIDE)] {
        for word in quoted_words(text) {
            let looks_like_a_name = !word.is_empty() && word.chars().all(|c| c.is_ascii_lowercase() || c == '_');
            if looks_like_a_name {
                assert!(known.contains(&word), "the {name} guide names `{word}`, which does not exist");
            }
        }
    }
}
