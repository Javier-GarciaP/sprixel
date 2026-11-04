//! The power tools of the editor: what an agent does with `transform_region`,
//! `copy_region`, `auto_outline`, `auto_shade`, `analyze_sprite`, `check_tile`,
//! the palette tools and the game data tools, for a person. Each function
//! calls the same pure function as the MCP tool, so the result is the same,
//! and applies it as the user: one history entry. The functions need no
//! window, so the tests call them directly; the commands only add the event.

use super::browser::{CHANGED_EVENT, ChangedEvent, TargetInfo};
use super::editor::{EditorError, EditorState, HistoryState, SpriteArg, history_of};
use super::project::{self, ProjectState};
use crate::analysis::{self, Connectivity, Light, OutlineOptions, Region, ShadeOptions, View};
use crate::document::{Anchor, ColorEdit, Operation, PixelChange, Sprite};
use crate::mcp::tools::gamedata::flat_keys;
use crate::mcp::tools::palettes::{RampArgs, add_ramp};
use crate::mcp::tools::region::{self, Direction, Rect as Box4, changes, cut};
use serde::{Deserialize, Serialize};

const USER: &str = crate::history::USER;

type Failure = EditorError;

/// A rectangle of a frame, in pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl Rect {
    fn tuple(self) -> Box4 {
        (self.x, self.y, self.width, self.height)
    }
}

/// What a write answers: how many pixels changed, and the history flags.
#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Applied {
    pub pixels_changed: usize,
    #[serde(flatten)]
    pub history: HistoryState,
}

/// A change of a region of one frame.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RegionAction {
    FlipHorizontal,
    FlipVertical,
    Rotate { degrees: u32 },
    Move { dx: i64, dy: i64, wrap: bool },
    /// `left_to_right`, `right_to_left`, `top_to_bottom` or `bottom_to_top`.
    Mirror { direction: String },
}

fn fail(code: &str, message: impl AsRef<str>) -> Failure {
    EditorError::new(code, message.as_ref())
}

fn sprite_of<'a>(session: &'a crate::storage::Session, target: &SpriteArg) -> Result<&'a Sprite, Failure> {
    session.project.find_sprite(&target.to_ref()).ok_or_else(|| super::editor::not_found(target))
}

fn check_frame(sprite: &Sprite, frame: u32) -> Result<(), Failure> {
    if frame == 0 || frame as usize > sprite.frame_count() {
        return Err(fail("bad_frame", format!("frame {frame} does not exist")));
    }
    Ok(())
}

/// The region, or the whole frame; it must be inside the sprite.
fn inside(sprite: &Sprite, region: Option<Rect>) -> Result<Rect, Failure> {
    let whole = Rect { x: 0, y: 0, width: sprite.width, height: sprite.height };
    let rect = region.unwrap_or(whole);
    let fits = rect.width > 0
        && rect.height > 0
        && u64::from(rect.x) + u64::from(rect.width) <= u64::from(sprite.width)
        && u64::from(rect.y) + u64::from(rect.height) <= u64::from(sprite.height);
    if !fits {
        return Err(fail("region_out", format!("the region is not inside the {}x{} sprite", sprite.width, sprite.height)));
    }
    Ok(rect)
}

/// Runs `build` on a frame and applies the pixels it gives as one entry by
/// the user. Nothing is applied (and no history entry made) when no pixel
/// changes. A frame an agent is writing to is refused with `frame_busy`.
fn write_pixels(
    state: &ProjectState,
    agents: &EditorState,
    target: &SpriteArg,
    frame: u32,
    build: impl FnOnce(&crate::storage::Session, &Sprite) -> Result<Vec<PixelChange>, Failure>,
) -> Result<(Applied, ChangedEvent), Failure> {
    if agents.blocks(target, frame) {
        return Err(fail("frame_busy", "an agent is drawing on this frame"));
    }
    project::edit(state, |session| {
        let sprite = sprite_of(session, target)?;
        check_frame(sprite, frame)?;
        let pixels = build(session, sprite)?;
        let count = pixels.len();
        let touched = if pixels.is_empty() {
            Vec::new()
        } else {
            session.apply(USER, vec![Operation::SetPixels { sprite: target.to_ref(), layer: 0, frame, pixels }])?
        };
        let event = ChangedEvent {
            revision: session.project.revision,
            touched: touched.iter().map(TargetInfo::from).collect(),
        };
        Ok((Applied { pixels_changed: count, history: history_of(session) }, event))
    })
    .ok_or_else(|| fail("no_project", "no project is open"))?
}

