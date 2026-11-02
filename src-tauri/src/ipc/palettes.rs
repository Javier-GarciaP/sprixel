//! What the palette editor asks of the core: the colors of a palette, and the
//! one edit it sends on Done. The edit is one history entry by the user, so
//! undo is the one of the project. The functions do not need a window; the
//! commands at the end add the `project://changed` event.

use super::browser::{self, CHANGED_EVENT, ChangedEvent, CoreError};
use super::editor::EditorState;
use super::project::{self, ProjectState};
use crate::document::{Color, ColorEdit, DocError, Operation, SpriteRef};
use serde::{Deserialize, Serialize};

const USER: &str = crate::history::USER;

#[derive(Debug, Serialize, PartialEq)]
pub struct ColorInfo {
    pub key: String,
    pub name: String,
    pub hex: String,
    /// True when a sprite has pixels with this key.
    pub used: bool,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct PaletteInfo {
    pub name: String,
    pub colors: Vec<ColorInfo>,
}

/// One color of the final list the editor sends. `from` is the key the color
/// had when the editor opened, `None` for a color added in the editor.
#[derive(Debug, Clone, Deserialize)]
pub struct ColorArg {
    pub from: Option<String>,
    pub key: String,
    pub name: String,
    pub hex: String,
}

/// What the pixels of a removed color become: the key of a color that stays,
/// or `.` for transparent.
#[derive(Debug, Clone, Deserialize)]
pub struct Replacement {
    pub key: String,
    pub with: String,
}

// One character, or the error the editor shows under the key field.
fn one_char(text: &str) -> Result<char, DocError> {
    let mut chars = text.chars();
    match (chars.next(), chars.next()) {
        (Some(c), None) => Ok(c),
        _ => Err(DocError::new("bad_key", "a key is one printable ASCII character")),
    }
}

pub fn palette_info(state: &ProjectState, name: &str) -> Result<PaletteInfo, CoreError> {
    project::read(state, |session| {
        let palette = session.project.palette(name).ok_or_else(|| {
            CoreError::from(DocError::new("not_found", format!("palette {name:?} does not exist")))
        })?;
        let used = session.project.used_keys(&palette.name);
        Ok(PaletteInfo {
            name: palette.name.clone(),
            colors: palette
                .colors
                .iter()
                .map(|c| ColorInfo {
                    key: c.key.to_string(),
                    name: c.name.clone(),
                    hex: c.hex.clone(),
                    used: used.contains(&c.key),
                })
                .collect(),
        })
    })
    .ok_or_else(CoreError::no_project)?
}

/// Applies the editor session as one history entry by the user. While an
/// agent writes to a sprite that uses the palette, an edit that rewrites
/// pixels (a new key, a removed color) is refused with `agent_writing`; a new
/// color value or name is applied.
pub fn apply_edit(
    state: &ProjectState,
    agents: &EditorState,
    name: &str,
    colors: &[ColorArg],
    replacements: &[Replacement],
) -> Result<ChangedEvent, CoreError> {
    let edits = colors
        .iter()
        .map(|c| {
            Ok(ColorEdit {
                from: c.from.as_deref().map(one_char).transpose()?,
                color: Color {
                    key: one_char(&c.key)?,
                    name: c.name.trim().to_string(),
                    hex: c.hex.to_ascii_uppercase(),
                },
            })
        })
        .collect::<Result<Vec<_>, DocError>>()?;
    let replacements = replacements
        .iter()
        .map(|r| Ok((one_char(&r.key)?, one_char(&r.with)?)))
        .collect::<Result<Vec<_>, DocError>>()?;
    browser::change(state, |session| {
        let ops = session.project.palette_edit_operations(name, &edits, &replacements)?;
        let rewrites = ops
            .iter()
            .any(|op| matches!(op, Operation::SetPalette { pixels, .. } if !pixels.is_empty()));
        if rewrites {
            let writing = agents.writing().and_then(|w| {
                let sprite = SpriteRef { folder: w.sprite.folder, name: w.sprite.name };
                session.project.find_sprite(&sprite).map(|s| s.palette.clone())
            });
            if writing.is_some_and(|palette| palette.eq_ignore_ascii_case(name)) {
                return Err(DocError::new(
                    "agent_writing",
                    "an agent is writing to a sprite that uses this palette, wait or stop it first",
                ));
            }
        }
        let touched = session.apply(USER, ops)?;
        Ok(((), touched))
    })
    .map(|(_, event)| event)
}

#[tauri::command]
pub fn palette_info_of(
    state: tauri::State<ProjectState>,
    name: String,
) -> Result<PaletteInfo, CoreError> {
    palette_info(&state, &name)
}

#[tauri::command]
pub fn palette_apply(
    app: tauri::AppHandle,
    state: tauri::State<ProjectState>,
    agents: tauri::State<EditorState>,
    name: String,
    colors: Vec<ColorArg>,
    replacements: Vec<Replacement>,
) -> Result<(), CoreError> {
    let event = apply_edit(&state, &agents, &name, &colors, &replacements)?;
    let _ = tauri::Emitter::emit(&app, CHANGED_EVENT, event);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::{Operation, Palette, PixelChange, SpriteRef};
    use crate::ipc::browser::TargetInfo;

    fn color(key: char, name: &str, hex: &str) -> Color {
        Color { key, name: name.to_string(), hex: hex.to_string() }
    }

    /// An open project with the palette `main` (red R, blue B, green G) and a
    /// 2x2 sprite `goblin` that draws red and blue.
    fn open() -> (tempfile::TempDir, ProjectState) {
        let parent = tempfile::tempdir().unwrap();
        let state = ProjectState::default();
        project::create_project(&state, parent.path(), "Kit").unwrap();
        project::edit(&state, |session| {
            let ops = vec![
                Operation::CreatePalette {
                    palette: Palette {
                        name: "main".to_string(),
                        colors: vec![
                            color('R', "red", "#F04545"),
                            color('B', "blue", "#2F6BFF"),
                            color('G', "green", "#4ADE80"),
                        ],
                    },
                    index: None,
                },
                Operation::CreateSprite {
                    folder: vec![],
                    name: "goblin".to_string(),
                    width: 2,
                    height: 2,
                    palette: "main".to_string(),
                },
                Operation::SetPixels {
                    sprite: SpriteRef::new(&[], "goblin"),
                    layer: 0,
                    frame: 1,
                    pixels: vec![
                        PixelChange { x: 0, y: 0, key: 'R' },
                        PixelChange { x: 1, y: 0, key: 'B' },
                    ],
                },
            ];
            session.apply(USER, ops).unwrap();
            session.history.clear();
        })
        .unwrap();
        (parent, state)
    }

    fn arg(from: Option<&str>, key: &str, name: &str, hex: &str) -> ColorArg {
        ColorArg {
            from: from.map(str::to_string),
            key: key.to_string(),
            name: name.to_string(),
            hex: hex.to_string(),
        }
    }

    fn same() -> Vec<ColorArg> {
        vec![
            arg(Some("R"), "R", "red", "#F04545"),
            arg(Some("B"), "B", "blue", "#2F6BFF"),
            arg(Some("G"), "G", "green", "#4ADE80"),
        ]
    }

    fn goblin_pixels(state: &ProjectState) -> String {
        project::read(state, |s| {
            let sprite = s.project.find_sprite(&SpriteRef::new(&[], "goblin")).unwrap();
            String::from_utf8(sprite.layers[0].frames[0].pixels.clone()).unwrap()
        })
        .unwrap()
    }

    fn touches(event: &ChangedEvent, kind: &'static str, path: &[&str]) -> bool {
        event.touched.contains(&TargetInfo {
            kind,
            path: path.iter().map(|p| p.to_string()).collect(),
        })
    }

    // Checks that the info has each color with its key, name and value, and
    // says which colors a sprite uses (red and blue, not green).
    #[test]
    fn reads_the_colors_and_which_are_used() {
        let (_dir, state) = open();
        let info = palette_info(&state, "main").unwrap();
        assert_eq!(info.name, "main");
        let used: Vec<(&str, bool)> = info.colors.iter().map(|c| (c.key.as_str(), c.used)).collect();
        assert_eq!(used, vec![("R", true), ("B", true), ("G", false)]);
        assert_eq!(palette_info(&state, "nope").unwrap_err().code, "not_found");
        assert_eq!(palette_info(&ProjectState::default(), "main").unwrap_err().code, "no_project");
    }

    // Checks the "Change a color" scenario: a new value touches the palette
    // only (no sprite file is rewritten), and one project undo takes it back.
    #[test]
    fn a_new_value_touches_only_the_palette_and_undoes_in_one_step() {
        let (_dir, state) = open();
        let mut colors = same();
        colors[0].hex = "#d03030".to_string();
        let event = apply_edit(&state, &EditorState::default(), "main", &colors, &[]).unwrap();
        assert!(touches(&event, "palette", &["main"]));
        assert!(!touches(&event, "sprite", &["goblin"]));
        assert_eq!(palette_info(&state, "main").unwrap().colors[0].hex, "#D03030");
        browser::undo(&state).unwrap();
        assert_eq!(palette_info(&state, "main").unwrap().colors[0].hex, "#F04545");
    }

    // Checks the "Rename the key R to H" scenario: the sprite is rewritten
    // and touched, and one undo gives back the old key and the old pixels.
    #[test]
    fn a_new_key_rewrites_the_sprites_and_undoes_in_one_step() {
        let (_dir, state) = open();
        let mut colors = same();
        colors[0].key = "H".to_string();
        let event = apply_edit(&state, &EditorState::default(), "main", &colors, &[]).unwrap();
        assert!(touches(&event, "sprite", &["goblin"]));
        assert_eq!(goblin_pixels(&state), "HB..");
        browser::undo(&state).unwrap();
        assert_eq!(goblin_pixels(&state), "RB..");
        assert_eq!(palette_info(&state, "main").unwrap().colors[0].key, "R");
    }

    // Checks the "Remove a used color" scenario: without a replacement it is
    // refused and nothing changes; with one, the pixels take it.
    #[test]
    fn a_used_color_needs_a_replacement() {
        let (_dir, state) = open();
        let without_blue = vec![same()[0].clone(), same()[2].clone()];
        let error = apply_edit(&state, &EditorState::default(), "main", &without_blue, &[]).unwrap_err();
        assert_eq!(error.code, "replacement_needed");
        assert_eq!(goblin_pixels(&state), "RB..");
        let replacement = Replacement { key: "B".to_string(), with: "G".to_string() };
        apply_edit(&state, &EditorState::default(), "main", &without_blue, &[replacement]).unwrap();
        assert_eq!(goblin_pixels(&state), "RG..");
        assert_eq!(palette_info(&state, "main").unwrap().colors.len(), 2);
    }

    // Checks that the errors of the key rules reach the UI with their code,
    // and that a bad key text (empty, or two characters) is `bad_key`.
    #[test]
    fn key_errors_have_a_code() {
        let (_dir, state) = open();
        let code = |key: &str| {
            let mut colors = same();
            colors[0].key = key.to_string();
            apply_edit(&state, &EditorState::default(), "main", &colors, &[]).unwrap_err().code
        };
        assert_eq!(code("."), "reserved_key");
        assert_eq!(code(" "), "reserved_key");
        assert_eq!(code("B"), "duplicate_key");
        assert_eq!(code(""), "bad_key");
        assert_eq!(code("HH"), "bad_key");
        assert_eq!(goblin_pixels(&state), "RB..");
    }

    // Checks that a color added in the editor joins the palette.
    #[test]
    fn a_new_color_is_added() {
        let (_dir, state) = open();
        let mut colors = same();
        colors.push(arg(None, "Y", "yellow", "#F5C542"));
        apply_edit(&state, &EditorState::default(), "main", &colors, &[]).unwrap();
        assert_eq!(palette_info(&state, "main").unwrap().colors.len(), 4);
    }

    fn writing(frame: u32, sprite: &str) -> EditorState {
        let agents = EditorState::default();
        agents.begin(crate::ipc::editor::AgentWrite {
            agent: "Claude Code".to_string(),
            sprite: crate::ipc::editor::SpriteArg { folder: vec![], name: sprite.to_string() },
            frame,
        });
        agents
    }

    // Checks the "Agent is drawing" scenario: while an agent writes to a
    // sprite that uses the palette, a key change and a removal that rewrites
    // pixels are refused with `agent_writing` and nothing changes; when the
    // agent ends, the same edit is applied.
    #[test]
    fn key_changes_wait_for_the_agent() {
        let (_dir, state) = open();
        let agents = writing(1, "goblin");
        let mut renamed = same();
        renamed[0].key = "H".to_string();
        let error = apply_edit(&state, &agents, "main", &renamed, &[]).unwrap_err();
        assert_eq!(error.code, "agent_writing");
        assert_eq!(goblin_pixels(&state), "RB..");
        assert_eq!(palette_info(&state, "main").unwrap().colors[0].key, "R");

        let without_blue = vec![same()[0].clone(), same()[2].clone()];
        let replacement = Replacement { key: "B".to_string(), with: "G".to_string() };
        let error = apply_edit(&state, &agents, "main", &without_blue, &[replacement]).unwrap_err();
        assert_eq!(error.code, "agent_writing");

        agents.end();
        apply_edit(&state, &agents, "main", &renamed, &[]).unwrap();
        assert_eq!(goblin_pixels(&state), "HB..");
    }

    // Checks that color values, names and new colors are still applied while
    // an agent writes, and that an agent on a sprite of another palette does
    // not block key changes of this one.
    #[test]
    fn values_do_not_wait_and_other_palettes_do_not_block() {
        let (_dir, state) = open();
        let agents = writing(1, "goblin");
        let mut colors = same();
        colors[0].hex = "#D03030".to_string();
        colors[1].name = "sky".to_string();
        colors.push(arg(None, "Y", "yellow", "#F5C542"));
        apply_edit(&state, &agents, "main", &colors, &[]).unwrap();
        assert_eq!(palette_info(&state, "main").unwrap().colors.len(), 4);

        // An agent writing to a sprite that is not in the project does not
        // block this palette.
        let elsewhere = writing(1, "other");
        let mut renamed = vec![colors[0].clone(), colors[1].clone(), colors[2].clone(), colors[3].clone()];
        renamed[0].key = "H".to_string();
        apply_edit(&state, &elsewhere, "main", &renamed, &[]).unwrap();
    }

    // Checks the wire format the UI reads.
    #[test]
    fn the_info_serializes_for_the_ui() {
        let (_dir, state) = open();
        let json = serde_json::to_value(palette_info(&state, "main").unwrap()).unwrap();
        assert_eq!(
            json["colors"][0],
            serde_json::json!({"key": "R", "name": "red", "hex": "#F04545", "used": true})
        );
    }
}
