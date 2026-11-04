//! Animations made in one step. Each function looks at a project and returns
//! the operations of ONE user or tool action; the caller applies them as one
//! history entry. The editor commands and the MCP tools both use them, so
//! they follow the same rules and give the same error codes.

use super::{DEFAULT_FRAME_MS, DocError, Operation, Project, SpriteRef, Tag, TagMode, same_name};

/// The most frames one call may add (a 512x512 frame is 256 KB).
pub const MAX_NEW_FRAMES: usize = 128;

fn name_exists(name: &str) -> DocError {
    DocError::new("name_exists", format!("an animation called {name:?} already exists (names ignore case)"))
}

fn animation_not_found(name: &str) -> DocError {
    DocError::new("not_found", format!("animation {name:?} does not exist"))
}

fn sprite_not_found(sprite: &SpriteRef) -> DocError {
    DocError::new("not_found", format!("sprite {:?} does not exist", sprite.name))
}

/// Copies the frames in `sources` (frame numbers, in order) to place `at`
/// (default: the end) and tags the copies. Inserting moves the later frames,
/// so each source is followed while the copies go in.
fn copy_frames(
    project: &Project,
    sprite: &SpriteRef,
    sources: Vec<u32>,
    at: Option<u32>,
    tag: Tag,
) -> Result<Vec<Operation>, DocError> {
    let target = project.find_sprite(sprite).ok_or_else(|| sprite_not_found(sprite))?;
    if target.tags.iter().any(|t| same_name(&t.name, &tag.name)) {
        return Err(name_exists(&tag.name));
    }
    let count = target.frame_count() as u32;
    if sources.is_empty() || sources.len() > MAX_NEW_FRAMES {
        return Err(DocError::new("bad_argument", format!("`frames` must be 1 to {MAX_NEW_FRAMES}")));
    }
    for &source in &sources {
        if source < 1 || source > count {
            return Err(DocError::new(
                "bad_frame",
                format!("frame {source} does not exist, the sprite has {count} frames"),
            ));
        }
    }
    let at = at.unwrap_or(count + 1);
    if at < 1 || at > count + 1 {
        return Err(DocError::new("bad_frame", format!("`at` is {at}; use 1 to {}", count + 1)));
    }
    // New frames inside an animation would stretch it.
    if let Some(inside) = target.tags.iter().find(|t| t.from < at && at <= t.to) {
        return Err(DocError::new(
            "animation_overlap",
            format!(
                "place {at} is inside the animation {:?} (frames {} to {}); use another `at`",
                inside.name, inside.from, inside.to
            ),
        ));
    }
    let n = sources.len() as u32;
    let tag = Tag { from: at, to: at + n - 1, ..tag };
    tag.validate((count + n) as usize)?;
    let mut sources = sources;
    let mut ops = Vec::new();
    for i in 0..sources.len() {
        let place = at + i as u32;
        let source = sources[i];
        ops.push(Operation::InsertFrame {
            sprite: sprite.clone(),
            at: place,
            copy_of: Some(source),
            duration_ms: DEFAULT_FRAME_MS,
        });
        for later in &mut sources[i + 1..] {
            if place <= *later {
                *later += 1;
            }
        }
    }
    ops.push(Operation::AddTag { sprite: sprite.clone(), tag, index: None });
    Ok(ops)
}

/// A new animation of `frames` copies of the frame `base`.
pub fn create_animation(
    project: &Project,
    sprite: &SpriteRef,
    name: &str,
    frames: usize,
    fps: u32,
    mode: TagMode,
    base: u32,
    at: Option<u32>,
) -> Result<Vec<Operation>, DocError> {
    if frames == 0 || frames > MAX_NEW_FRAMES {
        return Err(DocError::new("bad_argument", format!("`frames` must be 1 to {MAX_NEW_FRAMES}")));
    }
    let tag = Tag { name: name.to_string(), from: 1, to: 1, fps, mode };
    copy_frames(project, sprite, vec![base; frames], at, tag)
}

/// A copy of the animation `name` (its frames and speed) called `new_name`.
pub fn duplicate_animation(
    project: &Project,
    sprite: &SpriteRef,
    name: &str,
    new_name: &str,
    at: Option<u32>,
) -> Result<Vec<Operation>, DocError> {
    let target = project.find_sprite(sprite).ok_or_else(|| sprite_not_found(sprite))?;
    let source = target
        .tags
        .iter()
        .find(|t| same_name(&t.name, name))
        .ok_or_else(|| animation_not_found(name))?;
    let tag = Tag { name: new_name.to_string(), ..source.clone() };
    copy_frames(project, sprite, (source.from..=source.to).collect(), at, tag)
}

/// The animation `name` gets the name `new_name`; its frames do not change.
pub fn rename_animation(
    project: &Project,
    sprite: &SpriteRef,
    name: &str,
    new_name: &str,
) -> Result<Vec<Operation>, DocError> {
    let target = project.find_sprite(sprite).ok_or_else(|| sprite_not_found(sprite))?;
    let old = target
        .tags
        .iter()
        .find(|t| same_name(&t.name, name))
        .ok_or_else(|| animation_not_found(name))?;
    if target.tags.iter().any(|t| !std::ptr::eq(t, old) && same_name(&t.name, new_name)) {
        return Err(name_exists(new_name));
    }
    let tag = Tag { name: new_name.to_string(), ..old.clone() };
    tag.validate(target.frame_count())?;
    Ok(vec![Operation::SetTag { sprite: sprite.clone(), name: old.name.clone(), tag }])
}

/// Removes the animation. With `with_frames` its frames go too (the last
/// frame first, so the numbers stay right); a sprite keeps at least one frame.
pub fn delete_animation(
    project: &Project,
    sprite: &SpriteRef,
    name: &str,
    with_frames: bool,
) -> Result<Vec<Operation>, DocError> {
    let target = project.find_sprite(sprite).ok_or_else(|| sprite_not_found(sprite))?;
    let tag = target
        .tags
        .iter()
        .find(|t| same_name(&t.name, name))
        .ok_or_else(|| animation_not_found(name))?;
    let mut ops = vec![Operation::RemoveTag { sprite: sprite.clone(), name: tag.name.clone() }];
    if with_frames {
        ops.extend((tag.from..=tag.to).rev().map(|frame| Operation::DeleteFrame { sprite: sprite.clone(), frame }));
    }
    Ok(ops)
}
