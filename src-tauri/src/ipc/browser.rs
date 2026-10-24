//! What the Project tab asks of the core: the tree of the project, the pixels
//! of a frame and the changes a person makes (creating, renaming, moving and
//! deleting). The functions here do not need a window, so the tests call them
//! directly; the commands in `ipc/mod.rs` add the `project://changed` event.

use crate::document::{
    DocError, Folder, Operation, Palette, Sprite, SpriteRef, Target, copy_name, same_name,
    validate_name,
};
use crate::ipc::project::{self, ProjectState};
use crate::storage::Session;
use serde::{Deserialize, Serialize};

/// The event the core emits after every change to the open project.
pub const CHANGED_EVENT: &str = "project://changed";

/// An error with a stable `code` (`no_project`, `name_exists`, ...) for the UI.
#[derive(Debug, Serialize, PartialEq)]
pub struct CoreError {
    pub code: String,
    pub message: String,
}

impl CoreError {
    pub fn no_project() -> CoreError {
        CoreError {
            code: "no_project".to_string(),
            message: "no project is open".to_string(),
        }
    }
}

impl From<DocError> for CoreError {
    fn from(error: DocError) -> Self {
        CoreError {
            code: error.code.to_string(),
            message: error.message,
        }
    }
}

/// A part of the project, as the UI names it. A sprite or a folder has its
/// whole path, a palette has its name.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct TargetInfo {
    pub kind: &'static str,
    pub path: Vec<String>,
}

impl From<&Target> for TargetInfo {
    fn from(target: &Target) -> Self {
        match target {
            Target::Folder(path) => TargetInfo { kind: "folder", path: path.clone() },
            Target::Palette(name) => TargetInfo { kind: "palette", path: vec![name.clone()] },
            Target::Sprite(sprite) => {
                let mut path = sprite.folder.clone();
                path.push(sprite.name.clone());
                TargetInfo { kind: "sprite", path }
            }
        }
    }
}

