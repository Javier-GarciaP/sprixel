//! `auto_shade`: lit and shadow tones on the pixels of one key, by a fixed rule
//! (see "auto_shade" in the design of the change `agent-analysis-tools`).

use super::View;
use crate::document::PixelChange;

/// Where the light comes from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Light {
    TopLeft,
    Top,
    TopRight,
    Left,
    Right,
    BottomLeft,
    Bottom,
    BottomRight,
}

impl Light {
    /// The names a tool accepts.
    pub const NAMES: [&'static str; 8] =
        ["top-left", "top", "top-right", "left", "right", "bottom-left", "bottom", "bottom-right"];

    pub fn parse(name: &str) -> Option<Light> {
        let all = [
            Light::TopLeft, Light::Top, Light::TopRight, Light::Left,
            Light::Right, Light::BottomLeft, Light::Bottom, Light::BottomRight,
        ];
        Light::NAMES.iter().position(|n| *n == name).map(|i| all[i])
    }

    /// One step toward the light.
    fn step(self) -> (i64, i64) {
        match self {
            Light::TopLeft => (-1, -1),
            Light::Top => (0, -1),
            Light::TopRight => (1, -1),
            Light::Left => (-1, 0),
            Light::Right => (1, 0),
            Light::BottomLeft => (-1, 1),
            Light::Bottom => (0, 1),
            Light::BottomRight => (1, 1),
        }
    }
}

/// A box of pixels. Only pixels inside it may change.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Region {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl Region {
    fn contains(&self, x: i64, y: i64) -> bool {
        x >= self.x as i64 && y >= self.y as i64 && x < (self.x + self.width) as i64 && y < (self.y + self.height) as i64
    }
}

pub struct ShadeOptions<'a> {
    pub light: Light,
    /// The key of the pixels to shade.
    pub base: char,
    /// The ramp, from dark to light. It has `base`.
    pub keys: &'a [char],
    pub region: Option<Region>,
}

pub struct Shade {
    /// The pixels to set, in reading order. Only pixels that really change.
    pub changes: Vec<PixelChange>,
    /// How many pixels became lighter and how many darker.
    pub lit: usize,
    pub shadow: usize,
}

/// How many steps from (x, y) in the direction (dx, dy) stay on `base` pixels,
/// counting at most `cap`.
fn run(view: View, x: i64, y: i64, (dx, dy): (i64, i64), base: u8, cap: usize) -> usize {
    (1..=cap as i64).take_while(|k| view.get(x + k * dx, y + k * dy) == Some(base)).count()
}

pub fn auto_shade(view: View, options: &ShadeOptions) -> Shade {
    let none = Shade { changes: Vec::new(), lit: 0, shadow: 0 };
    let Some(base_index) = options.keys.iter().position(|k| *k == options.base) else { return none };
    let n = options.keys.len();
    let (darker, lighter) = (base_index, n - 1 - base_index);
    let (toward, away) = (options.light.step(), (-options.light.step().0, -options.light.step().1));
    let base = options.base as u8;
    let mut out = none;
    for y in 0..view.height as i64 {
        for x in 0..view.width as i64 {
            if view.get(x, y) != Some(base) || options.region.is_some_and(|r| !r.contains(x, y)) {
                continue;
            }
            let lit = run(view, x, y, toward, base, lighter);
            let dark = run(view, x, y, away, base, darker);
            let (lit, dark) = ((lit < lighter).then_some(lit), (dark < darker).then_some(dark));
            let key = match (lit, dark) {
                (Some(a), Some(b)) if a == b => continue,
                (Some(a), Some(b)) if a < b => options.keys[n - 1 - a],
                (Some(a), None) => options.keys[n - 1 - a],
                (_, Some(b)) => options.keys[b],
                (None, None) => continue,
            };
            if key != options.base {
                if key_is_lighter(options.keys, key, base_index) { out.lit += 1 } else { out.shadow += 1 }
                out.changes.push(PixelChange { x: x as u32, y: y as u32, key });
            }
        }
    }
    out
}

/// True when `key` is after the base in the ramp.
fn key_is_lighter(keys: &[char], key: char, base_index: usize) -> bool {
    keys.iter().position(|k| *k == key).is_some_and(|i| i > base_index)
}
