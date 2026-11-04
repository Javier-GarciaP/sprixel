import { TRANSPARENT_CODE, type Rect } from "../types";
import type { ColorTable } from "./colors";

// What is drawn: the pixels of the frame, the previous frame as onion skin,
// the grid option and the selection. The theme is not an input on purpose:
// the checkerboard is the same in the light and the dark theme.
export type Scene = {
  width: number;
  height: number;
  pixels: Uint8Array;
  colors: ColorTable;
  onion: { pixels: Uint8Array; opacity: number } | null;
  grid: boolean;
  selection: Rect | null;
  // The mirror lines of the symmetry mode, in sprite pixels (see
  // `axisLines`): `x` is the vertical line and `y` the horizontal one.
  axis?: { x: number | null; y: number | null };
  // Pixels that a dialog would write (Outline, Shade): shown over the frame
  // in see-through colors, but not written. `code` is a palette key code.
  overlay?: { x: number; y: number; code: number }[];
  // A pixel to point at (the Review panel): a box around it.
  highlight?: { x: number; y: number } | null;
};

// How much of the preview color shows over the pixel under it.
export const OVERLAY_ALPHA = 0.75;

// Where the sprite is on the screen: a whole-number zoom and the screen
// position of its top left corner.
export type View = { zoom: number; x: number; y: number };

// Neutral grays: black and white pixels stay visible on both.
export const CHECKER_LIGHT = [180, 180, 180] as const;
export const CHECKER_DARK = [156, 156, 156] as const;
export const CHECKER_SIZE = 8;
export const ONION_BLUE = [59, 130, 246] as const;
// The grid is only drawn from 600%.
export const GRID_MIN_ZOOM = 6;

// Paints the scene into `buffer`, an RGBA buffer of the viewport
// (`viewWidth` x `viewHeight`). Every sprite pixel becomes a square block of
// `zoom` x `zoom` screen pixels with no smoothing. With `dirty` (a rectangle
// of sprite pixels) only that area is painted; without it the whole viewport
// is painted and the area outside the sprite is cleared to transparent.
export function renderScene(
  buffer: Uint8ClampedArray,
  viewWidth: number,
  viewHeight: number,
  scene: Scene,
  view: View,
  dirty?: Rect,
) {
  const { zoom } = view;
  let first: number;
  let last: number;
  let top: number;
  let bottom: number;
  if (dirty) {
    first = Math.max(0, dirty.x);
    last = Math.min(scene.width - 1, dirty.x + dirty.width - 1);
    top = Math.max(0, dirty.y);
    bottom = Math.min(scene.height - 1, dirty.y + dirty.height - 1);
  } else {
    buffer.fill(0, 0, viewWidth * viewHeight * 4);
    // Only the sprite pixels that are on the screen.
    first = Math.max(0, Math.floor(-view.x / zoom));
    last = Math.min(
      scene.width - 1,
      Math.floor((viewWidth - 1 - view.x) / zoom),
    );
    top = Math.max(0, Math.floor(-view.y / zoom));
    bottom = Math.min(
      scene.height - 1,
      Math.floor((viewHeight - 1 - view.y) / zoom),
    );
  }

  const showGrid = scene.grid && zoom >= GRID_MIN_ZOOM;
  const table = scene.colors;
  for (let py = top; py <= bottom; py++) {
    for (let px = first; px <= last; px++) {
      const at = py * scene.width + px;
      const code = scene.pixels[at];
      const opaque = code !== TRANSPARENT_CODE && table[code * 4 + 3] === 255;
      const onionOn =
        !opaque &&
        scene.onion !== null &&
        scene.onion.pixels[at] !== TRANSPARENT_CODE;
      const x0 = view.x + px * zoom;
      const y0 = view.y + py * zoom;
      for (let ly = 0; ly < zoom; ly++) {
        const sy = y0 + ly;
        if (sy < 0 || sy >= viewHeight) continue;
        for (let lx = 0; lx < zoom; lx++) {
          const sx = x0 + lx;
          if (sx < 0 || sx >= viewWidth) continue;
          let r: number, g: number, b: number;
          if (opaque) {
            r = table[code * 4];
            g = table[code * 4 + 1];
            b = table[code * 4 + 2];
          } else {
            // The checkerboard follows the sprite, so it does not slide
            // under the pixels when the view moves.
            const cell =
              (Math.floor((sx - view.x) / CHECKER_SIZE) +
                Math.floor((sy - view.y) / CHECKER_SIZE)) &
              1;
            [r, g, b] = cell === 0 ? CHECKER_LIGHT : CHECKER_DARK;
            if (onionOn) {
              const a = scene.onion!.opacity;
              r = Math.round(r + (ONION_BLUE[0] - r) * a);
              g = Math.round(g + (ONION_BLUE[1] - g) * a);
              b = Math.round(b + (ONION_BLUE[2] - b) * a);
            }
          }
          if (showGrid && (lx === 0 || ly === 0)) {
            // Darker on light colors, lighter on dark colors.
            const light = r * 0.299 + g * 0.587 + b * 0.114 > 128;
            const target = light ? 0 : 255;
            r = Math.round(r + (target - r) * 0.25);
            g = Math.round(g + (target - g) * 0.25);
            b = Math.round(b + (target - b) * 0.25);
          }
          const o = (sy * viewWidth + sx) * 4;
          buffer[o] = r;
          buffer[o + 1] = g;
          buffer[o + 2] = b;
          buffer[o + 3] = 255;
        }
      }
    }
  }
  if (scene.overlay) {
    drawOverlay(buffer, viewWidth, viewHeight, scene, view, {
      first,
      last,
      top,
      bottom,
    });
  }
  if (scene.selection) {
    drawSelection(buffer, viewWidth, viewHeight, scene.selection, view);
  }
  if (scene.highlight)
    drawHighlight(buffer, viewWidth, viewHeight, scene.highlight, view);
  if (scene.axis) drawAxis(buffer, viewWidth, viewHeight, scene, view);
}