/// Flip, rotate, move or mirror a region of a frame (the whole frame if none).
pub fn region_apply(
    state: &ProjectState,
    agents: &EditorState,
    target: &SpriteArg,
    frame: u32,
    region: Option<Rect>,
    action: &RegionAction,
) -> Result<(Applied, ChangedEvent), Failure> {
    write_pixels(state, agents, target, frame, |_, sprite| {
        let rect = inside(sprite, region)?;
        let (w, h) = (rect.width as usize, rect.height as usize);
        let current = &sprite.layers[0].frames[frame as usize - 1].pixels;
        let old = cut(current, sprite.width, rect.tuple());
        let new = match action {
            RegionAction::FlipHorizontal => region::flip_horizontal(&old, w, h),
            RegionAction::FlipVertical => region::flip_vertical(&old, w, h),
            RegionAction::Move { dx, dy, wrap } => region::shift(&old, w, h, *dx, *dy, *wrap),
            RegionAction::Mirror { direction } => {
                let direction = Direction::parse(direction)
                    .ok_or_else(|| fail("bad_argument", format!("{direction:?} is not a mirror direction")))?;
                region::mirror(&old, w, h, direction)
            }
            RegionAction::Rotate { degrees } => region::rotate(&old, w, h, *degrees).ok_or_else(|| {
                fail("not_square", format!("cannot rotate a {w}x{h} region by {degrees}: 90 and 270 need a square"))
            })?,
        };
        Ok(changes(current, sprite.width, rect.tuple(), &new))
    })
}

/// A copied region: the keys row by row, and the palette they belong to.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Clip {
    pub palette: String,
    pub width: u32,
    pub height: u32,
    pub keys: String,
}

/// Reads a region (the whole frame if none) to paste somewhere else.
pub fn region_copy(state: &ProjectState, target: &SpriteArg, frame: u32, region: Option<Rect>) -> Result<Clip, Failure> {
    project::read(state, |session| {
        let sprite = sprite_of(session, target)?;
        check_frame(sprite, frame)?;
        let rect = inside(sprite, region)?;
        let keys = cut(&sprite.layers[0].frames[frame as usize - 1].pixels, sprite.width, rect.tuple());
        Ok(Clip {
            palette: sprite.palette.clone(),
            width: rect.width,
            height: rect.height,
            keys: String::from_utf8_lossy(&keys).into_owned(),
        })
    })
    .ok_or_else(|| fail("no_project", "no project is open"))?
}

/// Pastes a clip with its top left pixel at `x`, `y`. The sprite must use the
/// palette of the clip (`palette_mismatch`) and the clip must fit (`region_out`).
pub fn region_paste(
    state: &ProjectState,
    agents: &EditorState,
    target: &SpriteArg,
    frame: u32,
    clip: &Clip,
    x: u32,
    y: u32,
    skip_transparent: bool,
) -> Result<(Applied, ChangedEvent), Failure> {
    write_pixels(state, agents, target, frame, |_, sprite| {
        if !crate::document::same_name(&clip.palette, &sprite.palette) {
            return Err(fail(
                "palette_mismatch",
                format!("the copy uses the palette {:?} and {:?} uses {:?}", clip.palette, sprite.name, sprite.palette),
            ));
        }
        let rect = Rect { x, y, width: clip.width, height: clip.height };
        let source = clip.keys.as_bytes();
        if source.len() != (clip.width * clip.height) as usize {
            return Err(fail("bad_argument", "the copy has the wrong number of pixels"));
        }
        inside(sprite, Some(rect))?;
        let current = &sprite.layers[0].frames[frame as usize - 1].pixels;
        let old = cut(current, sprite.width, rect.tuple());
        Ok(changes(current, sprite.width, rect.tuple(), &region::paste(&old, source, skip_transparent)))
    })
}

/// The options of the Outline dialog (the same as `auto_outline`).
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutlineArgs {
    pub key: String,
    /// 4 or 8.
    pub connectivity: u32,
    pub outside_only: bool,
    pub inner: bool,
}

