//! The rules for the colors of a palette: keys, names, values and the limit.

use super::error::DocError;
use super::model::{Color, Folder, MAX_COLORS, Palette, Project, Sprite, TRANSPARENT};
use super::names::same_name;
use super::operation::{Applied, Operation, PixelBatch, PixelChange, SpriteRef, Target};
use std::collections::HashMap;

impl Palette {
    /// Checks a color that is added to this palette, or that replaces the
    /// color with the key `own` (the one being edited). The other colors are
    /// the ones that its key and name must not repeat.
    pub fn check_color(&self, own: Option<char>, color: &Color) -> Result<(), DocError> {
        // The limit comes first: a full palette has no free key left anyway.
        if own.is_none() && self.colors.len() >= MAX_COLORS {
            return Err(DocError::new(
                "palette_full",
                format!("a palette has at most {MAX_COLORS} colors"),
            ));
        }
        if color.key == TRANSPARENT || color.key == ' ' {
            return Err(DocError::new(
                "reserved_key",
                "\".\" means transparent and the space is not a key, choose another key",
            ));
        }
        if !color.key.is_ascii_graphic() {
            return Err(DocError::new(
                "bad_key",
                "a key is one printable ASCII character",
            ));
        }
        if color.name.trim().is_empty() {
            return Err(DocError::new("bad_name", "a color needs a name"));
        }
        if !is_hex(&color.hex) {
            return Err(DocError::new(
                "bad_color",
                format!("{:?} is not a color, expected #RRGGBB", color.hex),
            ));
        }
        let others = || self.colors.iter().filter(|c| Some(c.key) != own);
        if others().any(|c| c.key == color.key) {
            return Err(DocError::new(
                "duplicate_key",
                format!("the key {:?} is already used by another color", color.key),
            ));
        }
        if others().any(|c| c.name.to_lowercase() == color.name.trim().to_lowercase()) {
            return Err(DocError::new(
                "duplicate_name",
                format!("the name {:?} is already used by another color", color.name),
            ));
        }
        Ok(())
    }
}

/// Checks a whole list of colors, as the editor sends it on Done: each color
/// is checked against the ones before it.
pub fn check_colors(colors: &[Color]) -> Result<(), DocError> {
    let mut growing = Palette {
        name: String::new(),
        colors: Vec::new(),
    };
    for color in colors {
        growing.check_color(None, color)?;
        growing.colors.push(color.clone());
    }
    Ok(())
}

fn is_hex(hex: &str) -> bool {
    hex.len() == 7 && hex.starts_with('#') && hex[1..].bytes().all(|b| b.is_ascii_hexdigit())
}

/// One color of the list the editor sends on Done. `from` is the key the
/// color had when the editor opened, or `None` for a color added there.
#[derive(Debug, Clone, PartialEq)]
pub struct ColorEdit {
    pub from: Option<char>,
    pub color: Color,
}

impl Project {
    /// The operations of one palette editor session. `colors` is the final
    /// list. A color of the palette that is not in it is removed: if sprites
    /// use it, `replacements` (removed key, key that stays, or `.`) says what
    /// their pixels become. Nothing is applied here. An edit that changes
    /// nothing gives no operations.
    pub fn palette_edit_operations(
        &self,
        palette: &str,
        colors: &[ColorEdit],
        replacements: &[(char, char)],
    ) -> Result<Vec<Operation>, DocError> {
        let old = self
            .palette(palette)
            .ok_or_else(|| DocError::new("not_found", format!("palette {palette:?} does not exist")))?;
        let new: Vec<Color> = colors.iter().map(|c| c.color.clone()).collect();
        check_colors(&new)?;
        // Where each old key goes. A removed color goes to its replacement.
        let mut remap: HashMap<char, char> = HashMap::new();
        for edit in colors {
            let Some(from) = edit.from else { continue };
            if !old.colors.iter().any(|c| c.key == from) || remap.insert(from, edit.color.key).is_some() {
                return Err(DocError::new(
                    "not_found",
                    format!("color {from:?} is not in palette {:?} or is listed twice", old.name),
                ));
            }
        }
        let removed: Vec<char> = old
            .colors
            .iter()
            .map(|c| c.key)
            .filter(|k| !remap.contains_key(k))
            .collect();
        let kept = remap.clone();
        for key in &removed {
            let used = self.sprites_of(&old.name).any(|(_, s)| uses_key(s, *key));
            match replacements.iter().find(|(from, _)| from == key) {
                Some((_, to)) if *to == TRANSPARENT => {
                    remap.insert(*key, TRANSPARENT);
                }
                Some((_, to)) if kept.contains_key(to) => {
                    remap.insert(*key, kept[to]);
                }
                Some(_) => {
                    return Err(DocError::new(
                        "bad_replacement",
                        format!("the replacement of {key:?} must be a color that stays, or \".\""),
                    ));
                }
                None if used => {
                    return Err(DocError::new(
                        "replacement_needed",
                        format!("color {key:?} is used by a sprite, choose a replacement"),
                    ));
                }
                None => {}
            }
        }
        // The pixels that change, in every frame of every layer.
        let mut batches = Vec::new();
        for (sprite_ref, sprite) in self.sprites_of(&old.name) {
            for (layer, l) in sprite.layers.iter().enumerate() {
                for (i, frame) in l.frames.iter().enumerate() {
                    let pixels: Vec<PixelChange> = frame
                        .pixels
                        .iter()
                        .enumerate()
                        .filter_map(|(at, key)| {
                            let to = *remap.get(&(*key as char))?;
                            (to != *key as char).then(|| PixelChange {
                                x: (at as u32) % sprite.width,
                                y: (at as u32) / sprite.width,
                                key: to,
                            })
                        })
                        .collect();
                    if !pixels.is_empty() {
                        batches.push(PixelBatch {
                            sprite: sprite_ref.clone(),
                            layer,
                            frame: i as u32 + 1,
                            pixels,
                        });
                    }
                }
            }
        }
        if batches.is_empty() && new == old.colors {
            return Ok(Vec::new());
        }
        Ok(vec![Operation::SetPalette {
            palette: old.name.clone(),
            colors: new,
            pixels: batches,
        }])
    }

