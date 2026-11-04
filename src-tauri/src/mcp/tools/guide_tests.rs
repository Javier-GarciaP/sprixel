//! Spec `agent-live-design`, task 2.1: the tool `get_guide` and the guides.

use super::docs::{
    ANIMATION_GUIDE, DESIGN_GUIDE, EXAMPLE, EXAMPLE_GRID, FACES_GUIDE, GAME_READY_GUIDE, GUIDE_LIST, GUIDE_TOPICS,
    INSTRUCTIONS, LARGE_SPRITES_GUIDE, PALETTE_GUIDE, TOOLS_GUIDE, WALK_GUIDE, WORKFLOW,
};
use crate::mcp::testing::{call, core_with, open_demo_project, revision, setup};
use crate::mcp::{Permissions, tools};
use serde_json::json;
use std::collections::BTreeSet;

const DOC: &str = include_str!("../../../../docs/ai-and-mcp.md");
const SKILL_FILE: &str = include_str!("../../../../cli/internal/clients/skill/SKILL.md");

/// The skill with Unix line endings (a Windows checkout may have CRLF).
fn skill() -> String {
    SKILL_FILE.replace("\r\n", "\n")
}

// Checks the "Known topic" scenario: each topic gives its text.
#[test]
fn each_topic_gives_its_guide() {
    let (_host, core, session) = setup();

    let animation = call(&core, &session, "get_guide", json!({ "topic": "animation" })).unwrap();
    assert_eq!(animation["guide"], ANIMATION_GUIDE);
    assert_eq!(animation["topic"], "animation");
    assert_eq!(call(&core, &session, "get_guide", json!({ "topic": "design" })).unwrap()["guide"], DESIGN_GUIDE);
    assert_eq!(call(&core, &session, "get_guide", json!({ "topic": "palette" })).unwrap()["guide"], PALETTE_GUIDE);
    for (topic, text) in [
        ("large-sprites", LARGE_SPRITES_GUIDE),
        ("faces-and-animals", FACES_GUIDE),
        ("walk-and-attack-cycles", WALK_GUIDE),
        ("game-ready", GAME_READY_GUIDE),
        ("tools", TOOLS_GUIDE),
    ] {
        assert_eq!(call(&core, &session, "get_guide", json!({ "topic": topic })).unwrap()["guide"], text, "{topic}");
    }
}

