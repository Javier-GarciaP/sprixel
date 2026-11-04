//! Color ramps: darker and lighter tones of one base color.

use super::error::DocError;

/// Tones from dark to light. The base color is one of them. With `steps`
/// tones, `(steps - 1) / 2` (rounded down) are lighter and the rest darker.
/// `hue_shift` turns the hue by that many degrees at the ends of the ramp
/// (darker tones go the other way). Saturation stays the same.
pub fn ramp(base: &str, steps: usize, hue_shift: f64) -> Result<Vec<String>, DocError> {
    if !(3..=9).contains(&steps) {
        return Err(DocError::new("bad_argument", format!("steps must be 3 to 9, got {steps}")));
    }
    if !(-60.0..=60.0).contains(&hue_shift) {
        return Err(DocError::new("bad_argument", "hue_shift must be -60 to 60"));
    }
    let (h, s, l) = to_hsl(parse(base)?);
    let lighter = (steps - 1) / 2;
    let darker = steps - 1 - lighter;
    let mut out = Vec::with_capacity(steps);
    for k in (1..=darker).rev() {
        let t = k as f64 / darker as f64;
        out.push(to_hex(h - hue_shift * t, s, l * (1.0 - 0.65 * t)));
    }
    out.push(base.to_uppercase());
    for k in 1..=lighter {
        let t = k as f64 / lighter as f64;
        out.push(to_hex(h + hue_shift * t, s, l + (1.0 - l) * 0.65 * t));
    }
    Ok(out)
}

fn parse(hex: &str) -> Result<[f64; 3], DocError> {
    let bad = || DocError::new("bad_argument", format!("{hex:?} is not a color, expected #RRGGBB"));
    if hex.len() != 7 || !hex.starts_with('#') {
        return Err(bad());
    }
    let byte = |i: usize| u8::from_str_radix(&hex[i..i + 2], 16).map(f64::from).map_err(|_| bad());
    Ok([byte(1)? / 255.0, byte(3)? / 255.0, byte(5)? / 255.0])
}

fn to_hsl([r, g, b]: [f64; 3]) -> (f64, f64, f64) {
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) / 2.0;
    let d = max - min;
    if d == 0.0 {
        return (0.0, 0.0, l);
    }
    let s = d / (1.0 - (2.0 * l - 1.0).abs());
    let h = if max == r {
        ((g - b) / d).rem_euclid(6.0)
    } else if max == g {
        (b - r) / d + 2.0
    } else {
        (r - g) / d + 4.0
    };
    (h * 60.0, s, l)
}

fn to_hex(h: f64, s: f64, l: f64) -> String {
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let h6 = h.rem_euclid(360.0) / 60.0;
    let x = c * (1.0 - (h6.rem_euclid(2.0) - 1.0).abs());
    let (r, g, b) = match h6 as u8 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    let m = l - c / 2.0;
    let channel = |v: f64| ((v + m) * 255.0).round().clamp(0.0, 255.0) as u8;
    format!("#{:02X}{:02X}{:02X}", channel(r), channel(g), channel(b))
}
