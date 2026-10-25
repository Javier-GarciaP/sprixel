use super::error::DocError;
use super::model::{Color, Folder, Frame, MAX_COLORS, Palette, Project, Sprite, Tag};
use super::names::{same_name, validate_name};

/// Where a sprite is: the folders from the project root, then its name.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SpriteRef {
    pub folder: Vec<String>,
    pub name: String,
}

impl SpriteRef {
    pub fn new(folder: &[&str], name: &str) -> SpriteRef {
        SpriteRef {
            folder: folder.iter().map(|f| f.to_string()).collect(),
            name: name.to_string(),
        }
    }
}

/// A part of the project that an operation changed. Storage uses it to know
/// which files to write, move to the trash or restore. The order puts
/// folders before palettes and sprites, and parents before children.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Target {
    Folder(Vec<String>),
    Palette(String),
    Sprite(SpriteRef),
}

#[derive(Debug, Clone, PartialEq)]
pub struct PixelChange {
    pub x: u32,
    pub y: u32,
    pub key: char,
}

/// One change to the project. Frames are numbered from 1, layers from 0.
///
/// The `Restore*` operations and the `index` fields exist so that the
/// inverse of a delete can put things back in the same place.
#[derive(Debug, Clone, PartialEq)]
pub enum Operation {
    CreatePalette {
        palette: Palette,
        /// Where to insert it; `None` adds it at the end.
        index: Option<usize>,
    },
    DeletePalette {
        name: String,
    },
    /// Adds a color at the end of a palette (or at `index`). It fails with
    /// `palette_full` when the palette has 93 colors.
    AddColor {
        palette: String,
        color: Color,
        index: Option<usize>,
    },
    /// Removes a color that no sprite uses.
    RemoveColor {
        palette: String,
        key: char,
    },
    CreateFolder {
        parent: Vec<String>,
        name: String,
    },
    DeleteFolder {
        path: Vec<String>,
    },
    RestoreFolder {
        parent: Vec<String>,
        index: usize,
        folder: Folder,
    },
    CreateSprite {
        folder: Vec<String>,
        name: String,
        width: u32,
        height: u32,
        palette: String,
    },
    DeleteSprite {
        sprite: SpriteRef,
    },
    RestoreSprite {
        folder: Vec<String>,
        index: usize,
        sprite: Sprite,
    },
    /// Changes the name of a sprite. Its content and position stay.
    RenameSprite {
        sprite: SpriteRef,
        name: String,
    },
    /// Changes the name of a folder. The project root cannot be renamed.
    RenameFolder {
        path: Vec<String>,
        name: String,
    },
    /// Moves a sprite to another folder (an empty path is the project root).
    /// `index` is the place in that folder; `None` puts it at the end.
    MoveSprite {
        sprite: SpriteRef,
        to: Vec<String>,
        index: Option<usize>,
    },
    /// Adds a copy of a sprite with a new name right after the original.
    DuplicateSprite {
        sprite: SpriteRef,
        name: String,
    },
    SetPixels {
        sprite: SpriteRef,
        layer: usize,
        frame: u32,
        pixels: Vec<PixelChange>,
    },
    AddFrame {
        sprite: SpriteRef,
        duration_ms: u32,
    },
    DeleteFrame {
        sprite: SpriteRef,
        frame: u32,
    },
    RestoreFrame {
        sprite: SpriteRef,
        frame: u32,
        /// The pixels of the frame, one buffer per layer.
        layers: Vec<Vec<u8>>,
        duration_ms: u32,
        /// All the tags the sprite had before the frame was deleted.
        tags: Vec<Tag>,
    },
    AddTag {
        sprite: SpriteRef,
        tag: Tag,
        /// Where to insert it; `None` adds it at the end.
        index: Option<usize>,
    },
    RemoveTag {
        sprite: SpriteRef,
        name: String,
    },
}

impl Operation {
    /// True for the operations that change the name or the place of a sprite
    /// or a folder (see `AppliedGroup::moved`).
    fn changes_place(&self) -> bool {
        matches!(
            self,
            Operation::RenameSprite { .. }
                | Operation::RenameFolder { .. }
                | Operation::MoveSprite { .. }
        )
    }
}