// Checks the "Unknown topic" scenario: the error says what is wrong and lists
// the eight valid topics, so the agent can pick one.
#[test]
fn an_unknown_topic_lists_the_valid_ones() {
    let (_host, core, session) = setup();

    let error = call(&core, &session, "get_guide", json!({ "topic": "colour" })).unwrap_err();

    assert_eq!(error["code"], "unknown_topic");
    let message = error["message"].as_str().unwrap();
    for topic in GUIDE_TOPICS {
        assert!(message.contains(topic), "{message}");
    }
    assert_eq!(error["topics"], json!(GUIDE_TOPICS));
    assert_eq!(GUIDE_TOPICS.len(), 8);
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

// Checks that the animation guide and the skill tell an agent about the data a
// game needs: frame times, anchors and tiles (spec `game-data`).
#[test]
fn the_guide_and_the_skill_teach_the_game_data() {
    for tool in ["set_frame_duration", "set_anchor"] {
        assert!(ANIMATION_GUIDE.contains(tool), "{tool} is not in the animation guide");
    }
    for tool in ["set_frame_duration", "set_anchor", "remove_anchor", "check_tile", "render_tiled"] {
        assert!(skill().contains(tool), "{tool} is not in the skill");
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

/// Every tool, argument and error code that exists, plus the words of the text
/// grid and the animation modes.
fn known_names() -> BTreeSet<String> {
    let (_host, core) = core_with(tools::all());
    let mut known: BTreeSet<String> = documented_codes();
    for tool in &core.tools {
        known.insert(tool.name.to_string());
        if let Some(properties) = tool.schema["properties"].as_object() {
            known.extend(properties.keys().cloned());
        }
    }
    known.extend(["legend", "loop", "frame"].map(String::from));
    // The values of the arguments that have a list of values (`horizontal`, `mirror`, `move`...).
    for tool in &core.tools {
        for property in tool.schema["properties"].as_object().into_iter().flat_map(|p| p.values()) {
            known.extend(property["enum"].as_array().into_iter().flatten().filter_map(|v| v.as_str()).map(String::from));
        }
    }
    // Fields of the answers and of the errors.
    known.extend(
        ["revision", "code", "message", "errors", "line", "row", "column", "frame_durations", "anchors", "ground_kept", "bbox", "changes", "changed_pixels", "loose_pixels", "outline_gaps", "seamless", "mismatch_count", "checked", "not_checked", "outline_key_used", "ground_rule", "bottom_rows_equal", "erased", "warnings", "thin_regions", "speckled_pixels", "would_change", "feet", "null"]
            .map(String::from),
    );
    // The topics of `get_guide`.
    known.extend(super::docs::GUIDE_TOPICS.map(String::from));
    known
}

/// Panics if the text names, between backticks, a tool, argument or code that
/// does not exist. Only words made of lowercase letters and `_` count (so
/// commands like `sprixel doctor` and symbols like `.` are not checked).
fn assert_names_are_real(what: &str, text: &str) {
    let known = known_names();
    for word in quoted_words(text) {
        let looks_like_a_name = !word.is_empty() && word.chars().all(|c| c.is_ascii_lowercase() || c == '_');
        if looks_like_a_name {
            assert!(known.contains(&word) || word == "sprixel", "{what} names `{word}`, which does not exist");
        }
    }
}

// Checks the "Names are real" scenario: every tool, argument and error code
// that a guide names between backticks exists, so a guide cannot send an
// agent to a tool that is not there.
#[test]
fn every_name_in_the_guides_is_real() {
    for (name, text) in [
        ("the design guide", DESIGN_GUIDE),
        ("the animation guide", ANIMATION_GUIDE),
        ("the palette guide", PALETTE_GUIDE),
        ("the large sprites guide", LARGE_SPRITES_GUIDE),
        ("the faces and animals guide", FACES_GUIDE),
        ("the walk and attack guide", WALK_GUIDE),
        ("the game ready guide", GAME_READY_GUIDE),
        ("the tools guide", TOOLS_GUIDE),
    ] {
        assert_names_are_real(name, text);
    }
}

// Checks the "Guides have their rules" scenario: each new guide says the ideas
// the spec asks for.
#[test]
fn the_new_guides_have_their_rules() {
    let checks: [(&str, &str, &[&str]); 4] = [
        ("large sprites", LARGE_SPRITES_GUIDE, &["one part at a time", "symmetry", "import_reference", "analyze_sprite", "base_revision"]),
        ("faces and animals", FACES_GUIDE, &["silhouette first", "eyes", "symmetry", "highlight"]),
        ("walk and attack", WALK_GUIDE, &["contact", "passing", "4 frames", "set_frame_duration", "ground", "loop closes", "anticipation", "recovery"]),
        ("game ready", GAME_READY_GUIDE, &["set_anchor", "feet", "set_frame_duration", "check_tile", "render_tiled", "export"]),
    ];
    for (name, text, parts) in checks {
        for part in parts {
            assert!(text.to_lowercase().contains(part), "the {name} guide does not say {part:?}");
        }
    }
}

// Checks the "No tool is forgotten" scenario: every tool of the server is
// named in the tools guide, so a new tool cannot be left out of the map.
#[test]
fn the_tools_guide_names_every_tool() {
    let (_host, core) = core_with(tools::all());
    for tool in &core.tools {
        assert!(TOOLS_GUIDE.contains(&format!("`{}`", tool.name)), "the tools guide does not name {}", tool.name);
    }
}

// Checks the "Instructions, skill and docs agree" scenario: the same sentence
// lists the topics in the three places, and it names all eight.
#[test]
fn the_instructions_the_skill_and_the_docs_list_the_same_guides() {
    for topic in GUIDE_TOPICS {
        assert!(GUIDE_LIST.contains(&format!("`{topic}`")), "the list does not name {topic}");
    }
    assert!(INSTRUCTIONS.contains(GUIDE_LIST), "the instructions");
    assert!(skill().contains(GUIDE_LIST), "the skill");
    assert!(DOC.replace("\r\n", "\n").contains(GUIDE_LIST), "docs/ai-and-mcp.md");
}

// Checks the same for the other places an agent reads: the workflow, the
// example, the server instructions and the Claude Code skill.
#[test]
fn every_name_in_the_workflow_the_instructions_and_the_skill_is_real() {
    assert_names_are_real("the workflow", WORKFLOW);
    assert_names_are_real("the example", EXAMPLE);
    assert_names_are_real("the server instructions", INSTRUCTIONS);
    assert_names_are_real("the skill", &skill());
}

/// The numbered steps of a text, in order: the lines that start with `N. `.
fn steps(text: &str) -> Vec<String> {
    text.lines()
        .filter(|line| line.chars().next().is_some_and(|c| c.is_ascii_digit()) && line.contains(". "))
        .map(str::to_string)
        .collect()
}

// Checks the "Skill and server agree" scenario: the server instructions and the
// skill both carry the workflow word for word, the same steps in the same order,
// and the worked example after it. If someone edits one and not the other, this
// fails.
#[test]
fn the_skill_and_the_server_carry_the_same_workflow_and_example() {
    let skill = skill();
    for (name, text) in [("the server instructions", INSTRUCTIONS.to_string()), ("the skill", skill.clone())] {
        let workflow = text.find(WORKFLOW).unwrap_or_else(|| panic!("{name} does not have the workflow"));
        let example = text.find(EXAMPLE).unwrap_or_else(|| panic!("{name} does not have the worked example"));
        assert!(workflow < example, "{name}: the workflow comes before the example");
        // The text is the same word for word, so the steps are the same and in the same order.
        let found = steps(&text[workflow..workflow + WORKFLOW.len()]);
        assert_eq!(found, steps(WORKFLOW), "{name}: the steps of the workflow");
        assert_eq!(found.len(), 6);
    }
    // The skill also carries the design rules of the guide.
    assert!(skill.contains(DESIGN_GUIDE), "the skill does not have the design guide");
}

// Checks that the workflow ends by looking at the image, and says so in the
// strongest words, in the three places (the guide, the instructions, the skill).
#[test]
fn everyone_is_told_to_look_at_the_image() {
    for (name, text) in [("the workflow", WORKFLOW.to_string()), ("the instructions", INSTRUCTIONS.to_string()), ("the skill", skill())] {
        assert!(text.contains("LOOK at the image"), "{name}");
        assert!(text.contains("preview"), "{name} should name the preview argument or tool");
    }
}

// Checks that the worked example really works: on a project with a normal
// palette, the calls of the example make a 16x16 sprite with two frames that
// are different (idle and jump), with the five colors of its legend.
#[test]
fn the_worked_example_runs() {
    use crate::mcp::testing::{call, open_demo_project, setup};
    let (host, core, session) = setup();
    open_demo_project(&host);

    call(&core, &session, "create_sprite", json!({ "name": "slime16", "width": 16, "height": 16 })).unwrap();
    call(&core, &session, "add_frame", json!({ "name": "slime16" })).unwrap();
    let done = call(&core, &session, "apply_grid", json!({ "name": "slime16", "grid": EXAMPLE_GRID })).unwrap();
    assert_eq!(done["colors_added"], 5);

    let frame = |n: u32| {
        call(&core, &session, "get_sprite", json!({ "name": "slime16", "frame": n })).unwrap()["grid"].as_str().unwrap().to_string()
    };
    assert_ne!(frame(1), frame(2), "idle and jump are different");
    assert!(frame(1).contains('K') && frame(2).contains('K'));
    // The grid of the text is exactly what the example prints.
    assert!(EXAMPLE.contains(EXAMPLE_GRID));
}

// Task 6.1: the animation and walk guides require what a real agent got wrong:
// a body bob, opposite arms, 6 or 8 frames, a fixed support foot with the
// `feet` anchor, ground contact in actions, and `ground_kept` that follows the
// support foot.
#[test]
fn the_animation_guides_require_bob_arms_frames_and_the_support_foot() {
    let walk = WALK_GUIDE.to_lowercase();
    for part in ["6 or 8 frames", "body bob", "up and down", "opposite", "support foot", "`feet`", "ground_kept", "follows the support foot", "a peck reaches the ground row"] {
        assert!(walk.contains(part), "the walk guide does not say {part:?}");
    }
    let animation = ANIMATION_GUIDE.to_lowercase();
    for part in ["support foot", "`feet`", "ground_kept", "bob", "opposite", "peck reaches the ground row"] {
        assert!(animation.contains(part), "the animation guide does not say {part:?}");
    }
    // The skill carries the design rules, so it has the new tools' words too.
    let skill = skill();
    for part in ["dry_run", "`checked`", "not_checked"] {
        assert!(skill.contains(part), "the skill does not say {part:?}");
    }
}

// Task 6.2: the `tools` guide has one example call per tool, as a line
// "`tool` {json}". Each one must parse, name a real tool, give every required
// argument and only arguments that the tool has. No tool is left without one.
#[test]
fn the_tools_guide_has_a_valid_example_call_for_every_tool() {
    let (_host, core) = core_with(tools::all());
    let mut seen = BTreeSet::new();
    for line in TOOLS_GUIDE.lines().filter(|l| l.starts_with('`')) {
        let Some((name, json_text)) = line[1..].split_once("` ") else { continue };
        let Ok(arguments) = serde_json::from_str::<serde_json::Value>(json_text) else { continue };
        let tool = core.tools.iter().find(|t| t.name == name).unwrap_or_else(|| panic!("the example calls {name}, which does not exist"));
        let properties = tool.schema["properties"].as_object().cloned().unwrap_or_default();
        for key in arguments.as_object().unwrap().keys() {
            assert!(properties.contains_key(key), "the example of {name} uses `{key}`, which the tool does not have");
        }
        for required in tool.schema["required"].as_array().into_iter().flatten() {
            assert!(arguments.get(required.as_str().unwrap()).is_some(), "the example of {name} misses `{required}`");
        }
        seen.insert(name.to_string());
    }
    for tool in &core.tools {
        assert!(seen.contains(tool.name), "the tools guide has no example call for {}", tool.name);
    }
}
