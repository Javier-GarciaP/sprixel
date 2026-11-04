//! What the frames strip asks to do with whole animations: create one from the
//! base frame, duplicate, rename and delete (with or without its frames). Each
//! call is one user action, so one Undo step. The rules are in
//! `document::animation`; the MCP tools use the same ones.

use super::browser::{self, ChangedEvent, CoreError};
use super::frames::{SpriteArg, announce};
use super::project::ProjectState;
use crate::document::{self, DocError, Operation, TagMode};

const USER: &str = crate::history::USER;

/// Looks at the project and applies the operations that `plan` returns.
fn run(
    state: &ProjectState,
    sprite: &SpriteArg,
    plan: impl FnOnce(&document::Project, &document::SpriteRef) -> Result<Vec<Operation>, DocError>,
) -> Result<ChangedEvent, CoreError> {
    browser::change(state, |session| {
        let target = sprite.to_ref();
        let ops = plan(&session.project, &target)?;
        let touched = session.apply(USER, ops)?;
        Ok(((), touched))
    })
    .map(|(_, event)| event)
}

#[allow(clippy::too_many_arguments)]
pub fn create(
    state: &ProjectState,
    sprite: &SpriteArg,
    name: &str,
    frames: usize,
    fps: u32,
    mode: &str,
    base: Option<u32>,
    at: Option<u32>,
) -> Result<ChangedEvent, CoreError> {
    let mode = TagMode::parse(mode)
        .ok_or_else(|| DocError::new("bad_mode", format!("{mode:?} is not a repeat mode")))?;
    run(state, sprite, |project, target| {
        document::create_animation(project, target, name, frames, fps, mode, base.unwrap_or(1), at)
    })
}

pub fn duplicate(state: &ProjectState, sprite: &SpriteArg, name: &str, new_name: &str) -> Result<ChangedEvent, CoreError> {
    run(state, sprite, |project, target| document::duplicate_animation(project, target, name, new_name, None))
}

pub fn rename(state: &ProjectState, sprite: &SpriteArg, name: &str, new_name: &str) -> Result<ChangedEvent, CoreError> {
    run(state, sprite, |project, target| document::rename_animation(project, target, name, new_name))
}

pub fn delete(state: &ProjectState, sprite: &SpriteArg, name: &str, with_frames: bool) -> Result<ChangedEvent, CoreError> {
    run(state, sprite, |project, target| document::delete_animation(project, target, name, with_frames))
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub fn animation_create(
    app: tauri::AppHandle,
    state: tauri::State<ProjectState>,
    sprite: SpriteArg,
    name: String,
    frames: usize,
    fps: u32,
    mode: String,
    base: Option<u32>,
    at: Option<u32>,
) -> Result<(), CoreError> {
    announce(&app, create(&state, &sprite, &name, frames, fps, &mode, base, at)?);
    Ok(())
}

#[tauri::command]
pub fn animation_duplicate(
    app: tauri::AppHandle,
    state: tauri::State<ProjectState>,
    sprite: SpriteArg,
    name: String,
    new_name: String,
) -> Result<(), CoreError> {
    announce(&app, duplicate(&state, &sprite, &name, &new_name)?);
    Ok(())
}

#[tauri::command]
pub fn animation_rename(
    app: tauri::AppHandle,
    state: tauri::State<ProjectState>,
    sprite: SpriteArg,
    name: String,
    new_name: String,
) -> Result<(), CoreError> {
    announce(&app, rename(&state, &sprite, &name, &new_name)?);
    Ok(())
}

#[tauri::command]
pub fn animation_delete(
    app: tauri::AppHandle,
    state: tauri::State<ProjectState>,
    sprite: SpriteArg,
    name: String,
    with_frames: bool,
) -> Result<(), CoreError> {
    announce(&app, delete(&state, &sprite, &name, with_frames)?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ipc::frames::tests::{arg, open};
    use crate::ipc::frames::{frames_info, TagArg};
    use crate::ipc::project;

    fn tags(state: &ProjectState) -> Vec<TagArg> {
        frames_info(state, &arg()).unwrap().tags
    }

    fn undo(state: &ProjectState) {
        project::edit(state, |session| session.undo().unwrap()).unwrap();
    }

    // Checks the four editor actions on the 4-frame sprite of the helper: each
    // one is ONE undo step, and the events say that the sprite changed.
    #[test]
    fn each_action_is_one_undo_step() {
        let (_dir, state) = open();
        let before = frames_info(&state, &arg()).unwrap();

        create(&state, &arg(), "walk", 3, 8, "loop", None, None).unwrap();
        assert_eq!(frames_info(&state, &arg()).unwrap().frame_count, 7);
        assert_eq!(tags(&state)[0], TagArg { name: "walk".into(), from: 5, to: 7, fps: 8, mode: "loop".into() });
        undo(&state);
        assert_eq!(frames_info(&state, &arg()).unwrap(), before);

        create(&state, &arg(), "walk", 2, 8, "once", Some(2), None).unwrap();
        let event = duplicate(&state, &arg(), "walk", "walk2").unwrap();
        assert!(!event.touched.is_empty());
        assert_eq!(tags(&state).len(), 2);
        undo(&state);
        assert_eq!(tags(&state).len(), 1);

        rename(&state, &arg(), "walk", "run").unwrap();
        assert_eq!(tags(&state)[0].name, "run");
        undo(&state);
        assert_eq!(tags(&state)[0].name, "walk");

        delete(&state, &arg(), "walk", true).unwrap();
        assert_eq!(frames_info(&state, &arg()).unwrap(), before);
        undo(&state);
        assert_eq!(frames_info(&state, &arg()).unwrap().frame_count, 6);
        assert_eq!(tags(&state).len(), 1);
    }

    // Checks the errors the UI gets, and that nothing changes after one.
    #[test]
    fn errors_have_the_codes_of_the_tools() {
        let (_dir, state) = open();
        create(&state, &arg(), "walk", 1, 8, "loop", None, None).unwrap();
        let before = frames_info(&state, &arg()).unwrap();
        assert_eq!(create(&state, &arg(), "WALK", 1, 8, "loop", None, None).unwrap_err().code, "name_exists");
        assert_eq!(create(&state, &arg(), "x", 1, 8, "spin", None, None).unwrap_err().code, "bad_mode");
        assert_eq!(rename(&state, &arg(), "nope", "x").unwrap_err().code, "not_found");
        assert_eq!(delete(&state, &arg(), "nope", false).unwrap_err().code, "not_found");
        assert_eq!(frames_info(&state, &arg()).unwrap(), before);
    }
}