    /// The keys that sprites using the palette have in their pixels.
    pub fn used_keys(&self, palette: &str) -> std::collections::HashSet<char> {
        self.sprites_of(palette)
            .flat_map(|(_, s)| s.layers.iter().flat_map(|l| l.frames.iter()))
            .flat_map(|f| f.pixels.iter().map(|k| *k as char))
            .collect()
    }

    /// Every sprite that uses the palette, with where it is.
    fn sprites_of<'a>(&'a self, palette: &'a str) -> impl Iterator<Item = (SpriteRef, &'a Sprite)> {
        let mut found = Vec::new();
        collect(&self.root, &mut Vec::new(), palette, &mut found);
        found.into_iter()
    }

    /// Applies `Operation::SetPalette`: all or nothing. The inverse is the
    /// same operation with the old colors and the old pixels.
    pub(super) fn set_palette(
        &mut self,
        palette: &str,
        colors: &[Color],
        batches: &[PixelBatch],
    ) -> Result<Applied, DocError> {
        let at = self
            .palettes
            .iter()
            .position(|p| same_name(&p.name, palette))
            .ok_or_else(|| DocError::new("not_found", format!("palette {palette:?} does not exist")))?;
        let name = self.palettes[at].name.clone();
        let new = Palette {
            name: name.clone(),
            colors: colors.to_vec(),
        };
        new.validate()?;
        // Check every batch before changing anything.
        let mut places = Vec::with_capacity(batches.len());
        for batch in batches {
            let (folder, index) = self.locate(&batch.sprite)?;
            let sprite = &self.folder_at_mut(&folder).sprites[index];
            let bad = |message: String| DocError::new("bad_pixels", message);
            if !same_name(&sprite.palette, &name) {
                return Err(bad(format!("sprite {:?} does not use palette {name:?}", sprite.name)));
            }
            let frame = batch.frame as usize;
            if frame < 1 || frame > sprite.frame_count() || batch.layer >= sprite.layers.len() {
                return Err(bad(format!("sprite {:?} has no such layer or frame", sprite.name)));
            }
            if batch.pixels.iter().any(|p| p.x >= sprite.width || p.y >= sprite.height) {
                return Err(bad(format!("a pixel is outside sprite {:?}", sprite.name)));
            }
            places.push((folder, index));
        }
        let old_colors = std::mem::replace(&mut self.palettes[at].colors, colors.to_vec());
        let mut inverse = Vec::with_capacity(batches.len());
        let mut touched = vec![Target::Palette(name.clone())];
        for (batch, (folder, index)) in batches.iter().zip(&places) {
            let sprite = &mut self.folder_at_mut(folder).sprites[*index];
            let width = sprite.width as usize;
            let buffer = &mut sprite.layers[batch.layer].frames[batch.frame as usize - 1].pixels;
            let mut old = Vec::with_capacity(batch.pixels.len());
            for pixel in &batch.pixels {
                let place = pixel.y as usize * width + pixel.x as usize;
                old.push(PixelChange {
                    x: pixel.x,
                    y: pixel.y,
                    key: buffer[place] as char,
                });
                buffer[place] = pixel.key as u8;
            }
            old.reverse();
            let target = SpriteRef {
                folder: folder.clone(),
                name: sprite.name.clone(),
            };
            if !touched.contains(&Target::Sprite(target.clone())) {
                touched.push(Target::Sprite(target.clone()));
            }
            inverse.push(PixelBatch {
                sprite: target,
                layer: batch.layer,
                frame: batch.frame,
                pixels: old,
            });
        }
        inverse.reverse();
        let applied = Applied {
            inverse: Operation::SetPalette {
                palette: name.clone(),
                colors: old_colors.clone(),
                pixels: inverse,
            },
            touched,
        };
        // Every sprite of the palette must still only use keys it has.
        let broken = self
            .sprites_of(&name)
            .find_map(|(_, s)| s.validate(&new).err());
        if let Some(error) = broken {
            let Operation::SetPalette { colors, pixels, .. } = &applied.inverse else {
                unreachable!()
            };
            self.set_palette(&name, colors, pixels)
                .map_err(|e| DocError::new("internal", format!("could not roll back: {e}")))?;
            return Err(error);
        }
        Ok(applied)
    }
}

fn collect<'a>(
    folder: &'a Folder,
    path: &mut Vec<String>,
    palette: &str,
    found: &mut Vec<(SpriteRef, &'a Sprite)>,
) {
    for sprite in &folder.sprites {
        if same_name(&sprite.palette, palette) {
            found.push((
                SpriteRef {
                    folder: path.clone(),
                    name: sprite.name.clone(),
                },
                sprite,
            ));
        }
    }
    for child in &folder.folders {
        path.push(child.name.clone());
        collect(child, path, palette, found);
        path.pop();
    }
}

fn uses_key(sprite: &Sprite, key: char) -> bool {
    sprite
        .layers
        .iter()
        .flat_map(|l| l.frames.iter())
        .any(|f| f.pixels.contains(&(key as u8)))
}
