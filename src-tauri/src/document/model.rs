use super::error::DocError;

pub const MAX_SIZE: u32 = 512;
pub const MAX_COLORS: usize = 93;
/// The pixel key of a transparent pixel. It is never a palette color.
pub const TRANSPARENT: char = '.';

/// A whole project. `revision` grows by one with every applied operation or
/// group of operations.
#[derive(Debug, Clone, PartialEq)]
pub struct Project {
    pub name: String,
    pub active_palette: Option<String>,
    pub palettes: Vec<Palette>,
    /// The project root. Its own name is not used.
    pub root: Folder,
    pub revision: u64,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Folder {
    pub name: String,
    pub folders: Vec<Folder>,
    pub sprites: Vec<Sprite>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Palette {
    pub name: String,
    pub colors: Vec<Color>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Color {
    /// One printable ASCII character, not a space and not `.`. Case sensitive.
    pub key: char,
    pub name: String,
    /// `#RRGGBB`
    pub hex: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Sprite {
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub palette: String,
    /// One duration in milliseconds per frame.
    pub frame_durations: Vec<u32>,
    pub layers: Vec<Layer>,
    pub tags: Vec<Tag>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Layer {
    pub name: String,
    pub frames: Vec<Frame>,
}

/// The pixels of one frame: `width * height` palette keys, row by row.
#[derive(Debug, Clone, PartialEq)]
pub struct Frame {
    pub pixels: Vec<u8>,
}

/// A named range of frames. Frames are numbered from 1, both ends included.
#[derive(Debug, Clone, PartialEq)]
pub struct Tag {
    pub name: String,
    pub from: u32,
    pub to: u32,
    pub fps: u32,
    pub mode: TagMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TagMode {
    Loop,
    Once,
    PingPong,
}

impl TagMode {
    pub fn as_str(self) -> &'static str {
        match self {
            TagMode::Loop => "loop",
            TagMode::Once => "once",
            TagMode::PingPong => "pingpong",
        }
    }

    pub fn parse(text: &str) -> Option<TagMode> {
        match text {
            "loop" => Some(TagMode::Loop),
            "once" => Some(TagMode::Once),
            "pingpong" => Some(TagMode::PingPong),
            _ => None,
        }
    }
}

impl Project {
    pub fn new(name: &str) -> Project {
        Project {
            name: name.to_string(),
            active_palette: None,
            palettes: Vec::new(),
            root: Folder::default(),
            revision: 0,
        }
    }

    /// Finds a palette by name, ignoring upper and lower case.
    pub fn palette(&self, name: &str) -> Option<&Palette> {
        self.palettes
            .iter()
            .find(|p| p.name.to_lowercase() == name.to_lowercase())
    }
}

impl Palette {
    pub fn validate(&self) -> Result<(), DocError> {
        if self.colors.len() > MAX_COLORS {
            return Err(DocError::new(
                "bad_palette",
                format!("a palette has at most {MAX_COLORS} colors"),
            ));
        }
        for (i, color) in self.colors.iter().enumerate() {
            let printable = color.key.is_ascii_graphic();
            if !printable || color.key == TRANSPARENT {
                return Err(DocError::new(
                    "bad_palette",
                    format!("key {:?} must be a printable ASCII character, not a space and not \".\"", color.key),
                ));
            }
            if !is_hex_color(&color.hex) {
                return Err(DocError::new(
                    "bad_palette",
                    format!("color {:?} has the invalid hex {:?}, expected #RRGGBB", color.key, color.hex),
                ));
            }
            if self.colors[..i].iter().any(|c| c.key == color.key) {
                return Err(DocError::new(
                    "bad_palette",
                    format!("key {:?} is used twice", color.key),
                ));
            }
        }
        Ok(())
    }

    /// True for `.` and for every key of the palette.
    pub fn accepts(&self, key: char) -> bool {
        key == TRANSPARENT || self.colors.iter().any(|c| c.key == key)
    }
}

fn is_hex_color(hex: &str) -> bool {
    hex.len() == 7 && hex.starts_with('#') && hex[1..].bytes().all(|b| b.is_ascii_hexdigit())
}

impl Folder {
    pub fn new(name: &str) -> Folder {
        Folder {
            name: name.to_string(),
            ..Folder::default()
        }
    }
}

impl Tag {
    /// Checks the tag against a sprite with `frame_count` frames.
    pub fn validate(&self, frame_count: usize) -> Result<(), DocError> {
        if self.from < 1 || self.from > self.to || self.to as usize > frame_count {
            return Err(DocError::new(
                "bad_range",
                format!(
                    "tag {:?} has frames {} to {}, the sprite has {frame_count} frames",
                    self.name, self.from, self.to
                ),
            ));
        }
        if !(1..=60).contains(&self.fps) {
            return Err(DocError::new(
                "bad_fps",
                format!("tag {:?} has {} fps, expected 1 to 60", self.name, self.fps),
            ));
        }
        Ok(())
    }
}

impl Sprite {
    /// A sprite with one layer and one frame, fully transparent.
    pub fn new(name: &str, width: u32, height: u32, palette: &str) -> Result<Sprite, DocError> {
        check_size(width, height)?;
        Ok(Sprite {
            name: name.to_string(),
            width,
            height,
            palette: palette.to_string(),
            frame_durations: vec![100],
            layers: vec![Layer {
                name: "Layer 1".to_string(),
                frames: vec![Frame::blank(width, height)],
            }],
            tags: Vec::new(),
        })
    }

    pub fn frame_count(&self) -> usize {
        self.frame_durations.len()
    }

    /// Checks every rule of a sprite against its palette.
    pub fn validate(&self, palette: &Palette) -> Result<(), DocError> {
        check_size(self.width, self.height)?;
        let bad = |message: String| DocError::new("bad_sprite", message);
        if self.layers.is_empty() {
            return Err(bad(format!("sprite {:?} has no layers", self.name)));
        }
        if self.frame_durations.is_empty() {
            return Err(bad(format!("sprite {:?} has no frames", self.name)));
        }
        let pixel_count = self.width as usize * self.height as usize;
        for layer in &self.layers {
            if layer.frames.len() != self.frame_count() {
                return Err(bad(format!(
                    "layer {:?} has {} frames, expected {}",
                    layer.name,
                    layer.frames.len(),
                    self.frame_count()
                )));
            }
            for (i, frame) in layer.frames.iter().enumerate() {
                if frame.pixels.len() != pixel_count {
                    return Err(bad(format!(
                        "frame {} of layer {:?} has {} pixels, expected {pixel_count}",
                        i + 1,
                        layer.name,
                        frame.pixels.len()
                    )));
                }
                if let Some(key) = frame.pixels.iter().find(|k| !palette.accepts(**k as char)) {
                    return Err(DocError::new(
                        "unknown_key",
                        format!(
                            "key {:?} is not in palette {:?}",
                            *key as char, palette.name
                        ),
                    ));
                }
            }
        }
        for (i, tag) in self.tags.iter().enumerate() {
            tag.validate(self.frame_count())?;
            if self.tags[..i]
                .iter()
                .any(|t| t.name.to_lowercase() == tag.name.to_lowercase())
            {
                return Err(DocError::new(
                    "tag_exists",
                    format!("tag {:?} is used twice", tag.name),
                ));
            }
        }
        Ok(())
    }
}

impl Frame {
    pub fn blank(width: u32, height: u32) -> Frame {
        Frame {
            pixels: vec![TRANSPARENT as u8; width as usize * height as usize],
        }
    }
}

fn check_size(width: u32, height: u32) -> Result<(), DocError> {
    if width > MAX_SIZE || height > MAX_SIZE {
        return Err(DocError::new(
            "too_big",
            format!("{width}x{height} is too big, the maximum is {MAX_SIZE}x{MAX_SIZE}"),
        ));
    }
    if width == 0 || height == 0 {
        return Err(DocError::new(
            "bad_size",
            format!("{width}x{height} is not valid, each side must be at least 1"),
        ));
    }
    Ok(())
}
