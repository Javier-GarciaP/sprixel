// Turns a frame (rows of palette keys) into pixels. These functions are pure,
// so tests can run them without a canvas (jsdom has none).

import type { FrameData } from "@/shared/lib/core";

// `#RRGGBB` to [r, g, b]. Anything else gives null.
export function parseHex(hex: string): [number, number, number] | null {
  const match = /^#([0-9a-f]{2})([0-9a-f]{2})([0-9a-f]{2})$/i.exec(hex);
  if (!match) return null;
  return [
    parseInt(match[1], 16),
    parseInt(match[2], 16),
    parseInt(match[3], 16),
  ];
}

// One RGBA pixel for each pixel of the frame. `.` and keys that are not in
// the palette stay transparent.
export function frameToRgba(frame: FrameData): Uint8ClampedArray<ArrayBuffer> {
  const colors = new Map<string, [number, number, number]>();
  for (const { key, hex } of frame.colors) {
    const rgb = parseHex(hex);
    if (rgb) colors.set(key, rgb);
  }
  const out = new Uint8ClampedArray(frame.width * frame.height * 4);
  frame.rows.forEach((row, y) => {
    for (let x = 0; x < frame.width; x++) {
      const rgb = colors.get(row[x]);
      if (!rgb) continue;
      const at = (y * frame.width + x) * 4;
      out[at] = rgb[0];
      out[at + 1] = rgb[1];
      out[at + 2] = rgb[2];
      out[at + 3] = 255;
    }
  });
  return out;
}

// The biggest whole number that fits the sprite in `box` pixels, at least 1.
// Whole numbers keep the pixels sharp.
export function integerScale(
  width: number,
  height: number,
  box: number,
): number {
  return Math.max(1, Math.floor(box / Math.max(width, height, 1)));
}

// Repeats every pixel `scale` times in both directions.
export function scaleRgba(
  rgba: Uint8ClampedArray<ArrayBuffer>,
  width: number,
  height: number,
  scale: number,
): Uint8ClampedArray<ArrayBuffer> {
  if (scale === 1) return rgba;
  const out = new Uint8ClampedArray(rgba.length * scale * scale);
  const outWidth = width * scale;
  for (let y = 0; y < height; y++) {
    for (let x = 0; x < width; x++) {
      const from = (y * width + x) * 4;
      for (let dy = 0; dy < scale; dy++) {
        for (let dx = 0; dx < scale; dx++) {
          const to = ((y * scale + dy) * outWidth + x * scale + dx) * 4;
          out.set(rgba.subarray(from, from + 4), to);
        }
      }
    }
  }
  return out;
}
