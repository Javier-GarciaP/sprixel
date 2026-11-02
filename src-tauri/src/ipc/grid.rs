//! What the Text view (and later the MCP tool `apply_grid`) asks of the core:
//! check a text grid and apply it to a sprite. The functions do not need a
//! window; the commands at the end add the `project://changed` event.

use super::browser::{ChangedEvent, CoreError, TargetInfo};
use super::editor::{EditorState, SpriteArg};
use super::project::{self, ProjectState};
use crate::document::{DocError, GridError, GridOptions, Operation, SpriteRef};
use serde::{Deserialize, Serialize};

/// Where the grid goes and how: the `x` and `y` of a region (both or none)
/// and whether `.` keeps the existing pixels.
#[derive(Debug, Clone, Copy, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GridArgs {
    pub x: Option<u32>,
    pub y: Option<u32>,
    #[serde(default)]
    pub skip_transparent: bool,
}

impl GridArgs {
    fn options(&self) -> GridOptions {
        // A region needs a place: with only one of the two, the other is 0.
        let region = match (self.x, self.y) {
            (None, None) => None,
            (x, y) => Some((x.unwrap_or(0), y.unwrap_or(0))),
        };
        GridOptions { region, skip_transparent: self.skip_transparent }
    }
}

/// Why a grid was not applied: a `code` and a message like every core error,
/// and the list of problems of the text when the grid is not valid
/// (`invalid_grid`).
#[derive(Debug, Serialize, PartialEq)]
pub struct GridFailure {
    pub code: String,
    pub message: String,
    pub errors: Vec<GridError>,
}

impl From<CoreError> for GridFailure {
    fn from(error: CoreError) -> Self {
        GridFailure { code: error.code, message: error.message, errors: Vec::new() }
    }
}

impl From<DocError> for GridFailure {
    fn from(error: DocError) -> Self {
        CoreError::from(error).into()
    }
}

impl GridFailure {
    fn invalid(errors: Vec<GridError>) -> GridFailure {
        // The first error is the message; an AI gets all of them in `errors`.
        let message = match errors.as_slice() {
            [only] => only.message.clone(),
            [first, rest @ ..] => format!("{} (and {} more)", first.message, rest.len()),
            [] => "the grid is not valid".to_string(),
        };
        let code = errors.first().map_or("invalid_grid", |e| e.code).to_string();
        GridFailure { code, message, errors }
    }
}

fn sprite_ref(sprite: &SpriteArg) -> SpriteRef {
    SpriteRef { folder: sprite.folder.clone(), name: sprite.name.clone() }
}