/// The options of the Shade dialog (the same as `auto_shade`).
#[derive(Debug, Clone, Deserialize)]
pub struct ShadeArgs {
    pub light: String,
    pub base: String,
    /// The ramp from dark to light; it includes `base`.
    pub keys: Vec<String>,
    pub region: Option<Rect>,
}

/// The pixels a preview would change, as `[x, y, key]`. Nothing is written.
#[derive(Debug, Serialize, PartialEq)]
pub struct Preview {
    pub pixels: Vec<(u32, u32, String)>,
}

fn preview_of(changes: &[PixelChange]) -> Preview {
    Preview { pixels: changes.iter().map(|c| (c.x, c.y, c.key.to_string())).collect() }
}

fn palette_keys(session: &crate::storage::Session, sprite: &Sprite) -> Vec<char> {
    session.project.palette(&sprite.palette).map(|p| p.colors.iter().map(|c| c.key).collect()).unwrap_or_default()
}

fn view_of(sprite: &Sprite, frame: u32) -> View<'_> {
    View::new(&sprite.layers[0].frames[frame as usize - 1].pixels, sprite.width as usize, sprite.height as usize)
}

/// One color key of the palette (not transparent).
fn color_key(text: &str, palette: &[char], what: &str) -> Result<char, Failure> {
    let mut chars = text.chars();
    match (chars.next(), chars.next()) {
        (Some(key), None) if palette.contains(&key) => Ok(key),
        _ => Err(fail("unknown_key", format!("{what} {text:?} is not a color of the palette"))),
    }
}

fn outline_changes(session: &crate::storage::Session, sprite: &Sprite, frame: u32, args: &OutlineArgs) -> Result<Vec<PixelChange>, Failure> {
    let key = color_key(&args.key, &palette_keys(session, sprite), "the outline key")?;
    let connectivity = match args.connectivity {
        4 => Connectivity::Four,
        8 => Connectivity::Eight,
        _ => return Err(fail("bad_argument", "connectivity must be 4 or 8")),
    };
    let options = OutlineOptions { key, connectivity, outside_only: args.outside_only, inner: args.inner };
    Ok(analysis::auto_outline(view_of(sprite, frame), &options).changes)
}

fn shade_changes(session: &crate::storage::Session, sprite: &Sprite, frame: u32, args: &ShadeArgs) -> Result<Vec<PixelChange>, Failure> {
    let palette = palette_keys(session, sprite);
    let light = Light::parse(&args.light).ok_or_else(|| fail("bad_argument", format!("{:?} is not a light direction", args.light)))?;
    let base = color_key(&args.base, &palette, "the base key")?;
    let keys = args.keys.iter().map(|k| color_key(k, &palette, "the ramp key")).collect::<Result<Vec<char>, _>>()?;
    let mut sorted = keys.clone();
    sorted.sort();
    sorted.dedup();
    if !(2..=8).contains(&keys.len()) || sorted.len() != keys.len() || !keys.contains(&base) {
        return Err(fail("bad_argument", "the ramp needs 2 to 8 different keys and must include the base"));
    }
    let region = args.region.map(|r| inside(sprite, Some(r)).map(|r| Region { x: r.x, y: r.y, width: r.width, height: r.height })).transpose()?;
    Ok(analysis::auto_shade(view_of(sprite, frame), &ShadeOptions { light, base, keys: &keys, region }).changes)
}

/// Reads a frame and gives the pixels a preview or an apply would change.
fn look(
    state: &ProjectState,
    target: &SpriteArg,
    frame: u32,
    find: impl FnOnce(&crate::storage::Session, &Sprite) -> Result<Vec<PixelChange>, Failure>,
) -> Result<Preview, Failure> {
    project::read(state, |session| {
        let sprite = sprite_of(session, target)?;
        check_frame(sprite, frame)?;
        Ok(preview_of(&find(session, sprite)?))
    })
    .ok_or_else(|| fail("no_project", "no project is open"))?
}

pub fn outline_preview(state: &ProjectState, target: &SpriteArg, frame: u32, args: &OutlineArgs) -> Result<Preview, Failure> {
    look(state, target, frame, |session, sprite| outline_changes(session, sprite, frame, args))
}

