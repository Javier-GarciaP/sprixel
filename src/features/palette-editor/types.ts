// A color of a palette: one printable character, a name and a #RRGGBB value.
export type PaletteColor = { key: string; hex: string };

// A color as the editor reads it. `used` is true when a sprite has pixels
// with this key.
export type PaletteEntry = PaletteColor & { name: string; used: boolean };

export type PaletteInfo = { name: string; colors: PaletteEntry[] };

// One color of the final list sent on Done. `from` is the key the color had
// when the editor opened, or null for a color added in the editor.
export type ColorEdit = {
  from: string | null;
  key: string;
  name: string;
  hex: string;
};

// What the pixels of a removed color become: the key of a color that stays,
// or "." for transparent.
export type Replacement = { key: string; with: string };

// The codes of the errors the core sends for palettes.
export type PaletteFailure = { code: string; message?: string };

// Everything the palette editor asks of the Rust core. The real calls are in
// `api.ts`; the tests use a fake with the same shape.
export interface PaletteApi {
  loadPalette(name: string): Promise<PaletteInfo>;
  // The whole edit session as one undoable action.
  applyEdit(
    name: string,
    colors: ColorEdit[],
    replacements: Replacement[],
  ): Promise<void>;
}