/// Applies a text grid to a sprite as one history entry by `author`, with the
/// colors that its legend adds. All or nothing: a grid with any error changes
/// nothing, and the errors are all in the failure. A frame an agent is writing
/// to refuses it with `frame_busy`. A grid that changes nothing makes no
/// history entry.
pub fn apply_grid(
    state: &ProjectState,
    agents: &EditorState,
    author: &str,
    sprite: &SpriteArg,
    text: &str,
    args: &GridArgs,
) -> Result<ChangedEvent, GridFailure> {
    let target = sprite_ref(sprite);
    let options = args.options();
    project::edit(state, |session| {
        let ops = session
            .project
            .grid_operations(&target, text, &options)
            .map_err(GridFailure::invalid)?;
        let busy = ops.iter().any(|op| {
            matches!(op, Operation::SetPixels { frame, .. } if agents.blocks(sprite, *frame))
        });
        if busy {
            return Err(DocError::new("frame_busy", "an agent is drawing on this frame").into());
        }
        let touched = if ops.is_empty() { Vec::new() } else { session.apply(author, ops)? };
        Ok(ChangedEvent {
            revision: session.project.revision,
            touched: touched.iter().map(TargetInfo::from).collect(),
        })
    })
    .unwrap_or_else(|| Err(CoreError::no_project().into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::{Color, Palette};
    use crate::ipc::browser;
    use crate::ipc::editor::AgentWrite;

    const USER: &str = crate::history::USER;

    fn color(key: char, name: &str, hex: &str) -> Color {
        Color { key, name: name.to_string(), hex: hex.to_string() }
    }

    fn sprite() -> SpriteArg {
        SpriteArg { folder: vec![], name: "goblin".to_string() }
    }

    /// An open project with the palette `main` (R and B) and a 2x2 sprite
    /// `goblin` of transparent pixels. The setup is not in the history.
    fn open() -> (tempfile::TempDir, ProjectState, EditorState) {
        let parent = tempfile::tempdir().unwrap();
        let state = ProjectState::default();
        project::create_project(&state, parent.path(), "Kit").unwrap();
        project::edit(&state, |session| {
            let ops = vec![
                Operation::CreatePalette {
                    palette: Palette {
                        name: "main".to_string(),
                        colors: vec![color('R', "red", "#F04545"), color('B', "blue", "#2F6BFF")],
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
            ];
            session.apply(USER, ops).unwrap();
            session.history.clear();
        })
        .unwrap();
        (parent, state, EditorState::default())
    }

    fn apply(state: &ProjectState, agents: &EditorState, text: &str) -> Result<ChangedEvent, GridFailure> {
        apply_grid(state, agents, USER, &sprite(), text, &GridArgs::default())
    }

    // What the project has now: the pixels of frame 1, the number of colors
    // of the palette, the revision and the number of history entries.
    fn look(state: &ProjectState) -> (String, usize, u64, usize) {
        project::read(state, |session| {
            let found = session.project.find_sprite(&sprite_ref(&sprite())).unwrap();
            (
                String::from_utf8(found.layers[0].frames[0].pixels.clone()).unwrap(),
                session.project.palette("main").unwrap().colors.len(),
                session.project.revision,
                session.history.entries().len(),
            )
        })
        .unwrap()
    }

    // Checks "Undo an applied grid": the pixels and the new legend color are
    // one history entry, and one undo gives back the frame and the palette.
    #[test]
    fn an_applied_grid_is_one_undo_step() {
        let (_dir, state, agents) = open();
        let event = apply(&state, &agents, "legend\n  Q teal #1FB5A8\nframe 1\nRQ\nBR\n").unwrap();
        assert_eq!(look(&state), ("RQBR".to_string(), 3, 2, 1));
        assert_eq!(event.revision, 2);
        browser::undo(&state).unwrap();
        // Colors and pixels came back together, in one step.
        assert_eq!(look(&state), ("....".to_string(), 2, 3, 0));
        browser::redo(&state).unwrap();
        assert_eq!(look(&state).0, "RQBR");
    }

    // Checks "Several errors" and "no partial results": a grid with a good
    // legend color and two bad rows reports both errors and changes nothing,
    // not the pixels, not the palette, not the revision or the history.
    #[test]
    fn a_grid_with_errors_changes_nothing() {
        let (_dir, state, agents) = open();
        let before = look(&state);
        let failure = apply(&state, &agents, "legend\n  Q teal #1FB5A8\nframe 1\nQ\nBX\n").unwrap_err();
        let codes: Vec<&str> = failure.errors.iter().map(|e| e.code).collect();
        assert_eq!(codes, vec!["row_length", "unknown_key"]);
        assert_eq!(failure.code, "row_length");
        assert!(failure.message.contains("and 1 more"), "{}", failure.message);
        assert_eq!(look(&state), before);
    }

    // Checks that the history entry has the author that applied the grid,
    // so an AI call is labeled and a person's is "user".
    #[test]
    fn the_history_entry_has_the_author() {
        let (_dir, state, agents) = open();
        apply_grid(&state, &agents, "claude-code", &sprite(), "frame 1\nRR\nRR\n", &GridArgs::default()).unwrap();
        let author = project::read(&state, |s| s.history.entries()[0].author.clone()).unwrap();
        assert_eq!(author, "claude-code");
    }

    // Checks that a grid that changes nothing makes no history entry, so the
    // Undo button is not used up by it.
    #[test]
    fn a_grid_with_no_change_makes_no_history_entry() {
        let (_dir, state, agents) = open();
        apply(&state, &agents, "frame 1\n..\n..\n").unwrap();
        assert_eq!(look(&state), ("....".to_string(), 2, 1, 0));
    }

    // Checks that a region is written at its place and `skip_transparent`
    // keeps the pixels under a ".".
    #[test]
    fn a_region_is_written_at_its_place() {
        let (_dir, state, agents) = open();
        apply(&state, &agents, "frame 1\nRR\nRR\n").unwrap();
        let args = GridArgs { x: Some(1), y: Some(1), skip_transparent: false };
        apply_grid(&state, &agents, USER, &sprite(), "frame 1\nB\n", &args).unwrap();
        assert_eq!(look(&state).0, "RRRB");
        let args = GridArgs { x: Some(0), y: Some(0), skip_transparent: true };
        apply_grid(&state, &agents, USER, &sprite(), "frame 1\n.B\n", &args).unwrap();
        assert_eq!(look(&state).0, "RBRB");
        let out = GridArgs { x: Some(1), y: Some(0), skip_transparent: false };
        let failure = apply_grid(&state, &agents, USER, &sprite(), "frame 1\nRR\n", &out).unwrap_err();
        assert_eq!(failure.code, "region_out");
    }

    // Checks that a frame an agent is drawing on refuses the grid, like a
    // stroke, and nothing changes.
    #[test]
    fn a_busy_frame_refuses_the_grid() {
        let (_dir, state, agents) = open();
        agents.begin(AgentWrite { agent: "claude-code".to_string(), sprite: sprite(), frame: 1 });
        let failure = apply(&state, &agents, "frame 1\nRR\nRR\n").unwrap_err();
        assert_eq!(failure.code, "frame_busy");
        assert_eq!(look(&state).0, "....");
    }

    // Checks the errors that are not about the text: no open project and a
    // sprite that does not exist.
    #[test]
    fn no_project_or_no_sprite_is_an_error() {
        let agents = EditorState::default();
        let closed = ProjectState::default();
        assert_eq!(apply(&closed, &agents, "frame 1\n.\n").unwrap_err().code, "no_project");
        let (_dir, state, agents) = open();
        let nobody = SpriteArg { folder: vec![], name: "nobody".to_string() };
        let failure = apply_grid(&state, &agents, USER, &nobody, "frame 1\n.\n", &GridArgs::default()).unwrap_err();
        assert_eq!(failure.code, "not_found");
    }
}