pub fn outline_apply(state: &ProjectState, agents: &EditorState, target: &SpriteArg, frame: u32, args: &OutlineArgs) -> Result<(Applied, ChangedEvent), Failure> {
    write_pixels(state, agents, target, frame, |session, sprite| outline_changes(session, sprite, frame, args))
}

pub fn shade_preview(state: &ProjectState, target: &SpriteArg, frame: u32, args: &ShadeArgs) -> Result<Preview, Failure> {
    look(state, target, frame, |session, sprite| shade_changes(session, sprite, frame, args))
}

pub fn shade_apply(state: &ProjectState, agents: &EditorState, target: &SpriteArg, frame: u32, args: &ShadeArgs) -> Result<(Applied, ChangedEvent), Failure> {
    write_pixels(state, agents, target, frame, |session, sprite| shade_changes(session, sprite, frame, args))
}

/// What is wrong in one frame.
#[derive(Debug, Serialize, PartialEq)]
pub struct FrameReview {
    pub frame: u32,
    pub loose: Vec<(u32, u32)>,
    /// Empty unless an outline key was given.
    pub gaps: Vec<(u32, u32)>,
}

/// How a frame differs from the next one.
#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ChangeReview {
    pub from: u32,
    pub to: u32,
    pub changed_pixels: usize,
    pub bbox: Option<Rect>,
    pub ground_kept: bool,
}

/// The same checks as `analyze_sprite`, without the limit of 50 per list.
#[derive(Debug, Serialize, PartialEq)]
pub struct Review {
    pub used: Vec<String>,
    pub unused: Vec<String>,
    pub frames: Vec<FrameReview>,
    pub changes: Vec<ChangeReview>,
}

pub fn review(state: &ProjectState, target: &SpriteArg, outline_key: Option<&str>, ground_rows: usize) -> Result<Review, Failure> {
    project::read(state, |session| {
        let sprite = sprite_of(session, target)?;
        let palette = palette_keys(session, sprite);
        let outline = outline_key.map(|k| color_key(k, &palette, "the outline key")).transpose()?;
        let count = sprite.frame_count() as u32;
        let views: Vec<View> = (1..=count).map(|n| view_of(sprite, n)).collect();
        let (used, unused) = analysis::colors(&views, &palette);
        let text = |keys: Vec<char>| keys.iter().map(char::to_string).collect::<Vec<_>>();
        let frames = views
            .iter()
            .enumerate()
            .map(|(i, view)| FrameReview {
                frame: i as u32 + 1,
                loose: analysis::loose_pixels(*view),
                gaps: outline.map(|key| analysis::outline_gaps(*view, key)).unwrap_or_default(),
            })
            .collect();
        let changes = views
            .windows(2)
            .enumerate()
            .map(|(i, pair)| {
                let change = analysis::compare(pair[0], pair[1], ground_rows.max(1));
                ChangeReview {
                    from: i as u32 + 1,
                    to: i as u32 + 2,
                    changed_pixels: change.changed_pixels,
                    bbox: change.bbox.map(|b| Rect { x: b.x, y: b.y, width: b.width, height: b.height }),
                    ground_kept: change.ground_kept,
                }
            })
            .collect();
        Ok(Review { used: text(used), unused: text(unused), frames, changes })
    })
    .ok_or_else(|| fail("no_project", "no project is open"))?
}

/// A pair of pixels that should be equal when the tile repeats.
#[derive(Debug, Serialize, PartialEq)]
pub struct Mismatch {
    /// `left_right` or `top_bottom`.
    pub edge: String,
    pub a: (u32, u32),
    pub b: (u32, u32),
}

/// The edges of a tile: the first and the last column, the first and the
/// last row (the same rule as `check_tile`).
#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TileCheck {
    pub seamless: bool,
    pub mismatches: Vec<Mismatch>,
}

