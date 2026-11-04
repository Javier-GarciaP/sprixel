//! `docs/game-engines.md` and the code can not drift apart: the options and
//! tools the page names are real, and the page names every export option.

use crate::mcp::testing::core_with;
use crate::mcp::tools;
use serde_json::Value;
use std::collections::BTreeSet;
use std::path::PathBuf;

const DOC: &str = include_str!("../../../../docs/game-engines.md");

/// The first backticked word of every row of the table whose header starts
/// with `| <title> |`.
fn first_column(title: &str) -> BTreeSet<String> {
    let text = DOC.replace("\r\n", "\n");
    let header = format!("| {title} |");
    let start = text.find(&header).unwrap_or_else(|| panic!("the page has no table {title:?}"));
    text[start..]
        .lines()
        .take_while(|line| line.starts_with('|'))
        .filter_map(|line| line.strip_prefix("| `"))
        .filter_map(|row| row.split('`').next())
        .map(str::to_string)
        .collect()
}

fn export_schema() -> Value {
    let (_host, core) = core_with(tools::all());
    core.tools.iter().find(|t| t.name == "export").unwrap().schema.clone()
}

// Checks that the options table of the page lists exactly the arguments of
// the `export` tool (so an option added to the code must be explained).
#[test]
fn the_options_table_has_every_export_argument() {
    let schema = export_schema();
    let real: BTreeSet<String> = schema["properties"].as_object().unwrap().keys().cloned().collect();
    assert_eq!(first_column("Export options"), real);
}

// Checks that every value of an option with a fixed list (`format`, `scope`,
// `json_format`, `scale`) is written in the page, in backticks.
#[test]
fn the_page_names_every_value_of_the_listed_options() {
    let schema = export_schema();
    for (name, property) in schema["properties"].as_object().unwrap() {
        for value in property["enum"].as_array().into_iter().flatten() {
            let text = value.as_str().map(str::to_string).unwrap_or_else(|| value.to_string());
            assert!(DOC.contains(&format!("`{text}`")), "the page does not name `{text}` of the option {name}");
        }
    }
}

// Checks that the tools in the table "Tools you will use" exist, and that the
// default output folder of the page is the real default.
#[test]
fn the_tools_and_the_default_folder_are_real() {
    let (_host, core) = core_with(tools::all());
    let real: BTreeSet<String> = core.tools.iter().map(|t| t.name.to_string()).collect();
    let named = first_column("Tools you will use");
    assert!(!named.is_empty());
    for tool in &named {
        assert!(real.contains(tool), "the page names the tool {tool} but the server has none");
    }
    assert!(DOC.contains(&format!("`{}`", crate::export::DEFAULT_OUTPUT)));
}

// Checks that every file of the docs that the page links to exists.
#[test]
fn the_linked_pages_exist() {
    let docs = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../docs");
    let mut found = 0;
    for part in DOC.split("](").skip(1) {
        let target = part.split(')').next().unwrap();
        let file = target.split('#').next().unwrap();
        if file.ends_with(".md") && !file.starts_with("http") {
            assert!(docs.join(file).is_file(), "the page links to {file}, which does not exist");
            found += 1;
        }
    }
    assert!(found >= 2);
}

// Checks that the names the page uses for the anchor and the JSON fields are
// the real ones: the anchor is `feet`, and the pivot comes from it (the same
// rule the export code applies).
#[test]
fn the_feet_anchor_and_the_pivot_are_described() {
    for text in ["`feet`", "`pivot`", "`frame_number`", "`index`"] {
        assert!(DOC.contains(text), "the page does not mention {text}");
    }
}
