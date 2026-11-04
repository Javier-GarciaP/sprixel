import { TRANSPARENT_CODE } from "../types";
import type { ColorTable } from "./colors";

export const TILES = 3;

// The RGBA picture of a tile repeated 3 x 3 times. Transparent pixels stay
// transparent, so the background of the preview shows through them.
export function tilePicture(
  pixels: Uint8Array,
  width: number,
  height: number,
  colors: ColorTable,
): Uint8ClampedArray<ArrayBuffer> {
  const out = new Uint8ClampedArray(
    new ArrayBuffer(width * TILES * height * TILES * 4),
  );
  const rowLength = width * TILES;
  for (let y = 0; y < height * TILES; y++) {
    for (let x = 0; x < rowLength; x++) {
      const code = pixels[(y % height) * width + (x % width)];
      if (code === TRANSPARENT_CODE || colors[code * 4 + 3] !== 255) continue;
      const o = (y * rowLength + x) * 4;
      out.set(colors.subarray(code * 4, code * 4 + 4), o);
    }
  }
  return out;
}