pub fn tile_check(state: &ProjectState, target: &SpriteArg, frame: u32) -> Result<TileCheck, Failure> {
    project::read(state, |session| {
        let sprite = sprite_of(session, target)?;
        check_frame(sprite, frame)?;
        let (w, h) = (sprite.width as usize, sprite.height as usize);
        let flat = flat_keys(sprite, frame as usize - 1);
        let mut found = Vec::new();
        for y in 0..h {
            if flat[y * w] != flat[y * w + w - 1] {
                found.push(Mismatch { edge: "left_right".into(), a: (0, y as u32), b: (w as u32 - 1, y as u32) });
            }
        }
        for x in 0..w {
            if flat[x] != flat[(h - 1) * w + x] {
                found.push(Mismatch { edge: "top_bottom".into(), a: (x as u32, 0), b: (x as u32, h as u32 - 1) });
            }
        }
        Ok(TileCheck { seamless: found.is_empty(), mismatches: found })
    })
    .ok_or_else(|| fail("no_project", "no project is open"))?
}

/// The tones a ramp would add, as `#RRGGBB`, dark to light (nothing is written).
pub fn ramp_preview(base: &str, steps: usize, hue_shift: f64) -> Result<Vec<String>, Failure> {
    crate::document::ramp(&base.to_ascii_uppercase(), steps, hue_shift).map_err(Failure::from)
}

/// The keys of the tones that Generate ramp added or reused.
#[derive(Debug, Serialize, PartialEq)]
pub struct RampAdded {
    pub keys: Vec<String>,
}

/// Adds the tones of a ramp to a palette, as `generate_ramp` does: a tone that
/// the palette already has reuses its key. One history entry.
pub fn ramp_apply(
    state: &ProjectState,
    palette: &str,
    base: &str,
    steps: usize,
    hue_shift: f64,
) -> Result<(RampAdded, ChangedEvent), Failure> {
    project::edit(state, |session| {
        let old = session
            .project
            .palette(palette)
            .ok_or_else(|| fail("not_found", format!("palette {palette:?} does not exist")))?;
        let mut grown = old.clone();
        let ramp = RampArgs { base: base.to_ascii_uppercase(), steps, hue_shift, name: None };
        let tones = add_ramp(&mut grown, &ramp).map_err(|e| fail(&e.code, &e.message))?;
        let edits: Vec<ColorEdit> = grown
            .colors
            .iter()
            .map(|c| ColorEdit { from: old.colors.iter().any(|o| o.key == c.key).then_some(c.key), color: c.clone() })
            .collect();
        let ops = session.project.palette_edit_operations(&old.name, &edits, &[])?;
        let touched = session.apply(USER, ops)?;
        let event = ChangedEvent { revision: session.project.revision, touched: touched.iter().map(TargetInfo::from).collect() };
        Ok((RampAdded { keys: tones.iter().map(|c| c.key.to_string()).collect() }, event))
    })
    .ok_or_else(|| fail("no_project", "no project is open"))?
}

/// Removes the colors of a palette that no sprite uses, as
/// `remove_unused_colors` does. Returns the removed keys. One history entry.
/// (The dialog lists them first with the `used` flags of `palette_info`.)
pub fn unused_remove(state: &ProjectState, palette: &str) -> Result<(Vec<String>, ChangedEvent), Failure> {
    project::edit(state, |session| {
        let found = session
            .project
            .palette(palette)
            .ok_or_else(|| fail("not_found", format!("palette {palette:?} does not exist")))?;
        let used = session.project.used_keys(&found.name);
        let (keep, drop): (Vec<_>, Vec<_>) = found.colors.iter().partition(|c| used.contains(&c.key));
        let edits: Vec<ColorEdit> = keep.iter().map(|c| ColorEdit { from: Some(c.key), color: (*c).clone() }).collect();
        let ops = session.project.palette_edit_operations(&found.name, &edits, &[])?;
        let removed = drop.iter().map(|c| c.key.to_string()).collect();
        let touched = if drop.is_empty() { Vec::new() } else { session.apply(USER, ops)? };
        let event = ChangedEvent { revision: session.project.revision, touched: touched.iter().map(TargetInfo::from).collect() };
        Ok((removed, event))
    })
    .ok_or_else(|| fail("no_project", "no project is open"))?
}

/// Gives one frame its own duration in milliseconds (10 to 10000).
pub fn duration_set(state: &ProjectState, target: &SpriteArg, frame: u32, ms: u32) -> Result<ChangedEvent, Failure> {
    if !(10..=10000).contains(&ms) {
        return Err(fail("bad_argument", "the duration must be from 10 to 10000 milliseconds"));
    }
    one_entry(state, target, |sprite| {
        check_frame(sprite, frame)?;
        Ok(vec![Operation::SetFrameDuration { sprite: target.to_ref(), frame, duration_ms: ms }])
    })
}