/// The result of one applied operation.
#[derive(Debug, Clone, PartialEq)]
pub struct Applied {
    /// Applying it undoes the operation.
    pub inverse: Operation,
    pub touched: Vec<Target>,
}

/// The result of an applied group of operations.
#[derive(Debug, Clone, PartialEq)]
pub struct AppliedGroup {
    /// Applying them in this order undoes the whole group.
    pub inverses: Vec<Operation>,
    pub touched: Vec<Target>,
    /// The sprites and folders that changed place or name, from the old
    /// place to the new one, in the order they happened. Storage moves their
    /// files instead of writing a copy and trashing the original.
    pub moved: Vec<(Target, Target)>,
}

impl Project {
    /// Applies several operations as one step: all of them or none. The
    /// revision grows by one. If an operation is rejected, the ones before it
    /// are undone with their inverses and the error is returned.
    pub fn apply_group(&mut self, ops: &[Operation]) -> Result<AppliedGroup, DocError> {
        let mut inverses = Vec::with_capacity(ops.len());
        let mut touched = Vec::new();
        let mut moved = Vec::new();
        for op in ops {
            match self.apply_op(op) {
                Ok(applied) => {
                    if op.changes_place() {
                        if let [from, to] = applied.touched.as_slice() {
                            moved.push((from.clone(), to.clone()));
                        }
                    }
                    inverses.push(applied.inverse);
                    touched.extend(applied.touched);
                }
                Err(error) => {
                    for inverse in inverses.iter().rev() {
                        self.apply_op(inverse).map_err(|e| {
                            DocError::new("internal", format!("could not roll back: {e}"))
                        })?;
                    }
                    return Err(error);
                }
            }
        }
        inverses.reverse();
        self.revision += 1;
        Ok(AppliedGroup {
            inverses,
            touched,
            moved,
        })
    }

    /// Applies one operation. On success the revision grows by one. On error
    /// nothing changes, including the revision.
    pub fn apply(&mut self, op: &Operation) -> Result<Applied, DocError> {
        let applied = self.apply_op(op)?;
        self.revision += 1;
        Ok(applied)
    }