// The preview pixels that are inside the painted area, blended over it.
function drawOverlay(
  buffer: Uint8ClampedArray,
  viewWidth: number,
  viewHeight: number,
  scene: Scene,
  view: View,
  area: { first: number; last: number; top: number; bottom: number },
) {
  const table = scene.colors;
  for (const { x, y, code } of scene.overlay!) {
    if (x < area.first || x > area.last || y < area.top || y > area.bottom) {
      continue;
    }
    const color = [table[code * 4], table[code * 4 + 1], table[code * 4 + 2]];
    for (let ly = 0; ly < view.zoom; ly++) {
      const sy = view.y + y * view.zoom + ly;
      if (sy < 0 || sy >= viewHeight) continue;
      for (let lx = 0; lx < view.zoom; lx++) {
        const sx = view.x + x * view.zoom + lx;
        if (sx < 0 || sx >= viewWidth) continue;
        const o = (sy * viewWidth + sx) * 4;
        for (let c = 0; c < 3; c++) {
          buffer[o + c] = Math.round(
            buffer[o + c] + (color[c] - buffer[o + c]) * OVERLAY_ALPHA,
          );
        }
        buffer[o + 3] = 255;
      }
    }
  }
}

// A black box with a white box around it: seen on any color.
function drawHighlight(
  buffer: Uint8ClampedArray,
  viewWidth: number,
  viewHeight: number,
  pixel: { x: number; y: number },
  view: View,
) {
  const box = (grow: number, value: number) => {
    const left = view.x + pixel.x * view.zoom - grow;
    const top = view.y + pixel.y * view.zoom - grow;
    const right = view.x + (pixel.x + 1) * view.zoom - 1 + grow;
    const bottom = view.y + (pixel.y + 1) * view.zoom - 1 + grow;
    const put = (sx: number, sy: number) => {
      if (sx < 0 || sy < 0 || sx >= viewWidth || sy >= viewHeight) return;
      const o = (sy * viewWidth + sx) * 4;
      buffer[o] = buffer[o + 1] = buffer[o + 2] = value;
      buffer[o + 3] = 255;
    };
    for (let sx = left; sx <= right; sx++) {
      put(sx, top);
      put(sx, bottom);
    }
    for (let sy = top; sy <= bottom; sy++) {
      put(left, sy);
      put(right, sy);
    }
  };
  box(2, 255);
  box(1, 0);
}

// Magenta stands out on the gray checkerboard and on most palettes.
export const AXIS_COLOR = [236, 72, 153] as const;

// A dashed line over the sprite for each mirror line that is on. It is
// painted last, so it is on top of the pixels and of the selection.
function drawAxis(
  buffer: Uint8ClampedArray,
  viewWidth: number,
  viewHeight: number,
  scene: Scene,
  view: View,
) {
  const put = (sx: number, sy: number) => {
    if (sx < 0 || sy < 0 || sx >= viewWidth || sy >= viewHeight) return;
    const o = (sy * viewWidth + sx) * 4;
    [buffer[o], buffer[o + 1], buffer[o + 2]] = AXIS_COLOR;
    buffer[o + 3] = 255;
  };
  const { x, y } = scene.axis!;
  const left = view.x;
  const top = view.y;
  const right = left + scene.width * view.zoom;
  const bottom = top + scene.height * view.zoom;
  if (x !== null) {
    const sx = Math.round(view.x + x * view.zoom);
    for (let sy = top; sy < bottom; sy++) if ((sy >> 2) % 2 === 0) put(sx, sy);
  }
  if (y !== null) {
    const sy = Math.round(view.y + y * view.zoom);
    for (let sx = left; sx < right; sx++) if ((sx >> 2) % 2 === 0) put(sx, sy);
  }
}

// A dashed black and white outline on the screen edge of the selection.
function drawSelection(
  buffer: Uint8ClampedArray,
  viewWidth: number,
  viewHeight: number,
  selection: Rect,
  view: View,
) {
  const left = view.x + selection.x * view.zoom;
  const top = view.y + selection.y * view.zoom;
  const right = left + selection.width * view.zoom - 1;
  const bottom = top + selection.height * view.zoom - 1;
  const put = (sx: number, sy: number) => {
    if (sx < 0 || sy < 0 || sx >= viewWidth || sy >= viewHeight) return;
    const value = ((sx + sy) >> 2) & 1 ? 0 : 255;
    const o = (sy * viewWidth + sx) * 4;
    buffer[o] = buffer[o + 1] = buffer[o + 2] = value;
    buffer[o + 3] = 255;
  };
  for (let sx = left; sx <= right; sx++) {
    put(sx, top);
    put(sx, bottom);
  }
  for (let sy = top; sy <= bottom; sy++) {
    put(left, sy);
    put(right, sy);
  }
}
