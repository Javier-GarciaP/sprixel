//! What the editor asks of the core: the size, palette and pixels of a sprite,
//! strokes, undo and redo, and the state of an AI agent that is writing. The
//! core owns the document; the UI only reads it and sends strokes. The
//! functions here do not need a window, so the tests can call them directly.

use super::project::{self, ProjectState};
use crate::document::{ColorChange, DocError, Project, SpriteRef};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

/// The history author of everything the editor sends.
const USER: &str = crate::history::USER;

/// A sprite as the UI names it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpriteArg {
    pub folder: Vec<String>,
    pub name: String,
}

impl SpriteArg {
    fn to_ref(&self) -> SpriteRef {
        SpriteRef {
            folder: self.folder.clone(),
            name: self.name.clone(),
        }
    }
}

/// One pixel of a stroke: `color` is `#RRGGBB`, or `null` for transparent.
#[derive(Debug, Clone, Deserialize)]
pub struct Change {
    pub x: u32,
    pub y: u32,
    pub color: Option<String>,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct PaletteColor {
    pub key: String,
    pub hex: String,
}

#[derive(Debug, Serialize, PartialEq, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub struct HistoryState {
    pub can_undo: bool,
    pub can_redo: bool,
    pub revision: u64,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SpriteInfo {
    pub width: u32,
    pub height: u32,
    pub frame_count: usize,
    pub palette: Vec<PaletteColor>,
    #[serde(flatten)]
    pub history: HistoryState,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct ApplyResult {
    pub palette: Vec<PaletteColor>,
    #[serde(flatten)]
    pub history: HistoryState,
}

/// Errors cross to the UI as a stable `code` and a message. Besides the codes
/// of the document (`palette_full`, `bad_frame`, `not_found`...) there are
/// `no_project` and `frame_busy`.
#[derive(Debug, Serialize, PartialEq)]
pub struct EditorError {
    pub code: String,
    pub message: String,
}

impl EditorError {
    fn new(code: &str, message: &str) -> EditorError {
        EditorError {
            code: code.to_string(),
            message: message.to_string(),
        }
    }
}

impl From<DocError> for EditorError {
    fn from(error: DocError) -> Self {
        EditorError {
            code: error.code.to_string(),
            message: error.message,
        }
    }
}

fn no_project() -> EditorError {
    EditorError::new("no_project", "no project is open")
}

/// An AI agent writing to a frame. While it lasts the frame is busy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentWrite {
    pub agent: String,
    pub sprite: SpriteArg,
    pub frame: u32,
}

/// The event the UI listens to. The payload is an `AgentWrite`, or `null` when
/// the agent ends.
pub const AGENT_EVENT: &str = "agent-activity";

/// Who is writing now and whether the user asked to stop. The MCP server will
/// call `begin` and `end` around each write and look at `stop_requested`.
#[derive(Default)]
pub struct EditorState(Mutex<Agents>);

#[derive(Default)]
struct Agents {
    writing: Option<AgentWrite>,
    stop_requested: bool,
}

impl EditorState {
    pub fn begin(&self, write: AgentWrite) {
        let mut agents = self.0.lock().unwrap();
        agents.stop_requested = false;
        agents.writing = Some(write);
    }

    pub fn end(&self) {
        self.0.lock().unwrap().writing = None;
    }

    /// The Stop button: the frame is free at once and the agent must stop.
    pub fn stop(&self) {
        let mut agents = self.0.lock().unwrap();
        agents.writing = None;
        agents.stop_requested = true;
    }

    // The MCP server reads it; nothing in the app does yet.
    #[allow(dead_code)]
    pub fn stop_requested(&self) -> bool {
        self.0.lock().unwrap().stop_requested
    }

    pub fn writing(&self) -> Option<AgentWrite> {
        self.0.lock().unwrap().writing.clone()
    }