    /// Applies one operation without touching the revision. Every operation
    /// checks everything first and changes the project only at the end.
    fn apply_op(&mut self, op: &Operation) -> Result<Applied, DocError> {
        match op {
            Operation::CreatePalette { palette, index } => {
                palette.validate()?;
                validate_name(&palette.name)?;
                if self.palette(&palette.name).is_some() {
                    return Err(name_exists(&palette.name));
                }
                let at = index.unwrap_or(self.palettes.len()).min(self.palettes.len());
                self.palettes.insert(at, palette.clone());
                Ok(Applied {
                    inverse: Operation::DeletePalette {
                        name: palette.name.clone(),
                    },
                    touched: vec![Target::Palette(palette.name.clone())],
                })
            }
            Operation::DeletePalette { name } => {
                let index = self
                    .palettes
                    .iter()
                    .position(|p| same_name(&p.name, name))
                    .ok_or_else(|| not_found("palette", name))?;
                let palette_name = self.palettes[index].name.clone();
                if all_sprites(&self.root).any(|s| same_name(&s.palette, &palette_name)) {
                    return Err(DocError::new(
                        "palette_in_use",
                        format!("palette {palette_name:?} is used by a sprite"),
                    ));
                }
                let palette = self.palettes.remove(index);
                Ok(Applied {
                    inverse: Operation::CreatePalette {
                        palette,
                        index: Some(index),
                    },
                    touched: vec![Target::Palette(palette_name)],
                })
            }
            Operation::AddColor {
                palette,
                color,
                index,
            } => {
                let at_palette = self
                    .palettes
                    .iter()
                    .position(|p| same_name(&p.name, palette))
                    .ok_or_else(|| not_found("palette", palette))?;
                let target = &self.palettes[at_palette];
                if target.colors.len() >= MAX_COLORS {
                    return Err(DocError::new(
                        "palette_full",
                        format!("palette {:?} already has {MAX_COLORS} colors", target.name),
                    ));
                }
                // Check the new color together with the others before changing.
                let mut colors = target.colors.clone();
                let at = index.unwrap_or(colors.len()).min(colors.len());
                colors.insert(at, color.clone());
                Palette {
                    name: target.name.clone(),
                    colors,
                }
                .validate()?;
                let name = target.name.clone();
                self.palettes[at_palette].colors.insert(at, color.clone());
                Ok(Applied {
                    inverse: Operation::RemoveColor {
                        palette: name.clone(),
                        key: color.key,
                    },
                    touched: vec![Target::Palette(name)],
                })
            }
            Operation::RemoveColor { palette, key } => {
                let at_palette = self
                    .palettes
                    .iter()
                    .position(|p| same_name(&p.name, palette))
                    .ok_or_else(|| not_found("palette", palette))?;
                let name = self.palettes[at_palette].name.clone();
                let at = self.palettes[at_palette]
                    .colors
                    .iter()
                    .position(|c| c.key == *key)
                    .ok_or_else(|| not_found("color", &key.to_string()))?;
                let in_use = all_sprites(&self.root)
                    .filter(|s| same_name(&s.palette, &name))
                    .any(|s| {
                        s.layers
                            .iter()
                            .flat_map(|l| l.frames.iter())
                            .any(|f| f.pixels.contains(&(*key as u8)))
                    });
                if in_use {
                    return Err(DocError::new(
                        "color_in_use",
                        format!("color {key:?} is used by a sprite"),
                    ));
                }
                let color = self.palettes[at_palette].colors.remove(at);
                Ok(Applied {
                    inverse: Operation::AddColor {
                        palette: name.clone(),
                        color,
                        index: Some(at),
                    },
                    touched: vec![Target::Palette(name)],
                })
            }
            Operation::CreateFolder { parent, name } => {
                validate_name(name)?;
                let parent = self.canonical_folder(parent)?;
                let folder = self.folder_mut(&parent);
                if name_taken(folder, name) {
                    return Err(name_exists(name));
                }
                folder.folders.push(Folder::new(name));
                let mut path = parent;
                path.push(name.clone());
                Ok(Applied {
                    inverse: Operation::DeleteFolder { path: path.clone() },
                    touched: vec![Target::Folder(path)],
                })
            }
            Operation::DeleteFolder { path } => {
                let mut path = self.canonical_folder(path)?;
                let Some(name) = path.pop() else {
                    return Err(DocError::new("bad_path", "the project root cannot be deleted"));
                };
                let parent = self.folder_mut(&path);
                let index = parent
                    .folders
                    .iter()
                    .position(|f| f.name == name)
                    .expect("canonical path exists");
                let folder = parent.folders.remove(index);
                let inverse = Operation::RestoreFolder {
                    parent: path.clone(),
                    index,
                    folder,
                };
                path.push(name);
                Ok(Applied {
                    inverse,
                    touched: vec![Target::Folder(path)],
                })
            }
            Operation::RestoreFolder {
                parent,
                index,
                folder,
            } => {
                validate_name(&folder.name)?;
                let parent = self.canonical_folder(parent)?;
                let target = self.folder_mut(&parent);
                if name_taken(target, &folder.name) {
                    return Err(name_exists(&folder.name));
                }
                let at = (*index).min(target.folders.len());
                target.folders.insert(at, folder.clone());
                let mut path = parent;
                path.push(folder.name.clone());
                Ok(Applied {
                    inverse: Operation::DeleteFolder { path: path.clone() },
                    touched: vec![Target::Folder(path)],
                })
            }
            Operation::CreateSprite {
                folder,
                name,
                width,
                height,
                palette,
            } => {
                // The size is checked first so `too_big` wins over other errors.
                let sprite = Sprite::new(name, *width, *height, palette)?;
                validate_name(name)?;
                let palette = self
                    .palette(palette)
                    .ok_or_else(|| DocError::new("palette_not_found", format!("palette {palette:?} does not exist")))?
                    .name
                    .clone();
                let folder = self.canonical_folder(folder)?;
                let target = self.folder_mut(&folder);
                if name_taken(target, name) {
                    return Err(name_exists(name));
                }
                target.sprites.push(Sprite { palette, ..sprite });
                let sprite_ref = SpriteRef {
                    folder,
                    name: name.clone(),
                };
                Ok(Applied {
                    inverse: Operation::DeleteSprite {
                        sprite: sprite_ref.clone(),
                    },
                    touched: vec![Target::Sprite(sprite_ref)],
                })
            }
            Operation::DeleteSprite { sprite } => {
                let (folder, index) = self.locate(sprite)?;
                let removed = self.folder_mut(&folder).sprites.remove(index);
                let touched = Target::Sprite(SpriteRef {
                    folder: folder.clone(),
                    name: removed.name.clone(),
                });
                Ok(Applied {
                    inverse: Operation::RestoreSprite {
                        folder,
                        index,
                        sprite: removed,
                    },
                    touched: vec![touched],
                })
            }
            Operation::RestoreSprite {
                folder,
                index,
                sprite,
            } => {
                validate_name(&sprite.name)?;
                let palette = self.palette(&sprite.palette).ok_or_else(|| {
                    DocError::new(
                        "palette_not_found",
                        format!("palette {:?} does not exist", sprite.palette),
                    )
                })?;
                sprite.validate(palette)?;
                let folder = self.canonical_folder(folder)?;
                let target = self.folder_mut(&folder);
                if name_taken(target, &sprite.name) {
                    return Err(name_exists(&sprite.name));
                }
                let at = (*index).min(target.sprites.len());
                target.sprites.insert(at, sprite.clone());
                let sprite_ref = SpriteRef {
                    folder,
                    name: sprite.name.clone(),
                };
                Ok(Applied {
                    inverse: Operation::DeleteSprite {
                        sprite: sprite_ref.clone(),
                    },
                    touched: vec![Target::Sprite(sprite_ref)],
                })
            }
            Operation::RenameSprite { sprite, name } => {
                validate_name(name)?;
                let (folder, index) = self.locate(sprite)?;
                let target = self.folder_mut(&folder);
                let old = target.sprites[index].name.clone();
                if name_taken_by_other(target, name, &old) {
                    return Err(name_exists(name));
                }
                target.sprites[index].name = name.clone();
                let from = SpriteRef {
                    folder: folder.clone(),
                    name: old,
                };
                let to = SpriteRef {
                    folder,
                    name: name.clone(),
                };
                Ok(Applied {
                    inverse: Operation::RenameSprite {
                        sprite: to.clone(),
                        name: from.name.clone(),
                    },
                    touched: renamed(Target::Sprite(from), Target::Sprite(to)),
                })
            }
            Operation::RenameFolder { path, name } => {
                validate_name(name)?;
                let mut path = self.canonical_folder(path)?;
                let Some(old) = path.pop() else {
                    return Err(DocError::new("bad_path", "the project root cannot be renamed"));
                };
                let parent = self.folder_mut(&path);
                if name_taken_by_other(parent, name, &old) {
                    return Err(name_exists(name));
                }
                parent
                    .folders
                    .iter_mut()
                    .find(|f| f.name == old)
                    .expect("canonical path exists")
                    .name = name.clone();
                let mut from = path.clone();
                from.push(old.clone());
                let mut to = path;
                to.push(name.clone());
                Ok(Applied {
                    inverse: Operation::RenameFolder {
                        path: to.clone(),
                        name: old,
                    },
                    touched: renamed(Target::Folder(from), Target::Folder(to)),
                })
            }
            Operation::MoveSprite { sprite, to, index } => {
                let (from, at) = self.locate(sprite)?;
                let to = self.canonical_folder(to)?;
                if from == to {
                    return Err(DocError::new(
                        "same_folder",
                        "the sprite is already in that folder",
                    ));
                }
                let name = self.folder(&from).sprites[at].name.clone();
                if name_taken(self.folder(&to), &name) {
                    return Err(name_exists(&name));
                }
                let moved = self.folder_mut(&from).sprites.remove(at);
                let target = self.folder_mut(&to);
                let place = index.unwrap_or(target.sprites.len()).min(target.sprites.len());
                target.sprites.insert(place, moved);
                let old = SpriteRef {
                    folder: from,
                    name: name.clone(),
                };
                let new = SpriteRef { folder: to, name };
                Ok(Applied {
                    inverse: Operation::MoveSprite {
                        sprite: new.clone(),
                        to: old.folder.clone(),
                        index: Some(at),
                    },
                    touched: renamed(Target::Sprite(old), Target::Sprite(new)),
                })
            }
            Operation::DuplicateSprite { sprite, name } => {
                validate_name(name)?;
                let (folder, index) = self.locate(sprite)?;
                let target = self.folder_mut(&folder);
                if name_taken(target, name) {
                    return Err(name_exists(name));
                }
                let copy = Sprite {
                    name: name.clone(),
                    ..target.sprites[index].clone()
                };
                target.sprites.insert(index + 1, copy);
                let copy_ref = SpriteRef {
                    folder,
                    name: name.clone(),
                };
                Ok(Applied {
                    inverse: Operation::DeleteSprite {
                        sprite: copy_ref.clone(),
                    },
                    touched: vec![Target::Sprite(copy_ref)],
                })
            }
            Operation::SetPixels {
                sprite,
                layer,
                frame,
                pixels,
            } => {
                let (folder, index) = self.locate(sprite)?;
                let target = &self.folder(&folder).sprites[index];
                let palette = self.palette(&target.palette).ok_or_else(|| {
                    DocError::new("palette_not_found", format!("palette {:?} does not exist", target.palette))
                })?;
                if *layer >= target.layers.len() {
                    return Err(not_found("layer", &layer.to_string()));
                }
                check_frame(target, *frame)?;
                for pixel in pixels {
                    if pixel.x >= target.width || pixel.y >= target.height {
                        return Err(DocError::new(
                            "out_of_range",
                            format!(
                                "pixel ({}, {}) is outside the {}x{} sprite",
                                pixel.x, pixel.y, target.width, target.height
                            ),
                        ));
                    }
                    if !palette.accepts(pixel.key) {
                        return Err(DocError::new(
                            "unknown_key",
                            format!("key {:?} is not in palette {:?}", pixel.key, palette.name),
                        ));
                    }
                }
                let width = target.width as usize;
                let sprite_ref = SpriteRef {
                    folder: folder.clone(),
                    name: target.name.clone(),
                };
                let buffer = &mut self.folder_mut(&folder).sprites[index].layers[*layer].frames
                    [*frame as usize - 1]
                    .pixels;
                // Old values are saved in reverse, so a pixel set twice is restored right.
                let mut old = Vec::with_capacity(pixels.len());
                for pixel in pixels {
                    let at = pixel.y as usize * width + pixel.x as usize;
                    old.push(PixelChange {
                        x: pixel.x,
                        y: pixel.y,
                        key: buffer[at] as char,
                    });
                    buffer[at] = pixel.key as u8;
                }
                old.reverse();
                Ok(Applied {
                    inverse: Operation::SetPixels {
                        sprite: sprite_ref.clone(),
                        layer: *layer,
                        frame: *frame,
                        pixels: old,
                    },
                    touched: vec![Target::Sprite(sprite_ref)],
                })
            }
            Operation::AddFrame {
                sprite,
                duration_ms,
            } => {
                if *duration_ms == 0 {
                    return Err(DocError::new("bad_duration", "a frame lasts at least 1 ms"));
                }
                let (folder, index) = self.locate(sprite)?;
                let target = &mut self.folder_mut(&folder).sprites[index];
                for layer in &mut target.layers {
                    layer.frames.push(Frame::blank(target.width, target.height));
                }
                target.frame_durations.push(*duration_ms);
                let sprite_ref = SpriteRef {
                    folder,
                    name: target.name.clone(),
                };
                Ok(Applied {
                    inverse: Operation::DeleteFrame {
                        sprite: sprite_ref.clone(),
                        frame: target.frame_count() as u32,
                    },
                    touched: vec![Target::Sprite(sprite_ref)],
                })
            }
            Operation::DeleteFrame { sprite, frame } => {
                let (folder, index) = self.locate(sprite)?;
                let target = &mut self.folder_mut(&folder).sprites[index];
                check_frame(target, *frame)?;
                if target.frame_count() == 1 {
                    return Err(DocError::new(
                        "last_frame",
                        "a sprite needs at least one frame",
                    ));
                }
                let at = *frame as usize - 1;
                let old_tags = target.tags.clone();
                let layers = target
                    .layers
                    .iter_mut()
                    .map(|layer| layer.frames.remove(at).pixels)
                    .collect();
                let duration_ms = target.frame_durations.remove(at);
                shrink_tags(&mut target.tags, *frame);
                let sprite_ref = SpriteRef {
                    folder,
                    name: target.name.clone(),
                };
                Ok(Applied {
                    inverse: Operation::RestoreFrame {
                        sprite: sprite_ref.clone(),
                        frame: *frame,
                        layers,
                        duration_ms,
                        tags: old_tags,
                    },
                    touched: vec![Target::Sprite(sprite_ref)],
                })
            }
            Operation::RestoreFrame {
                sprite,
                frame,
                layers,
                duration_ms,
                tags,
            } => {
                let (folder, index) = self.locate(sprite)?;
                let target = &mut self.folder_mut(&folder).sprites[index];
                let pixel_count = target.width as usize * target.height as usize;
                if layers.len() != target.layers.len()
                    || layers.iter().any(|l| l.len() != pixel_count)
                    || *frame < 1
                    || *frame as usize > target.frame_count() + 1
                {
                    return Err(DocError::new("bad_frame", "the frame does not fit the sprite"));
                }
                let at = *frame as usize - 1;
                for (layer, pixels) in target.layers.iter_mut().zip(layers) {
                    layer.frames.insert(at, Frame { pixels: pixels.clone() });
                }
                target.frame_durations.insert(at, *duration_ms);
                target.tags = tags.clone();
                let sprite_ref = SpriteRef {
                    folder,
                    name: target.name.clone(),
                };
                Ok(Applied {
                    inverse: Operation::DeleteFrame {
                        sprite: sprite_ref.clone(),
                        frame: *frame,
                    },
                    touched: vec![Target::Sprite(sprite_ref)],
                })
            }
            Operation::AddTag { sprite, tag, index } => {
                let (folder, sprite_index) = self.locate(sprite)?;
                let target = &mut self.folder_mut(&folder).sprites[sprite_index];
                tag.validate(target.frame_count())?;
                if target.tags.iter().any(|t| same_name(&t.name, &tag.name)) {
                    return Err(DocError::new(
                        "tag_exists",
                        format!("tag {:?} already exists", tag.name),
                    ));
                }
                let at = index.unwrap_or(target.tags.len()).min(target.tags.len());
                target.tags.insert(at, tag.clone());
                let sprite_ref = SpriteRef {
                    folder,
                    name: target.name.clone(),
                };
                Ok(Applied {
                    inverse: Operation::RemoveTag {
                        sprite: sprite_ref.clone(),
                        name: tag.name.clone(),
                    },
                    touched: vec![Target::Sprite(sprite_ref)],
                })
            }
            Operation::RemoveTag { sprite, name } => {
                let (folder, sprite_index) = self.locate(sprite)?;
                let target = &mut self.folder_mut(&folder).sprites[sprite_index];
                let at = target
                    .tags
                    .iter()
                    .position(|t| same_name(&t.name, name))
                    .ok_or_else(|| not_found("tag", name))?;
                let tag = target.tags.remove(at);
                let sprite_ref = SpriteRef {
                    folder,
                    name: target.name.clone(),
                };
                Ok(Applied {
                    inverse: Operation::AddTag {
                        sprite: sprite_ref.clone(),
                        tag,
                        index: Some(at),
                    },
                    touched: vec![Target::Sprite(sprite_ref)],
                })
            }
        }
    }