/// A named point of a sprite, or of one frame of it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnchorArg {
    pub name: String,
    pub x: u32,
    pub y: u32,
    pub frame: Option<u32>,
}

/// Which anchor a save replaces (a rename or a move of an existing anchor).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnchorKey {
    pub name: String,
    pub frame: Option<u32>,
}

pub fn anchors_of(state: &ProjectState, target: &SpriteArg) -> Result<Vec<AnchorArg>, Failure> {
    project::read(state, |session| {
        let sprite = sprite_of(session, target)?;
        Ok(sprite.anchors.iter().map(|a| AnchorArg { name: a.name.clone(), x: a.x, y: a.y, frame: a.frame }).collect())
    })
    .ok_or_else(|| fail("no_project", "no project is open"))?
}

/// Creates an anchor, or with `replaces` renames or moves that one: the old
/// one is removed and the new one set, in ONE history entry.
pub fn anchor_save(state: &ProjectState, target: &SpriteArg, anchor: &AnchorArg, replaces: Option<&AnchorKey>) -> Result<ChangedEvent, Failure> {
    one_entry(state, target, |sprite| {
        let name = anchor.name.trim().to_string();
        let new = Anchor { name, x: anchor.x, y: anchor.y, frame: anchor.frame };
        new.validate(sprite.width, sprite.height, sprite.frame_count()).map_err(Failure::from)?;
        let mut ops = Vec::new();
        if let Some(old) = replaces {
            if !old.name.eq_ignore_ascii_case(&new.name) || old.frame != new.frame {
                ops.push(Operation::RemoveAnchor { sprite: target.to_ref(), name: old.name.clone(), frame: old.frame });
            }
        }
        ops.push(Operation::SetAnchor { sprite: target.to_ref(), anchor: new, index: None });
        Ok(ops)
    })
}

pub fn anchor_delete(state: &ProjectState, target: &SpriteArg, key: &AnchorKey) -> Result<ChangedEvent, Failure> {
    one_entry(state, target, |_| {
        Ok(vec![Operation::RemoveAnchor { sprite: target.to_ref(), name: key.name.clone(), frame: key.frame }])
    })
}

/// Applies the operations `make` gives as one history entry by the user.
fn one_entry(
    state: &ProjectState,
    target: &SpriteArg,
    make: impl FnOnce(&Sprite) -> Result<Vec<Operation>, Failure>,
) -> Result<ChangedEvent, Failure> {
    project::edit(state, |session| {
        let ops = make(sprite_of(session, target)?)?;
        let touched = session.apply(USER, ops)?;
        Ok(ChangedEvent { revision: session.project.revision, touched: touched.iter().map(TargetInfo::from).collect() })
    })
    .ok_or_else(|| fail("no_project", "no project is open"))?
}

fn announce(app: &tauri::AppHandle, event: ChangedEvent) {
    let _ = tauri::Emitter::emit(app, CHANGED_EVENT, event);
}

#[tauri::command]
pub fn power_region(
    app: tauri::AppHandle,
    state: tauri::State<ProjectState>,
    agents: tauri::State<EditorState>,
    sprite: SpriteArg,
    frame: u32,
    region: Option<Rect>,
    action: RegionAction,
) -> Result<Applied, Failure> {
    let (applied, event) = region_apply(&state, &agents, &sprite, frame, region, &action)?;
    announce(&app, event);
    Ok(applied)
}

