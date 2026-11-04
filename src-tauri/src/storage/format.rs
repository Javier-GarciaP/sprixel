use crate::document::{Anchor, Color, Frame, Layer, Palette, Part, Sprite, Tag, TagMode};
use super::migrate::FileKind;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// The `formatVersion` that this app reads and writes.
pub const FORMAT_VERSION: u32 = 1;

/// The content of `sprixel.json`.
#[derive(Debug, Clone, PartialEq)]
pub struct Manifest {
    pub name: String,
    pub active_palette: Option<String>,
}

/// Files without a `formatVersion` are the first ones: version 1.
fn first_version() -> u32 {
    1
}

impl SpriteFile {
    /// Fills what the first files did not have: 100 ms for every frame.
    fn fill_old_defaults(&mut self) {
        if self.frame_durations.is_empty() {
            let frames = self.layers.first().map_or(0, |l| l.frames.len());
            self.frame_durations = vec![crate::document::DEFAULT_FRAME_MS; frames];
        }
    }
}

// The structs below are the shape of the JSON files. Their field order is
// the order in the files, so writing is always the same.

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ManifestFile {
    #[serde(default = "first_version")]
    format_version: u32,
    name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    active_palette: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PaletteFile {
    #[serde(default = "first_version")]
    format_version: u32,
    colors: Vec<ColorFile>,
}

#[derive(Serialize, Deserialize)]
struct ColorFile {
    key: char,
    name: String,
    hex: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SpriteFile {
    #[serde(default = "first_version")]
    format_version: u32,
    width: u32,
    height: u32,
    palette: String,
    /// The first files had no durations: they get 100 ms per frame.
    #[serde(default)]
    frame_durations: Vec<u32>,
    layers: Vec<LayerFile>,
    tags: Vec<TagFile>,
    /// Added after version 1 first shipped: old files do not have it, and a
    /// sprite with no anchors does not write it.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    anchors: Vec<AnchorFile>,
    /// Added after version 1 first shipped: named rectangles. Old files do not
    /// have it, and a sprite with no parts does not write it.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    parts: Vec<PartFile>,
}

#[derive(Serialize, Deserialize)]
struct PartFile {
    name: String,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

#[derive(Serialize, Deserialize)]
struct AnchorFile {
    name: String,
    x: u32,
    y: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    frame: Option<u32>,
}

#[derive(Serialize, Deserialize)]
struct LayerFile {
    name: String,
    /// Frames, each one an array of row strings.
    frames: Vec<Vec<String>>,
}

#[derive(Serialize, Deserialize)]
struct TagFile {
    name: String,
    from: u32,
    to: u32,
    fps: u32,
    mode: String,
}

pub fn manifest_to_text(manifest: &Manifest) -> String {
    to_text(&ManifestFile {
        format_version: FORMAT_VERSION,
        name: manifest.name.clone(),
        active_palette: manifest.active_palette.clone(),
    })
}

pub fn manifest_from_text(text: &str) -> Result<Manifest, String> {
    let file: ManifestFile = from_text(text)?;
    Ok(Manifest {
        name: file.name,
        active_palette: file.active_palette,
    })
}

pub fn palette_to_text(palette: &Palette) -> String {
    to_text(&PaletteFile {
        format_version: FORMAT_VERSION,
        colors: palette
            .colors
            .iter()
            .map(|c| ColorFile {
                key: c.key,
                name: c.name.clone(),
                hex: c.hex.clone(),
            })
            .collect(),
    })
}

/// The palette name is the name of its file, so it is given here.
pub fn palette_from_text(name: &str, text: &str) -> Result<Palette, String> {
    let file: PaletteFile = from_text(text)?;
    let palette = Palette {
        name: name.to_string(),
        colors: file
            .colors
            .into_iter()
            .map(|c| Color {
                key: c.key,
                name: c.name,
                hex: c.hex,
            })
            .collect(),
    };
    palette.validate().map_err(|e| e.to_string())?;
    Ok(palette)
}

pub fn sprite_to_text(sprite: &Sprite) -> String {
    let width = sprite.width as usize;
    let rows = |frame: &Frame| -> Vec<String> {
        frame
            .pixels
            .chunks(width)
            .map(|row| String::from_utf8_lossy(row).into_owned())
            .collect()
    };
    to_text(&SpriteFile {
        format_version: FORMAT_VERSION,
        width: sprite.width,
        height: sprite.height,
        palette: sprite.palette.clone(),
        frame_durations: sprite.frame_durations.clone(),
        layers: sprite
            .layers
            .iter()
            .map(|layer| LayerFile {
                name: layer.name.clone(),
                frames: layer.frames.iter().map(rows).collect(),
            })
            .collect(),
        tags: sprite
            .tags
            .iter()
            .map(|t| TagFile {
                name: t.name.clone(),
                from: t.from,
                to: t.to,
                fps: t.fps,
                mode: t.mode.as_str().to_string(),
            })
            .collect(),
        anchors: sprite
            .anchors
            .iter()
            .map(|a| AnchorFile { name: a.name.clone(), x: a.x, y: a.y, frame: a.frame })
            .collect(),
        parts: sprite
            .parts
            .iter()
            .map(|p| PartFile { name: p.name.clone(), x: p.x, y: p.y, width: p.width, height: p.height })
            .collect(),
    })
}

/// The sprite name is the name of its file, so it is given here. The sprite
/// is checked against its palette, so a file that breaks a rule is refused.
pub fn sprite_from_text(name: &str, text: &str, palette: &Palette) -> Result<Sprite, String> {
    let mut file: SpriteFile = from_text(text)?;
    file.fill_old_defaults();
    if file.palette.to_lowercase() != palette.name.to_lowercase() {
        return Err(format!(
            "the sprite uses palette {:?}, not {:?}",
            file.palette, palette.name
        ));
    }
    let mut layers = Vec::new();
    for layer in file.layers {
        let mut frames = Vec::new();
        for (number, rows) in layer.frames.iter().enumerate() {
            frames.push(frame_from_rows(rows, file.width, file.height, number + 1)?);
        }
        layers.push(Layer {
            name: layer.name,
            frames,
        });
    }
    let mut tags = Vec::new();
    for tag in file.tags {
        let mode = TagMode::parse(&tag.mode)
            .ok_or_else(|| format!("tag {:?} has the unknown mode {:?}", tag.name, tag.mode))?;
        tags.push(Tag {
            name: tag.name,
            from: tag.from,
            to: tag.to,
            fps: tag.fps,
            mode,
        });
    }
    let sprite = Sprite {
        name: name.to_string(),
        width: file.width,
        height: file.height,
        palette: palette.name.clone(),
        frame_durations: file.frame_durations,
        layers,
        tags,
        anchors: file
            .anchors
            .into_iter()
            .map(|a| Anchor { name: a.name, x: a.x, y: a.y, frame: a.frame })
            .collect(),
        parts: file
            .parts
            .into_iter()
            .map(|p| Part { name: p.name, x: p.x, y: p.y, width: p.width, height: p.height })
            .collect(),
    };
    sprite.validate(palette).map_err(|e| e.to_string())?;
    Ok(sprite)
}

fn frame_from_rows(
    rows: &[String],
    width: u32,
    height: u32,
    number: usize,
) -> Result<Frame, String> {
    if rows.len() != height as usize {
        return Err(format!(
            "frame {number} has {} rows, expected {height}",
            rows.len()
        ));
    }
    let mut pixels = Vec::with_capacity(width as usize * height as usize);
    for (i, row) in rows.iter().enumerate() {
        if !row.is_ascii() || row.len() != width as usize {
            return Err(format!(
                "frame {number} row {} has {} columns, expected {width}",
                i + 1,
                row.chars().count()
            ));
        }
        pixels.extend_from_slice(row.as_bytes());
    }
    Ok(Frame { pixels })
}

fn to_text<T: Serialize>(file: &T) -> String {
    let mut text = serde_json::to_string_pretty(file).expect("a file struct always serializes");
    text.push('\n');
    text
}

/// Reads a file of the current version. Other versions are refused with a
/// reason: older files must be migrated first, newer ones need a newer app.
fn from_text<T: for<'de> Deserialize<'de>>(text: &str) -> Result<T, String> {
    let value: Value = serde_json::from_str(text).map_err(|e| e.to_string())?;
    match version_of(&value)? {
        v if v > FORMAT_VERSION => Err(format!(
            "saved with a newer Sprixel (format version {v}); update the app"
        )),
        v if v < FORMAT_VERSION => Err(format!(
            "format version {v} is old and needs a migration"
        )),
        _ => serde_json::from_value(value).map_err(|e| e.to_string()),
    }
}

/// The `formatVersion` of a parsed file.
pub(super) fn version_of(value: &Value) -> Result<u32, String> {
    // The first files had no `formatVersion`: they are version 1.
    let Some(version) = value.get("formatVersion") else {
        return Ok(1);
    };
    let version = version.as_u64().ok_or("formatVersion must be a whole number")?;
    Ok(u32::try_from(version).unwrap_or(u32::MAX))
}

/// The text of a file of the current version, written in the canonical
/// field order. Used after a migration.
pub(super) fn canonical_text(kind: FileKind, value: Value) -> Result<String, String> {
    fn rewrite<T: Serialize + for<'de> Deserialize<'de>>(value: Value) -> Result<String, String> {
        let file: T = serde_json::from_value(value).map_err(|e| e.to_string())?;
        Ok(to_text(&file))
    }
    match kind {
        FileKind::Manifest => rewrite::<ManifestFile>(value),
        FileKind::Palette => rewrite::<PaletteFile>(value),
        FileKind::Sprite => {
            let mut file: SpriteFile = serde_json::from_value(value).map_err(|e| e.to_string())?;
            file.fill_old_defaults();
            Ok(to_text(&file))
        }
    }
}

/// The palette name written in a sprite file. Read first, to find the palette
/// that the whole file is checked against.
pub fn sprite_palette_name(text: &str) -> Result<String, String> {
    let value: Value = serde_json::from_str(text).map_err(|e| e.to_string())?;
    match value.get("palette").and_then(Value::as_str) {
        Some(name) => Ok(name.to_string()),
        None => Err("the sprite file has no palette".to_string()),
    }
}