    pub(super) fn locate_sprite(&self, sprite: &SpriteRef) -> Option<(Vec<String>, usize)> {
        self.locate(sprite).ok()
    }

    pub(super) fn folder_at_mut(&mut self, path: &[String]) -> &mut Folder {
        self.folder_mut(path)
    }

    pub(super) fn all_sprites(&self) -> impl Iterator<Item = &Sprite> {
        all_sprites(&self.root)
    }

    /// Finds a sprite ignoring case in names.
    pub fn find_sprite(&self, sprite: &SpriteRef) -> Option<&Sprite> {
        let (folder, index) = self.locate(sprite).ok()?;
        Some(&self.folder(&folder).sprites[index])
    }

    /// Finds a folder by its path, ignoring case in names. An empty path is the root.
    pub fn find_folder(&self, path: &[String]) -> Option<&Folder> {
        let real = self.canonical_folder(path).ok()?;
        Some(self.folder(&real))
    }

    /// Follows a folder path ignoring case and returns it with the real names.
    fn canonical_folder(&self, path: &[String]) -> Result<Vec<String>, DocError> {
        let mut folder = &self.root;
        let mut real = Vec::new();
        for name in path {
            folder = folder
                .folders
                .iter()
                .find(|f| same_name(&f.name, name))
                .ok_or_else(|| not_found("folder", name))?;
            real.push(folder.name.clone());
        }
        Ok(real)
    }