    fn blocks(&self, sprite: &SpriteArg, frame: u32) -> bool {
        self.writing()
            .is_some_and(|w| w.frame == frame && same_sprite(&w.sprite, sprite))
    }
}

// The same sprite, ignoring the case of the names like the document does.
fn same_sprite(a: &SpriteArg, b: &SpriteArg) -> bool {
    a.folder.len() == b.folder.len()
        && a.folder.iter().zip(&b.folder).all(|(x, y)| x.eq_ignore_ascii_case(y))
        && a.name.eq_ignore_ascii_case(&b.name)
}

fn history_of(session: &crate::storage::Session) -> HistoryState {
    HistoryState {
        can_undo: !session.history.entries().is_empty(),
        can_redo: session.history.can_redo(),
        revision: session.project.revision,
    }
}

fn palette_of(project: &Project, sprite: &SpriteRef) -> Vec<PaletteColor> {
    project
        .find_sprite(sprite)
        .and_then(|s| project.palette(&s.palette))
        .map(|p| {
            p.colors
                .iter()
                .map(|c| PaletteColor {
                    key: c.key.to_string(),
                    hex: c.hex.clone(),
                })
                .collect()
        })
        .unwrap_or_default()
}

fn not_found(sprite: &SpriteArg) -> EditorError {
    EditorError::new("not_found", &format!("sprite {:?} does not exist", sprite.name))
}

pub fn sprite_info(state: &ProjectState, sprite: &SpriteArg) -> Result<SpriteInfo, EditorError> {
    project::read(state, |session| {
        let found = session
            .project
            .find_sprite(&sprite.to_ref())
            .ok_or_else(|| not_found(sprite))?;
        Ok(SpriteInfo {
            width: found.width,
            height: found.height,
            frame_count: found.frame_count(),
            palette: palette_of(&session.project, &sprite.to_ref()),
            history: history_of(session),
        })
    })
    .ok_or_else(no_project)?
}

/// The pixels of a frame (frames start at 1): one palette key for each pixel,
/// row by row.
pub fn frame_pixels(
    state: &ProjectState,
    sprite: &SpriteArg,
    frame: u32,
) -> Result<String, EditorError> {
    project::read(state, |session| {
        let found = session
            .project
            .find_sprite(&sprite.to_ref())
            .ok_or_else(|| not_found(sprite))?;
        let layer = found.layers.first().expect("a sprite has a layer");
        let pixels = frame
            .checked_sub(1)
            .and_then(|i| layer.frames.get(i as usize))
            .ok_or_else(|| EditorError::new("bad_frame", &format!("frame {frame} does not exist")))?;
        Ok(String::from_utf8_lossy(&pixels.pixels).into_owned())
    })
    .ok_or_else(no_project)?
}

/// One stroke, fill, shape or move: the changed pixels become one history
/// entry by the user, with the new colors they need added to the palette. A
/// frame an agent is writing to refuses it with `frame_busy`.
pub fn apply_changes(
    state: &ProjectState,
    agents: &EditorState,
    sprite: &SpriteArg,
    frame: u32,
    changes: &[Change],
) -> Result<ApplyResult, EditorError> {
    if agents.blocks(sprite, frame) {
        return Err(EditorError::new("frame_busy", "an agent is drawing on this frame"));
    }
    let changes: Vec<ColorChange> = changes
        .iter()
        .map(|c| ColorChange {
            x: c.x,
            y: c.y,
            color: c.color.clone(),
        })
        .collect();
    project::edit(state, |session| {
        let target = sprite.to_ref();
        let ops = session.project.stroke_operations(&target, 0, frame, &changes)?;
        session.apply(USER, ops)?;
        Ok(ApplyResult {
            palette: palette_of(&session.project, &target),
            history: history_of(session),
        })
    })
    .ok_or_else(no_project)?
}

pub fn undo(state: &ProjectState) -> Result<HistoryState, EditorError> {
    project::edit(state, |session| {
        session.undo()?;
        Ok(history_of(session))
    })
    .ok_or_else(no_project)?
}

pub fn redo(state: &ProjectState) -> Result<HistoryState, EditorError> {
    project::edit(state, |session| {
        session.redo()?;
        Ok(history_of(session))
    })
    .ok_or_else(no_project)?
}

#[tauri::command]
pub fn editor_sprite(
    state: tauri::State<ProjectState>,
    sprite: SpriteArg,
) -> Result<SpriteInfo, EditorError> {
    sprite_info(&state, &sprite)
}

#[tauri::command]
pub fn editor_frame(
    state: tauri::State<ProjectState>,
    sprite: SpriteArg,
    frame: u32,
) -> Result<String, EditorError> {
    frame_pixels(&state, &sprite, frame)
}

#[tauri::command]
pub fn editor_apply(
    state: tauri::State<ProjectState>,
    agents: tauri::State<EditorState>,
    sprite: SpriteArg,
    frame: u32,
    changes: Vec<Change>,
) -> Result<ApplyResult, EditorError> {
    apply_changes(&state, &agents, &sprite, frame, &changes)
}

#[tauri::command]
pub fn editor_undo(state: tauri::State<ProjectState>) -> Result<HistoryState, EditorError> {
    undo(&state)
}

#[tauri::command]
pub fn editor_redo(state: tauri::State<ProjectState>) -> Result<HistoryState, EditorError> {
    redo(&state)
}

/// The agent that is writing now, if any (read when the editor opens).
#[tauri::command]
pub fn agent_activity(agents: tauri::State<EditorState>) -> Option<AgentWrite> {
    agents.writing()
}

/// The Stop button.
#[tauri::command]
pub fn agent_stop(app: tauri::AppHandle, agents: tauri::State<EditorState>) {
    agents.stop();
    let _ = tauri::Emitter::emit(&app, AGENT_EVENT, Option::<AgentWrite>::None);
}

/// Development only: pretends an agent writes to a frame for a while, so the
/// marker, the busy frame and Stop can be tried without the MCP server. Call
/// it from the developer console of `npm run tauri dev`.
#[tauri::command]
pub fn debug_agent_write(
    app: tauri::AppHandle,
    agents: tauri::State<EditorState>,
    agent: String,
    sprite: SpriteArg,
    frame: u32,
    milliseconds: u64,
) -> Result<(), EditorError> {
    if !cfg!(debug_assertions) {
        return Err(EditorError::new("not_available", "only in development builds"));
    }
    let write = AgentWrite { agent, sprite, frame };
    agents.begin(write.clone());
    let _ = tauri::Emitter::emit(&app, AGENT_EVENT, Some(&write));
    let handle = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(milliseconds));
        let agents = tauri::Manager::state::<EditorState>(&handle);
        // Stop may have ended it already, and another write may have begun.
        if agents.writing().as_ref() == Some(&write) {
            agents.end();
            let _ = tauri::Emitter::emit(&handle, AGENT_EVENT, Option::<AgentWrite>::None);
        }
    });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::{Color, Operation, Palette};
    use crate::storage::Session;
    use std::path::Path;

    fn arg(name: &str) -> SpriteArg {
        SpriteArg {
            folder: vec![],
            name: name.to_string(),
        }
    }

    fn change(x: u32, y: u32, color: Option<&str>) -> Change {
        Change {
            x,
            y,
            color: color.map(str::to_string),
        }
    }

    /// An open project with the palette `main` (empty or with `colors`) and a
    /// 16x16 sprite `goblin`. Returns the folder of the project too.
    fn open_project(colors: Vec<Color>) -> (tempfile::TempDir, ProjectState, EditorState) {
        let parent = tempfile::tempdir().unwrap();
        let state = ProjectState::default();
        project::create_project(&state, parent.path(), "Kit").unwrap();
        project::edit(&state, |session| {
            session
                .apply(
                    USER,
                    vec![
                        Operation::CreatePalette {
                            palette: Palette {
                                name: "main".to_string(),
                                colors,
                            },
                            index: None,
                        },
                        Operation::CreateSprite {
                            folder: vec![],
                            name: "goblin".to_string(),
                            width: 16,
                            height: 16,
                            palette: "main".to_string(),
                        },
                    ],
                )
                .unwrap();
            // The setup is not something the user can undo.
            session.history.clear();
        })
        .unwrap();
        (parent, state, EditorState::default())
    }

    fn frame(state: &ProjectState) -> String {
        frame_pixels(state, &arg("goblin"), 1).unwrap()
    }

    // Checks that the editor reads the size, the frames and the palette of a
    // sprite, and a frame as one key per pixel.
    #[test]
    fn reads_a_sprite_and_a_frame() {
        let (_dir, state, _agents) = open_project(vec![]);
        let info = sprite_info(&state, &arg("goblin")).unwrap();
        assert_eq!((info.width, info.height, info.frame_count), (16, 16, 1));
        assert!(info.palette.is_empty());
        assert!(!info.history.can_undo && !info.history.can_redo);
        assert_eq!(frame(&state), ".".repeat(256));
    }

    // Checks the errors the UI translates: no project, a sprite that does not
    // exist and a frame that does not exist.
    #[test]
    fn reading_fails_with_clear_codes() {
        let state = ProjectState::default();
        assert_eq!(sprite_info(&state, &arg("goblin")).unwrap_err().code, "no_project");

        let (_dir, state, _agents) = open_project(vec![]);
        assert_eq!(sprite_info(&state, &arg("ghost")).unwrap_err().code, "not_found");
        assert_eq!(frame_pixels(&state, &arg("goblin"), 0).unwrap_err().code, "bad_frame");
        assert_eq!(frame_pixels(&state, &arg("goblin"), 2).unwrap_err().code, "bad_frame");
    }

    // Checks the "Undo a stroke" scenario in the core: a stroke of 30 pixels
    // with a new color is one history entry, so one undo gives back all 30
    // pixels and removes the new color, and redo brings both back.
    #[test]
    fn a_stroke_of_30_pixels_undoes_in_one_step() {
        let (_dir, state, agents) = open_project(vec![]);
        let changes: Vec<Change> = (0..30)
            .map(|i| change(i % 16, i / 16, Some("#336699")))
            .collect();

        let applied = apply_changes(&state, &agents, &arg("goblin"), 1, &changes).unwrap();
        assert_eq!(applied.palette.len(), 1);
        assert_eq!(applied.palette[0].key, "A");
        assert_eq!(applied.palette[0].hex, "#336699");
        assert!(applied.history.can_undo);
        assert_eq!(frame(&state).chars().filter(|c| *c == 'A').count(), 30);

        let after_undo = undo(&state).unwrap();
        assert_eq!(frame(&state), ".".repeat(256));
        assert!(sprite_info(&state, &arg("goblin")).unwrap().palette.is_empty());
        assert!(!after_undo.can_undo && after_undo.can_redo);

        let after_redo = redo(&state).unwrap();
        assert_eq!(frame(&state).chars().filter(|c| *c == 'A').count(), 30);
        assert!(after_redo.can_undo && !after_redo.can_redo);
    }

    // Checks that two strokes are two history entries, undone one at a time.
    #[test]
    fn each_stroke_is_its_own_undo_step() {
        let (_dir, state, agents) = open_project(vec![]);
        apply_changes(&state, &agents, &arg("goblin"), 1, &[change(0, 0, Some("#336699"))]).unwrap();
        apply_changes(&state, &agents, &arg("goblin"), 1, &[change(1, 0, Some("#336699"))]).unwrap();
        undo(&state).unwrap();
        assert!(frame(&state).starts_with("A."));
        undo(&state).unwrap();
        assert!(frame(&state).starts_with(".."));
        assert_eq!(undo(&state).unwrap_err().code, "nothing_to_undo");
    }

    // Checks that the history author of a stroke is the user.
    #[test]
    fn a_stroke_is_by_the_user() {
        let (_dir, state, agents) = open_project(vec![]);
        apply_changes(&state, &agents, &arg("goblin"), 1, &[change(0, 0, None)]).unwrap();
        // A stroke that changes nothing still is an action, by the user.
        let author = project::read(&state, |s| s.history.entries()[0].author.clone()).unwrap();
        assert_eq!(author, "user");
    }

    // Checks the "The palette is full" scenario in the core: a new color on a
    // palette with 93 colors fails with `palette_full` and nothing changes,
    // not even the history.
    #[test]
    fn a_full_palette_refuses_a_new_color_and_changes_nothing() {
        let (_dir, state, agents) = open_project(vec![]);
        let mut palette = Palette {
            name: "main".to_string(),
            colors: vec![],
        };
        while let Some(key) = palette.next_free_key() {
            palette.colors.push(Color {
                key,
                name: key.to_string(),
                hex: format!("#{:06X}", palette.colors.len() + 1),
            });
        }
        project::edit(&state, |session| {
            for color in palette.colors.clone() {
                session
                    .apply(
                        USER,
                        vec![Operation::AddColor {
                            palette: "main".to_string(),
                            color,
                            index: None,
                        }],
                    )
                    .unwrap();
            }
            session.history.clear();
        })
        .unwrap();
        let before = frame(&state);

        let error =
            apply_changes(&state, &agents, &arg("goblin"), 1, &[change(0, 0, Some("#FEDCBA"))])
                .unwrap_err();

        assert_eq!(error.code, "palette_full");
        assert_eq!(frame(&state), before);
        assert!(!sprite_info(&state, &arg("goblin")).unwrap().history.can_undo);
    }

    // Checks the busy frame: while an agent writes to frame 1 a stroke on it
    // is refused with `frame_busy` (also with other letter case), another
    // frame or sprite is not blocked, and the frame is free again when the
    // agent ends.
    #[test]
    fn a_busy_frame_refuses_strokes_until_the_agent_ends() {
        let (_dir, state, agents) = open_project(vec![]);
        agents.begin(AgentWrite {
            agent: "Claude Code".to_string(),
            sprite: arg("goblin"),
            frame: 1,
        });
        let draw = |name: &str, frame: u32| {
            apply_changes(&state, &agents, &arg(name), frame, &[change(0, 0, Some("#336699"))])
        };

        assert_eq!(draw("goblin", 1).unwrap_err().code, "frame_busy");
        assert_eq!(draw("GOBLIN", 1).unwrap_err().code, "frame_busy");
        assert_eq!(frame(&state), ".".repeat(256));
        // Frame 2 does not exist: the error is about that, not about being busy.
        assert_eq!(draw("goblin", 2).unwrap_err().code, "bad_frame");

        agents.end();
        assert!(draw("goblin", 1).is_ok());
    }

    // Checks the Stop button: the frame is free at once and the agent is told
    // to stop; a new write clears that request.
    #[test]
    fn stop_frees_the_frame_and_asks_the_agent_to_stop() {
        let (_dir, state, agents) = open_project(vec![]);
        let write = AgentWrite {
            agent: "OpenCode".to_string(),
            sprite: arg("goblin"),
            frame: 1,
        };
        agents.begin(write.clone());
        assert_eq!(agents.writing(), Some(write.clone()));
        assert!(!agents.stop_requested());

        agents.stop();

        assert_eq!(agents.writing(), None);
        assert!(agents.stop_requested());
        assert!(apply_changes(&state, &agents, &arg("goblin"), 1, &[change(0, 0, None)]).is_ok());
        agents.begin(write);
        assert!(!agents.stop_requested());
    }

    // Checks that a stroke reaches the disk: the palette file has the new
    // color and the sprite file the key, and the project opens again with
    // both (the palette is written before the sprite).
    #[test]
    fn a_stroke_and_its_new_color_are_saved() {
        let (_dir, state, agents) = open_project(vec![]);
        apply_changes(&state, &agents, &arg("goblin"), 1, &[change(2, 1, Some("#336699"))]).unwrap();
        let root = project::read(&state, |s| s.root().to_path_buf()).unwrap();

        project::close_project(&state);

        let reopened = Session::open(Path::new(&root)).unwrap();
        let palette = reopened.project.palette("main").unwrap();
        assert_eq!(palette.colors.len(), 1);
        assert_eq!(palette.colors[0].hex, "#336699");
        assert_eq!(palette.colors[0].name, "336699");
        let sprite = &reopened.project.root.sprites[0];
        assert_eq!(sprite.layers[0].frames[0].pixels[16 + 2], b'A');
    }

    // Checks the wire format the UI reads: camelCase fields, the history
    // flags next to the size, and errors as a code with a message.
    #[test]
    fn results_serialize_for_the_ui() {
        let (_dir, state, agents) = open_project(vec![]);
        let info = serde_json::to_value(sprite_info(&state, &arg("goblin")).unwrap()).unwrap();
        assert_eq!(info["width"], 16);
        assert_eq!(info["frameCount"], 1);
        assert_eq!(info["canUndo"], false);
        assert_eq!(info["canRedo"], false);
        assert!(info["revision"].is_u64());
        assert_eq!(info["palette"], serde_json::json!([]));

        let applied = apply_changes(&state, &agents, &arg("goblin"), 1, &[change(0, 0, Some("#336699"))])
            .unwrap();
        let applied = serde_json::to_value(applied).unwrap();
        assert_eq!(applied["palette"][0], serde_json::json!({"key": "A", "hex": "#336699"}));
        assert_eq!(applied["canUndo"], true);

        let error = serde_json::to_value(EditorError::new("palette_full", "full")).unwrap();
        assert_eq!(error, serde_json::json!({"code": "palette_full", "message": "full"}));

        let write = AgentWrite {
            agent: "Claude Code".to_string(),
            sprite: arg("goblin"),
            frame: 2,
        };
        let json = serde_json::to_value(&write).unwrap();
        assert_eq!(json["sprite"]["folder"], serde_json::json!([]));
        assert_eq!(json["frame"], 2);
    }
}
