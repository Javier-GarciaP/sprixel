//! What the frames column, the animations and the preview ask of the core:
//! the frames and tags of a sprite, and the changes a person makes to them.
//! Every change is an operation by the user, so undo is the one of the
//! project, and every change gives the `project://changed` event. The
//! functions do not need a window; the commands at the end add the event.

use super::browser::{self, CHANGED_EVENT, ChangedEvent, CoreError};
use super::project::{self, ProjectState};
use crate::document::{DocError, Operation, SpriteRef, Tag, TagMode, DEFAULT_FRAME_MS};
use serde::{Deserialize, Serialize};

const USER: &str = crate::history::USER;

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

/// An animation as the UI sends and reads it. `mode` is `loop`, `once` or
/// `pingpong`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TagArg {
    pub name: String,
    pub from: u32,
    pub to: u32,
    pub fps: u32,
    pub mode: String,
}

impl TagArg {
    fn to_tag(&self) -> Result<Tag, DocError> {
        let mode = TagMode::parse(&self.mode)
            .ok_or_else(|| DocError::new("bad_mode", format!("{:?} is not a repeat mode", self.mode)))?;
        Ok(Tag {
            name: self.name.clone(),
            from: self.from,
            to: self.to,
            fps: self.fps,
            mode,
        })
    }
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FramesInfo {
    pub frame_count: usize,
    /// One duration in milliseconds per frame. `DEFAULT_FRAME_MS` means that
    /// the frame has none of its own.
    pub durations: Vec<u32>,
    pub default_ms: u32,
    pub tags: Vec<TagArg>,
}

fn not_found(sprite: &SpriteArg) -> DocError {
    DocError::new("not_found", format!("sprite {:?} does not exist", sprite.name))
}

pub fn frames_info(state: &ProjectState, sprite: &SpriteArg) -> Result<FramesInfo, CoreError> {
    project::read(state, |session| {
        let found = session
            .project
            .find_sprite(&sprite.to_ref())
            .ok_or_else(|| CoreError::from(not_found(sprite)))?;
        Ok(FramesInfo {
            frame_count: found.frame_count(),
            durations: found.frame_durations.clone(),
            default_ms: DEFAULT_FRAME_MS,
            tags: found
                .tags
                .iter()
                .map(|t| TagArg {
                    name: t.name.clone(),
                    from: t.from,
                    to: t.to,
                    fps: t.fps,
                    mode: t.mode.as_str().to_string(),
                })
                .collect(),
        })
    })
    .ok_or_else(CoreError::no_project)?
}

/// Puts a frame at `at` (blank, or a copy of `copy_of`). Returns its number.
pub fn insert_frame(
    state: &ProjectState,
    sprite: &SpriteArg,
    at: u32,
    copy_of: Option<u32>,
) -> Result<(u32, ChangedEvent), CoreError> {
    run(state, sprite, |target| {
        vec![Operation::InsertFrame {
            sprite: target,
            at,
            copy_of,
            duration_ms: DEFAULT_FRAME_MS,
        }]
    })
    .map(|event| (at, event))
}

pub fn delete_frame(state: &ProjectState, sprite: &SpriteArg, frame: u32) -> Result<ChangedEvent, CoreError> {
    run(state, sprite, |target| vec![Operation::DeleteFrame { sprite: target, frame }])
}

pub fn move_frame(
    state: &ProjectState,
    sprite: &SpriteArg,
    from: u32,
    to: u32,
) -> Result<ChangedEvent, CoreError> {
    run(state, sprite, |target| {
        vec![Operation::MoveFrame { sprite: target, from, to, tags: None }]
    })
}

pub fn add_tag(state: &ProjectState, sprite: &SpriteArg, tag: &TagArg) -> Result<ChangedEvent, CoreError> {
    let tag = tag.to_tag()?;
    run(state, sprite, |target| {
        vec![Operation::AddTag { sprite: target, tag, index: None }]
    })
}

/// Replaces the animation called `name` (it can be renamed).
pub fn set_tag(
    state: &ProjectState,
    sprite: &SpriteArg,
    name: &str,
    tag: &TagArg,
) -> Result<ChangedEvent, CoreError> {
    let tag = tag.to_tag()?;
    run(state, sprite, |target| {
        vec![Operation::SetTag { sprite: target, name: name.to_string(), tag }]
    })
}

pub fn remove_tag(state: &ProjectState, sprite: &SpriteArg, name: &str) -> Result<ChangedEvent, CoreError> {
    run(state, sprite, |target| {
        vec![Operation::RemoveTag { sprite: target, name: name.to_string() }]
    })
}

// One user action on a sprite: the operations are one history entry.
fn run(
    state: &ProjectState,
    sprite: &SpriteArg,
    ops: impl FnOnce(SpriteRef) -> Vec<Operation>,
) -> Result<ChangedEvent, CoreError> {
    browser::change(state, |session| {
        let touched = session.apply(USER, ops(sprite.to_ref()))?;
        Ok(((), touched))
    })
    .map(|(_, event)| event)
}

fn announce(app: &tauri::AppHandle, event: ChangedEvent) {
    let _ = tauri::Emitter::emit(app, CHANGED_EVENT, event);
}

#[tauri::command]
pub fn frames_info_of(
    state: tauri::State<ProjectState>,
    sprite: SpriteArg,
) -> Result<FramesInfo, CoreError> {
    frames_info(&state, &sprite)
}

#[tauri::command]
pub fn frames_insert(
    app: tauri::AppHandle,
    state: tauri::State<ProjectState>,
    sprite: SpriteArg,
    at: u32,
    copy_of: Option<u32>,
) -> Result<u32, CoreError> {
    let (frame, event) = insert_frame(&state, &sprite, at, copy_of)?;
    announce(&app, event);
    Ok(frame)
}

#[tauri::command]
pub fn frames_delete(
    app: tauri::AppHandle,
    state: tauri::State<ProjectState>,
    sprite: SpriteArg,
    frame: u32,
) -> Result<(), CoreError> {
    announce(&app, delete_frame(&state, &sprite, frame)?);
    Ok(())
}

#[tauri::command]
pub fn frames_move(
    app: tauri::AppHandle,
    state: tauri::State<ProjectState>,
    sprite: SpriteArg,
    from: u32,
    to: u32,
) -> Result<(), CoreError> {
    announce(&app, move_frame(&state, &sprite, from, to)?);
    Ok(())
}

#[tauri::command]
pub fn tag_add(
    app: tauri::AppHandle,
    state: tauri::State<ProjectState>,
    sprite: SpriteArg,
    tag: TagArg,
) -> Result<(), CoreError> {
    announce(&app, add_tag(&state, &sprite, &tag)?);
    Ok(())
}

#[tauri::command]
pub fn tag_set(
    app: tauri::AppHandle,
    state: tauri::State<ProjectState>,
    sprite: SpriteArg,
    name: String,
    tag: TagArg,
) -> Result<(), CoreError> {
    announce(&app, set_tag(&state, &sprite, &name, &tag)?);
    Ok(())
}

#[tauri::command]
pub fn tag_remove(
    app: tauri::AppHandle,
    state: tauri::State<ProjectState>,
    sprite: SpriteArg,
    name: String,
) -> Result<(), CoreError> {
    announce(&app, remove_tag(&state, &sprite, &name)?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::{Color, Palette};
    use crate::ipc::browser::TargetInfo;

    fn arg() -> SpriteArg {
        SpriteArg { folder: vec![], name: "goblin".to_string() }
    }

    fn walk(from: u32, to: u32) -> TagArg {
        TagArg { name: "walk".to_string(), from, to, fps: 12, mode: "loop".to_string() }
    }

    /// An open project with a 2x1 sprite `goblin` of 4 frames.
    fn open() -> (tempfile::TempDir, ProjectState) {
        let parent = tempfile::tempdir().unwrap();
        let state = ProjectState::default();
        project::create_project(&state, parent.path(), "Kit").unwrap();
        project::edit(&state, |session| {
            let mut ops = vec![
                Operation::CreatePalette {
                    palette: Palette {
                        name: "main".to_string(),
                        colors: vec![Color { key: 'A', name: "a".into(), hex: "#112233".into() }],
                    },
                    index: None,
                },
                Operation::CreateSprite {
                    folder: vec![],
                    name: "goblin".to_string(),
                    width: 2,
                    height: 1,
                    palette: "main".to_string(),
                },
            ];
            for _ in 0..3 {
                ops.push(Operation::AddFrame {
                    sprite: SpriteRef::new(&[], "goblin"),
                    duration_ms: 100,
                });
            }
            session.apply(USER, ops).unwrap();
            session.history.clear();
        })
        .unwrap();
        (parent, state)
    }

    fn sprite_event(event: &ChangedEvent) -> bool {
        event.touched.contains(&TargetInfo { kind: "sprite", path: vec!["goblin".to_string()] })
    }

    // Checks that the info has the number of frames, their durations and the
    // animations, and fails clearly without a project or a sprite.
    #[test]
    fn reads_the_frames_and_the_animations() {
        let (_dir, state) = open();
        add_tag(&state, &arg(), &walk(1, 3)).unwrap();
        let info = frames_info(&state, &arg()).unwrap();
        assert_eq!(info.frame_count, 4);
        assert_eq!(info.durations, vec![100; 4]);
        assert_eq!(info.default_ms, 100);
        assert_eq!(info.tags, vec![walk(1, 3)]);
        let ghost = SpriteArg { folder: vec![], name: "ghost".to_string() };
        assert_eq!(frames_info(&state, &ghost).unwrap_err().code, "not_found");
        assert_eq!(frames_info(&ProjectState::default(), &arg()).unwrap_err().code, "no_project");
    }

    // Checks that an insert gives the event with the sprite, the new frame
    // number, and that the project undo (the one of every tab) takes it back.
    #[test]
    fn an_insert_gives_the_event_and_the_project_undo_reverts_it() {
        let (_dir, state) = open();
        let (frame, event) = insert_frame(&state, &arg(), 3, Some(2)).unwrap();
        assert_eq!(frame, 3);
        assert!(sprite_event(&event));
        assert_eq!(frames_info(&state, &arg()).unwrap().frame_count, 5);

        let (_, undone) = browser::undo(&state).unwrap();
        assert!(sprite_event(&undone));
        assert_eq!(frames_info(&state, &arg()).unwrap().frame_count, 4);
    }

    // Checks delete, with the last-frame rule as an error code, and move.
    #[test]
    fn deletes_and_moves_frames() {
        let (_dir, state) = open();
        move_frame(&state, &arg(), 4, 2).unwrap();
        delete_frame(&state, &arg(), 1).unwrap();
        assert_eq!(frames_info(&state, &arg()).unwrap().frame_count, 3);
        delete_frame(&state, &arg(), 1).unwrap();
        delete_frame(&state, &arg(), 1).unwrap();
        assert_eq!(delete_frame(&state, &arg(), 1).unwrap_err().code, "last_frame");
        assert_eq!(move_frame(&state, &arg(), 1, 5).unwrap_err().code, "bad_frame");
    }

    // Checks the animations: create, edit (rename and new range), delete, and
    // the errors of a bad range, a repeated name and a bad mode.
    #[test]
    fn creates_edits_and_deletes_animations() {
        let (_dir, state) = open();
        assert!(sprite_event(&add_tag(&state, &arg(), &walk(1, 4)).unwrap()));
        assert_eq!(add_tag(&state, &arg(), &walk(1, 2)).unwrap_err().code, "tag_exists");
        assert_eq!(add_tag(&state, &arg(), &TagArg { name: "x".into(), ..walk(3, 2) }).unwrap_err().code, "bad_range");
        let bad = TagArg { mode: "spin".to_string(), ..walk(1, 2) };
        assert_eq!(add_tag(&state, &arg(), &bad).unwrap_err().code, "bad_mode");

        let run = TagArg { name: "run".to_string(), from: 2, to: 3, fps: 24, mode: "pingpong".to_string() };
        set_tag(&state, &arg(), "walk", &run).unwrap();
        assert_eq!(frames_info(&state, &arg()).unwrap().tags, vec![run]);
        remove_tag(&state, &arg(), "run").unwrap();
        assert!(frames_info(&state, &arg()).unwrap().tags.is_empty());
        assert_eq!(remove_tag(&state, &arg(), "run").unwrap_err().code, "not_found");
    }

    // Checks the wire format the UI reads.
    #[test]
    fn the_info_serializes_for_the_ui() {
        let (_dir, state) = open();
        add_tag(&state, &arg(), &walk(1, 2)).unwrap();
        let json = serde_json::to_value(frames_info(&state, &arg()).unwrap()).unwrap();
        assert_eq!(json["frameCount"], 4);
        assert_eq!(json["defaultMs"], 100);
        assert_eq!(json["tags"][0]["mode"], "loop");
    }
}