    /// The folder at a path of real names (see `canonical_folder`).
    fn folder(&self, path: &[String]) -> &Folder {
        path.iter().fold(&self.root, |folder, name| {
            folder
                .folders
                .iter()
                .find(|f| f.name == *name)
                .expect("canonical path exists")
        })
    }

    fn folder_mut(&mut self, path: &[String]) -> &mut Folder {
        path.iter().fold(&mut self.root, |folder, name| {
            folder
                .folders
                .iter_mut()
                .find(|f| f.name == *name)
                .expect("canonical path exists")
        })
    }

    /// Finds a sprite ignoring case. Returns its real folder path and its index.
    pub(super) fn locate(&self, sprite: &SpriteRef) -> Result<(Vec<String>, usize), DocError> {
        let folder = self.canonical_folder(&sprite.folder)?;
        let index = self
            .folder(&folder)
            .sprites
            .iter()
            .position(|s| same_name(&s.name, &sprite.name))
            .ok_or_else(|| not_found("sprite", &sprite.name))?;
        Ok((folder, index))
    }
}

fn all_sprites(folder: &Folder) -> Box<dyn Iterator<Item = &Sprite> + '_> {
    Box::new(
        folder
            .sprites
            .iter()
            .chain(folder.folders.iter().flat_map(all_sprites)),
    )
}