#[tauri::command]
pub fn power_copy(
    state: tauri::State<ProjectState>,
    sprite: SpriteArg,
    frame: u32,
    region: Option<Rect>,
) -> Result<Clip, Failure> {
    region_copy(&state, &sprite, frame, region)
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub fn power_paste(
    app: tauri::AppHandle,
    state: tauri::State<ProjectState>,
    agents: tauri::State<EditorState>,
    sprite: SpriteArg,
    frame: u32,
    clip: Clip,
    x: u32,
    y: u32,
    skip_transparent: bool,
) -> Result<Applied, Failure> {
    let (applied, event) = region_paste(&state, &agents, &sprite, frame, &clip, x, y, skip_transparent)?;
    announce(&app, event);
    Ok(applied)
}

#[tauri::command]
pub fn power_outline_preview(state: tauri::State<ProjectState>, sprite: SpriteArg, frame: u32, args: OutlineArgs) -> Result<Preview, Failure> {
    outline_preview(&state, &sprite, frame, &args)
}

#[tauri::command]
pub fn power_outline_apply(
    app: tauri::AppHandle,
    state: tauri::State<ProjectState>,
    agents: tauri::State<EditorState>,
    sprite: SpriteArg,
    frame: u32,
    args: OutlineArgs,
) -> Result<Applied, Failure> {
    let (applied, event) = outline_apply(&state, &agents, &sprite, frame, &args)?;
    announce(&app, event);
    Ok(applied)
}

#[tauri::command]
pub fn power_shade_preview(state: tauri::State<ProjectState>, sprite: SpriteArg, frame: u32, args: ShadeArgs) -> Result<Preview, Failure> {
    shade_preview(&state, &sprite, frame, &args)
}

#[tauri::command]
pub fn power_shade_apply(
    app: tauri::AppHandle,
    state: tauri::State<ProjectState>,
    agents: tauri::State<EditorState>,
    sprite: SpriteArg,
    frame: u32,
    args: ShadeArgs,
) -> Result<Applied, Failure> {
    let (applied, event) = shade_apply(&state, &agents, &sprite, frame, &args)?;
    announce(&app, event);
    Ok(applied)
}

#[tauri::command]
pub fn power_review(state: tauri::State<ProjectState>, sprite: SpriteArg, outline_key: Option<String>) -> Result<Review, Failure> {
    review(&state, &sprite, outline_key.as_deref(), 2)
}

#[tauri::command]
pub fn power_tile_check(state: tauri::State<ProjectState>, sprite: SpriteArg, frame: u32) -> Result<TileCheck, Failure> {
    tile_check(&state, &sprite, frame)
}

#[tauri::command]
pub fn power_ramp_preview(base: String, steps: usize, hue_shift: f64) -> Result<Vec<String>, Failure> {
    ramp_preview(&base, steps, hue_shift)
}

#[tauri::command]
pub fn power_ramp_apply(
    app: tauri::AppHandle,
    state: tauri::State<ProjectState>,
    palette: String,
    base: String,
    steps: usize,
    hue_shift: f64,
) -> Result<RampAdded, Failure> {
    let (added, event) = ramp_apply(&state, &palette, &base, steps, hue_shift)?;
    announce(&app, event);
    Ok(added)
}

#[tauri::command]
pub fn power_unused_remove(app: tauri::AppHandle, state: tauri::State<ProjectState>, palette: String) -> Result<Vec<String>, Failure> {
    let (removed, event) = unused_remove(&state, &palette)?;
    announce(&app, event);
    Ok(removed)
}

#[tauri::command]
pub fn power_duration_set(app: tauri::AppHandle, state: tauri::State<ProjectState>, sprite: SpriteArg, frame: u32, ms: u32) -> Result<(), Failure> {
    announce(&app, duration_set(&state, &sprite, frame, ms)?);
    Ok(())
}

#[tauri::command]
pub fn power_anchors(state: tauri::State<ProjectState>, sprite: SpriteArg) -> Result<Vec<AnchorArg>, Failure> {
    anchors_of(&state, &sprite)
}

#[tauri::command]
pub fn power_anchor_save(
    app: tauri::AppHandle,
    state: tauri::State<ProjectState>,
    sprite: SpriteArg,
    anchor: AnchorArg,
    replaces: Option<AnchorKey>,
) -> Result<(), Failure> {
    announce(&app, anchor_save(&state, &sprite, &anchor, replaces.as_ref())?);
    Ok(())
}

#[tauri::command]
pub fn power_anchor_delete(app: tauri::AppHandle, state: tauri::State<ProjectState>, sprite: SpriteArg, key: AnchorKey) -> Result<(), Failure> {
    announce(&app, anchor_delete(&state, &sprite, &key)?);
    Ok(())
}

#[cfg(test)]
#[path = "power_tests.rs"]
mod tests;
