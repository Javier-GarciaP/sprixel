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

// A built-in palette to start from.
export type Preset = { id: string; name: string; colors: PaletteColor[] };

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
  listPresets(): Promise<Preset[]>;
  // Makes a palette, empty or with the colors of a preset. Gives its name.
  createPalette(name: string, preset: string | null): Promise<string>;
  // The native file picker for a `.gpl` or `.hex` file. Null when cancelled.
  pickPaletteFile(): Promise<string | null>;
  // Makes a palette from the colors of a file. Gives its name.
  importPalette(name: string, path: string): Promise<string>;
  // The tones a ramp would add (`#RRGGBB`, dark to light). Writes nothing.
  rampPreview(base: string, steps: number, hueShift: number): Promise<string[]>;
  // Adds a ramp to a palette as one undoable action. Gives the keys of its
  // tones (a tone the palette already has keeps its key).
  rampApply(
    name: string,
    base: string,
    steps: number,
    hueShift: number,
  ): Promise<{ keys: string[] }>;
  // Removes the colors no sprite uses, as one undoable action. Gives the
  // removed keys.
  removeUnused(name: string): Promise<string[]>;
}
