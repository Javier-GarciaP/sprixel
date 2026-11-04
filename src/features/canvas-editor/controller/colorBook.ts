import { buildColorTable, setColor, type ColorTable } from "../render/colors";
import { TRANSPARENT_CODE, type PaletteColor } from "../types";

// The first codes are not printable characters, so they are never a palette
// key. A color that is not in the palette yet is drawn with one of them until
// the core adds it to the palette and tells the real key.
const FIRST_PENDING = 1;
const LAST_PENDING = 31;

export const normalizeHex = (hex: string) => hex.toUpperCase();

// Maps colors to pixel codes and keeps the color table the canvas paints with.
export class ColorBook {
  readonly table: ColorTable;
  private keys = new Map<string, number>();
  // The color of each code, so reading a pixel is one lookup.
  private hexByCode: (string | null)[] = new Array(128).fill(null);
  private pending = new Map<number, string>();

  constructor(palette: PaletteColor[]) {
    this.table = buildColorTable(palette);
    for (const color of palette) {
      this.learn(normalizeHex(color.hex), color.key.charCodeAt(0));
    }
  }

  // The code to draw `hex` with: its key, or a pending code for a new color.
  codeFor(hex: string): number {
    const wanted = normalizeHex(hex);
    const known = this.keys.get(wanted);
    if (known !== undefined) return known;
    for (let code = FIRST_PENDING; code <= LAST_PENDING; code++) {
      if (!this.pending.has(code)) {
        this.pending.set(code, wanted);
        this.learn(wanted, code);
        setColor(this.table, code, wanted);
        return code;
      }
    }
    throw new Error("too many new colors waiting for the core");
  }

  // The color of a code, or null for a transparent pixel.
  hexOf(code: number): string | null {
    return code === TRANSPARENT_CODE ? null : (this.hexByCode[code] ?? null);
  }

  private learn(hex: string, code: number) {
    this.keys.set(hex, code);
    this.hexByCode[code] = hex;
  }

  // The core answered with the palette. Every pending color now has its real
  // key: returns what to replace in the pixels (pending code to key code).
  settle(palette: PaletteColor[]): Map<number, number> {
    const replace = new Map<number, number>();
    for (const color of palette) {
      const hex = normalizeHex(color.hex);
      const key = color.key.charCodeAt(0);
      const pendingCode = this.keys.get(hex);
      if (pendingCode !== undefined && this.pending.has(pendingCode)) {
        replace.set(pendingCode, key);
        this.pending.delete(pendingCode);
        this.table.fill(0, pendingCode * 4, pendingCode * 4 + 4);
        this.hexByCode[pendingCode] = null;
      }
      this.learn(hex, key);
      setColor(this.table, key, hex);
    }
    return replace;
  }

  get hasPending() {
    return this.pending.size > 0;
  }
}