/// The payload of `project://changed`: the new revision and what was touched.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ChangedEvent {
    pub revision: u64,
    pub touched: Vec<TargetInfo>,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TagInfo {
    pub name: String,
    pub from: u32,
    pub to: u32,
    pub fps: u32,
    pub mode: &'static str,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct EditorInfo {
    /// `user` or the name of an AI client.
    pub author: String,
    pub at_ms: u64,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SpriteSummary {
    pub name: String,
    pub folder: Vec<String>,
    pub width: u32,
    pub height: u32,
    pub frames: usize,
    pub palette: String,
    pub tags: Vec<TagInfo>,
    pub last_editor: Option<EditorInfo>,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FolderNode {
    pub name: String,
    pub path: Vec<String>,
    pub folders: Vec<FolderNode>,
    pub sprites: Vec<SpriteSummary>,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PaletteInfo {
    pub name: String,
    pub colors: usize,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TreeInfo {
    pub name: String,
    pub revision: u64,
    /// The project root. Its own name is not used.
    pub root: FolderNode,
    pub palettes: Vec<PaletteInfo>,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct ColorInfo {
    pub key: String,
    pub hex: String,
}

/// One frame of a sprite, ready to draw: rows of palette keys and the colors.
#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FrameInfo {
    pub revision: u64,
    pub width: u32,
    pub height: u32,
    pub rows: Vec<String>,
    pub colors: Vec<ColorInfo>,
}

/// Where a sprite is, as the UI names it.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SpriteTarget {
    pub folder: Vec<String>,
    pub name: String,
}

/// The palette a sprite gets when the person chooses none.
const DEFAULT_PALETTE: &str = "main";

fn read<R>(state: &ProjectState, look: impl FnOnce(&Session) -> R) -> Result<R, CoreError> {
    project::read(state, look).ok_or_else(CoreError::no_project)
}

/// Runs a change on the open project and reports it. The closure returns a
/// value for the caller and the parts it touched.
pub fn change<T>(
    state: &ProjectState,
    run: impl FnOnce(&mut Session) -> Result<(T, Vec<Target>), DocError>,
) -> Result<(T, ChangedEvent), CoreError> {
    project::edit(state, |session| {
        let (value, touched) = run(session)?;
        let event = ChangedEvent {
            revision: session.project.revision,
            touched: touched.iter().map(TargetInfo::from).collect(),
        };
        Ok((value, event))
    })
    .ok_or_else(CoreError::no_project)?
}

fn folder_node(session: &Session, folder: &Folder, path: Vec<String>) -> FolderNode {
    FolderNode {
        name: folder.name.clone(),
        folders: folder
            .folders
            .iter()
            .map(|child| {
                let mut child_path = path.clone();
                child_path.push(child.name.clone());
                folder_node(session, child, child_path)
            })
            .collect(),
        sprites: folder
            .sprites
            .iter()
            .map(|sprite| sprite_summary(session, sprite, &path))
            .collect(),
        path,
    }
}

fn sprite_summary(session: &Session, sprite: &Sprite, folder: &[String]) -> SpriteSummary {
    let sprite_ref = SpriteRef {
        folder: folder.to_vec(),
        name: sprite.name.clone(),
    };
    SpriteSummary {
        name: sprite.name.clone(),
        folder: folder.to_vec(),
        width: sprite.width,
        height: sprite.height,
        frames: sprite.frame_count(),
        palette: sprite.palette.clone(),
        tags: sprite
            .tags
            .iter()
            .map(|tag| TagInfo {
                name: tag.name.clone(),
                from: tag.from,
                to: tag.to,
                fps: tag.fps,
                mode: tag.mode.as_str(),
            })
            .collect(),
        last_editor: session.history.last_editor(&sprite_ref).map(|editor| EditorInfo {
            author: editor.author,
            at_ms: editor.at_ms,
        }),
    }
}

/// The folders, sprites and palettes of the open project.
pub fn project_tree(state: &ProjectState) -> Result<TreeInfo, CoreError> {
    read(state, |session| {
        let project = &session.project;
        TreeInfo {
            name: project.name.clone(),
            revision: project.revision,
            root: folder_node(session, &project.root, Vec::new()),
            palettes: project
                .palettes
                .iter()
                .map(|p| PaletteInfo {
                    name: p.name.clone(),
                    colors: p.colors.len(),
                })
                .collect(),
        }
    })
}

/// One frame (numbered from 1) of a sprite. Version 1 shows one layer, so
/// this is the first layer.
pub fn sprite_frame(
    state: &ProjectState,
    folder: &[String],
    name: &str,
    frame: u32,
) -> Result<FrameInfo, CoreError> {
    read(state, |session| {
        let project = &session.project;
        let sprite_ref = SpriteRef {
            folder: folder.to_vec(),
            name: name.to_string(),
        };
        let sprite = project.find_sprite(&sprite_ref).ok_or_else(|| {
            DocError::new("not_found", format!("sprite {name:?} does not exist"))
        })?;
        let pixels = &sprite
            .layers
            .first()
            .and_then(|layer| layer.frames.get((frame as usize).wrapping_sub(1)))
            .ok_or_else(|| {
                DocError::new(
                    "bad_frame",
                    format!("frame {frame} does not exist, the sprite has {} frames", sprite.frame_count()),
                )
            })?
            .pixels;
        let rows = pixels
            .chunks(sprite.width as usize)
            .map(|row| row.iter().map(|&key| key as char).collect())
            .collect();
        let colors = project
            .palette(&sprite.palette)
            .map(|palette| {
                palette
                    .colors
                    .iter()
                    .map(|c| ColorInfo {
                        key: c.key.to_string(),
                        hex: c.hex.clone(),
                    })
                    .collect()
            })
            .unwrap_or_default();
        Ok::<_, DocError>(FrameInfo {
            revision: project.revision,
            width: sprite.width,
            height: sprite.height,
            rows,
            colors,
        })
    })?
    .map_err(CoreError::from)
}

/// Checks a name for a new or renamed sprite or folder inside `parent`. The
/// answer is `None` when the name is fine, or a stable code: `empty`,
/// `invalid_name`, `folder_exists` or `sprite_exists`. `current` is the name
/// of the item being renamed, which does not clash with itself.
pub fn check_name(
    state: &ProjectState,
    parent: &[String],
    name: &str,
    current: Option<&str>,
) -> Result<Option<&'static str>, CoreError> {
    read(state, |session| {
        if name.trim().is_empty() {
            return Some("empty");
        }
        if validate_name(name).is_err() {
            return Some("invalid_name");
        }
        let folder = session.project.find_folder(parent)?;
        let is_other = |item: &str| !current.is_some_and(|c| same_name(c, item));
        if folder
            .folders
            .iter()
            .any(|f| same_name(&f.name, name) && is_other(&f.name))
        {
            return Some("folder_exists");
        }
        if folder
            .sprites
            .iter()
            .any(|s| same_name(&s.name, name) && is_other(&s.name))
        {
            return Some("sprite_exists");
        }
        None
    })
}

/// Creates a sprite. With no palette the sprite uses `main`, which is created
/// empty first when the project has none. Everything is one history entry.
pub fn create_sprite(
    state: &ProjectState,
    folder: &[String],
    name: &str,
    width: u32,
    height: u32,
    palette: Option<&str>,
) -> Result<(SpriteTarget, ChangedEvent), CoreError> {
    change(state, |session| {
        let mut ops = Vec::new();
        let palette = match palette {
            Some(chosen) => chosen.to_string(),
            None => {
                if session.project.palette(DEFAULT_PALETTE).is_none() {
                    ops.push(Operation::CreatePalette {
                        palette: Palette {
                            name: DEFAULT_PALETTE.to_string(),
                            colors: Vec::new(),
                        },
                        index: None,
                    });
                }
                DEFAULT_PALETTE.to_string()
            }
        };
        ops.push(Operation::CreateSprite {
            folder: folder.to_vec(),
            name: name.to_string(),
            width,
            height,
            palette,
        });
        let touched = session.apply("user", ops)?;
        let target = touched
            .iter()
            .find_map(|t| match t {
                Target::Sprite(sprite) => Some(SpriteTarget {
                    folder: sprite.folder.clone(),
                    name: sprite.name.clone(),
                }),
                _ => None,
            })
            .expect("a created sprite is touched");
        Ok((target, touched))
    })
}

/// Creates a folder inside `parent` (an empty path is the project root).
/// Returns the path of the new folder.
pub fn create_folder(
    state: &ProjectState,
    parent: &[String],
    name: &str,
) -> Result<(Vec<String>, ChangedEvent), CoreError> {
    change(state, |session| {
        let touched = session.apply(
            "user",
            vec![Operation::CreateFolder {
                parent: parent.to_vec(),
                name: name.to_string(),
            }],
        )?;
        let path = touched
            .iter()
            .find_map(|t| match t {
                Target::Folder(path) => Some(path.clone()),
                _ => None,
            })
            .expect("a created folder is touched");
        Ok((path, touched))
    })
}

// The first sprite or folder the change touched at its new place.
fn new_sprite_target(touched: &[Target]) -> SpriteTarget {
    touched
        .iter()
        .rev()
        .find_map(|t| match t {
            Target::Sprite(sprite) => Some(SpriteTarget {
                folder: sprite.folder.clone(),
                name: sprite.name.clone(),
            }),
            _ => None,
        })
        .expect("the change touched a sprite")
}

/// Changes the name of a sprite. It keeps its content and its history.
pub fn rename_sprite(
    state: &ProjectState,
    folder: &[String],
    name: &str,
    new_name: &str,
) -> Result<(SpriteTarget, ChangedEvent), CoreError> {
    change(state, |session| {
        let touched = session.apply(
            "user",
            vec![Operation::RenameSprite {
                sprite: SpriteRef {
                    folder: folder.to_vec(),
                    name: name.to_string(),
                },
                name: new_name.to_string(),
            }],
        )?;
        Ok((new_sprite_target(&touched), touched))
    })
}

/// Changes the name of a folder. Returns its new path.
pub fn rename_folder(
    state: &ProjectState,
    path: &[String],
    new_name: &str,
) -> Result<(Vec<String>, ChangedEvent), CoreError> {
    change(state, |session| {
        let touched = session.apply(
            "user",
            vec![Operation::RenameFolder {
                path: path.to_vec(),
                name: new_name.to_string(),
            }],
        )?;
        let new_path = touched
            .iter()
            .rev()
            .find_map(|t| match t {
                Target::Folder(path) => Some(path.clone()),
                _ => None,
            })
            .expect("the change touched a folder");
        Ok((new_path, touched))
    })
}

/// Adds a copy of a sprite in the same folder, named "goblin copy",
/// "goblin copy 2"... Returns the copy.
pub fn duplicate_sprite(
    state: &ProjectState,
    folder: &[String],
    name: &str,
) -> Result<(SpriteTarget, ChangedEvent), CoreError> {
    change(state, |session| {
        let sprite = SpriteRef {
            folder: folder.to_vec(),
            name: name.to_string(),
        };
        let siblings = session
            .project
            .find_folder(folder)
            .ok_or_else(|| DocError::new("not_found", "the folder does not exist"))?;
        let copy = copy_name(name, |candidate| {
            siblings.sprites.iter().any(|s| same_name(&s.name, candidate))
                || siblings.folders.iter().any(|f| same_name(&f.name, candidate))
        });
        let touched = session.apply(
            "user",
            vec![Operation::DuplicateSprite { sprite, name: copy }],
        )?;
        Ok((new_sprite_target(&touched), touched))
    })
}

/// Moves sprites to the trash, all in one step. Returns the revision after
/// the change, so the UI can tell if something else changed since.
pub fn delete_sprites(
    state: &ProjectState,
    sprites: &[SpriteTarget],
) -> Result<(u64, ChangedEvent), CoreError> {
    change(state, |session| {
        let ops = sprites
            .iter()
            .map(|sprite| Operation::DeleteSprite {
                sprite: SpriteRef {
                    folder: sprite.folder.clone(),
                    name: sprite.name.clone(),
                },
            })
            .collect();
        let touched = session.apply("user", ops)?;
        Ok((session.project.revision, touched))
    })
}

/// Moves sprites to another folder (an empty path is the root), all in one
/// step. If a sprite cannot move (its name exists there) none moves.
pub fn move_sprites(
    state: &ProjectState,
    sprites: &[SpriteTarget],
    to: &[String],
) -> Result<((), ChangedEvent), CoreError> {
    change(state, |session| {
        let ops = sprites
            .iter()
            .map(|sprite| Operation::MoveSprite {
                sprite: SpriteRef {
                    folder: sprite.folder.clone(),
                    name: sprite.name.clone(),
                },
                to: to.to_vec(),
                index: None,
            })
            .collect();
        Ok(((), session.apply("user", ops)?))
    })
}

/// Undoes the last change, whoever made it.
pub fn undo(state: &ProjectState) -> Result<((), ChangedEvent), CoreError> {
    change(state, |session| Ok(((), session.undo()?)))
}

pub fn redo(state: &ProjectState) -> Result<((), ChangedEvent), CoreError> {
    change(state, |session| Ok(((), session.redo()?)))
}

#[cfg(test)]
#[path = "browser_tests.rs"]
mod tests;
