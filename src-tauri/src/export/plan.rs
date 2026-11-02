//! What to export: the options, the frames of the scope in a fixed order and
//! the name of every frame.

use super::ExportError;
use crate::document::{Folder, Palette, Project, Sprite, SpriteRef, Tag, frame_duration_ms};

/// Which sprites to export.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Scope {
    Project,
    /// A folder and its sub-folders, by path from the project root.
    Folder(Vec<String>),
    Sprites(Vec<SpriteRef>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    SheetJson,
    PngFiles,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Options {
    /// 1, 2, 4 or 8.
    pub scale: u32,
    /// Pixels between frames, 0 to 16.
    pub spacing: u32,
    /// The pattern of the frame names. `{sprite}`, `{animation}` and `{frame}`.
    pub naming: String,
}

impl Default for Options {
    fn default() -> Self {
        Options { scale: 1, spacing: 0, naming: DEFAULT_NAMING.to_string() }
    }
}

pub const DEFAULT_NAMING: &str = "{sprite}_{animation}_{frame}";

/// A file to write, with its folder-relative name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportFile {
    pub name: String,
    pub bytes: Vec<u8>,
}

/// One frame to export.
pub(super) struct Item<'a> {
    pub sprite: &'a Sprite,
    pub palette: &'a Palette,
    /// From 0.
    pub index: usize,
    pub name: String,
    pub duration_ms: u32,
}

pub(super) fn check_options(options: &Options) -> Result<(), ExportError> {
    let bad = |message: &str| Err(ExportError::new("bad_options", message));
    if ![1, 2, 4, 8].contains(&options.scale) {
        return bad("the scale must be 1, 2, 4 or 8");
    }
    if options.spacing > 16 {
        return bad("the spacing must be from 0 to 16");
    }
    let pattern = &options.naming;
    if pattern.trim().is_empty() || pattern.contains(['/', '\\']) || pattern.contains("..") {
        return bad("the naming pattern must not be empty or have slashes or \"..\"");
    }
    Ok(())
}

/// The sprites of the scope, ordered by name (ignoring case).
pub(super) fn sprites_of<'a>(project: &'a Project, scope: &Scope) -> Vec<&'a Sprite> {
    let mut found: Vec<&Sprite> = match scope {
        Scope::Project => collect(&project.root),
        Scope::Folder(path) => project.find_folder(path).map(collect).unwrap_or_default(),
        Scope::Sprites(refs) => refs.iter().filter_map(|r| project.find_sprite(r)).collect(),
    };
    found.sort_by_key(|s| (s.name.to_lowercase(), s.name.clone()));
    found.dedup_by(|a, b| std::ptr::eq(*a, *b));
    found
}

fn collect(folder: &Folder) -> Vec<&Sprite> {
    let mut all: Vec<&Sprite> = folder.sprites.iter().collect();
    for sub in &folder.folders {
        all.extend(collect(sub));
    }
    all
}

/// The frames of the scope: sprites by name, frames by number.
pub(super) fn items<'a>(
    project: &'a Project,
    scope: &Scope,
    options: &Options,
) -> Result<Vec<Item<'a>>, ExportError> {
    check_options(options)?;
    let mut items = Vec::new();
    for sprite in sprites_of(project, scope) {
        let palette = project.palette(&sprite.palette).ok_or_else(|| {
            ExportError::new(
                "missing_palette",
                format!("the palette {:?} of sprite {:?} does not exist", sprite.palette, sprite.name),
            )
        })?;
        for index in 0..sprite.frame_count() {
            let tag = sprite.tags.iter().find(|t| in_tag(t, index));
            let duration_ms = match tag {
                Some(t) => frame_duration_ms(sprite.frame_durations[index], t.fps),
                None => sprite.frame_durations[index],
            };
            let name = frame_name(&options.naming, sprite, tag, index);
            items.push(Item { sprite, palette, index, name, duration_ms });
        }
    }
    make_unique(&mut items);
    Ok(items)
}

/// Fills the pattern. A frame in an animation is numbered inside it (the
/// first frame of the animation is 1); a frame that is in none has no
/// animation name and its number in the sprite.
fn frame_name(pattern: &str, sprite: &Sprite, tag: Option<&Tag>, index: usize) -> String {
    let (animation, number) = match tag {
        Some(t) => (t.name.as_str(), index + 2 - t.from as usize),
        None => ("", index + 1),
    };
    let mut name = pattern
        .replace("{sprite}", &sprite.name)
        .replace("{animation}", animation)
        .replace("{frame}", &number.to_string());
    // An empty animation name leaves a double separator: `goblin__1`.
    while name.contains("__") {
        name = name.replace("__", "_");
    }
    name.trim_matches('_').to_string()
}

/// Two sprites in different folders can share a name. Later duplicates get
/// `_2`, `_3`... Names compare ignoring case, as some systems do.
fn make_unique(items: &mut [Item]) {
    let mut seen = std::collections::HashSet::new();
    for item in items.iter_mut() {
        let base = item.name.clone();
        let mut n = 2;
        while !seen.insert(item.name.to_lowercase()) {
            item.name = format!("{base}_{n}");
            n += 1;
        }
    }
}

fn in_tag(tag: &Tag, index: usize) -> bool {
    (tag.from as usize..=tag.to as usize).contains(&(index + 1))
}
