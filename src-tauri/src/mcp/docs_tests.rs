//! Task 5.2: `docs/ai-and-mcp.md` and the tools can not drift apart.

use super::testing::core_with;
use super::tools;
use std::collections::BTreeSet;

const DOC: &str = include_str!("../../../docs/ai-and-mcp.md");

/// The tool names in the tables of "What the AI can do": the first cell of a
/// row, written as `name`.
fn documented_tools() -> BTreeSet<String> {
    let text = DOC.replace("\r\n", "\n");
    let start = text.find("## What the AI can do: tools").expect("the tools section");
    let end = start + text[start..].find("### Errors").expect("the errors section");
    text[start..end]
        .lines()
        .filter_map(|line| line.strip_prefix("| `"))
        .filter_map(|rest| rest.split('`').next())
        .map(str::to_string)
        .collect()
}

// Checks that every tool the server has is described in the page, and that
// the page does not describe a tool the server does not have.
#[test]
fn every_tool_is_documented_and_nothing_else() {
    let (_host, core) = core_with(tools::all());
    let real: BTreeSet<String> = core.tools.iter().map(|t| t.name.to_string()).collect();
    assert_eq!(documented_tools(), real);
    assert_eq!(real.len(), 40);
}

// Checks that the permissions of the page are the real defaults and that the
// limits of the page are the real limits, so a change in the code needs a
// change in the page.
#[test]
fn the_page_has_the_real_limits_and_defaults() {
    let limits = super::limits::Limits::default();
    assert_eq!(limits.body, 4 * 1024 * 1024);
    assert!(DOC.contains("4 MB per request"));
    assert!(DOC.contains(&format!("{} requests per second", limits.requests_per_second)));
    assert!(DOC.contains(&format!("{} seconds to\n  send a request", limits.request_time.as_secs())));
    assert!(DOC.contains(&format!("{} clients at the\n  same time", super::MAX_SESSIONS)));
    assert!(DOC.contains(&format!("{} connections open at once", limits.connections)));
    assert!(DOC.contains(&format!("{} clients waiting", super::clients::MAX_PENDING)));
    let defaults = super::Permissions::default();
    assert!(defaults.create && defaults.edit && defaults.export && defaults.ask_on_connect && !defaults.delete && !defaults.open_project);
    assert!(DOC.contains("| Delete sprites (`delete_sprite`) | **Off** |"));
    assert!(DOC.contains("| Open projects (`open_project`) | **Off** |"));
}