// Sprites and folders share the names of their folder.
fn name_taken(folder: &Folder, name: &str) -> bool {
    folder.sprites.iter().any(|s| same_name(&s.name, name))
        || folder.folders.iter().any(|f| same_name(&f.name, name))
}

// Like `name_taken`, but the item called `own` (the one being renamed) does
// not count, so a name can change only in case.
fn name_taken_by_other(folder: &Folder, name: &str, own: &str) -> bool {
    folder
        .sprites
        .iter()
        .any(|s| same_name(&s.name, name) && s.name != own)
        || folder
            .folders
            .iter()
            .any(|f| same_name(&f.name, name) && f.name != own)
}

// The touched parts of a rename or a move: the old place, then the new one.
// When the place did not change, it is one part.
fn renamed(from: Target, to: Target) -> Vec<Target> {
    if from == to {
        vec![from]
    } else {
        vec![from, to]
    }
}

fn check_frame(sprite: &Sprite, frame: u32) -> Result<(), DocError> {
    if frame < 1 || frame as usize > sprite.frame_count() {
        return Err(DocError::new(
            "bad_frame",
            format!(
                "frame {frame} does not exist, the sprite has {} frames",
                sprite.frame_count()
            ),
        ));
    }
    Ok(())
}

// Tags follow the frames: they shrink or move, and a tag left without
// frames is removed.
fn shrink_tags(tags: &mut Vec<Tag>, deleted: u32) {
    for tag in tags.iter_mut() {
        if deleted < tag.from {
            tag.from -= 1;
            tag.to -= 1;
        } else if deleted <= tag.to {
            tag.to -= 1;
        }
    }
    tags.retain(|tag| tag.to >= tag.from);
}

fn name_exists(name: &str) -> DocError {
    DocError::new("name_exists", format!("the name {name:?} is already used here"))
}

fn not_found(what: &str, name: &str) -> DocError {
    DocError::new("not_found", format!("{what} {name:?} does not exist"))
}
