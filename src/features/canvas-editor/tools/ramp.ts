import type { PaletteColor } from "../types";

// How light a color looks, from 0 (black) to 255 (white).
export function lightness(hex: string): number {
  const value = parseInt(hex.slice(1), 16);
  const r = (value >> 16) & 255;
  const g = (value >> 8) & 255;
  const b = value & 255;
  return r * 0.299 + g * 0.587 + b * 0.114;
}

// The keys sorted from the darkest color to the lightest one, which is the
// order a shading ramp needs.
export function darkToLight(palette: PaletteColor[], keys: string[]): string[] {
  const hex = (key: string) =>
    palette.find((color) => color.key === key)?.hex ?? "#000000";
  return [...keys].sort((a, b) => lightness(hex(a)) - lightness(hex(b)));
}

// The darkest color of the palette (the usual outline color), or null.
export function darkestKey(palette: PaletteColor[]): string | null {
  return (
    darkToLight(
      palette,
      palette.map((color) => color.key),
    )[0] ?? null
  );
}
