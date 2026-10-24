import { TRANSPARENT_CODE, type PaletteColor } from "../types";

// RGBA of every pixel code, 4 bytes each. A code is the character code of a
// palette key (below 128), so a lookup is `table[code * 4]`.
export type ColorTable = Uint8Array;

export function hexToRgb(hex: string): [number, number, number] {
  const value = parseInt(hex.slice(1), 16);
  return [(value >> 16) & 255, (value >> 8) & 255, value & 255];
}

export function setColor(table: ColorTable, code: number, hex: string) {
  const [r, g, b] = hexToRgb(hex);
  table.set([r, g, b, 255], code * 4);
}

// Every code starts transparent. `.` always stays transparent.
export function buildColorTable(colors: PaletteColor[]): ColorTable {
  const table = new Uint8Array(128 * 4);
  for (const color of colors) {
    const code = color.key.charCodeAt(0);
    if (code !== TRANSPARENT_CODE && code < 128)
      setColor(table, code, color.hex);
  }
  return table;
}
