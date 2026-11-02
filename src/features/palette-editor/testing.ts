import type {
  ColorEdit,
  PaletteApi,
  PaletteEntry,
  PaletteInfo,
  Replacement,
} from "./types";

// An in-memory stand-in for the Rust core, for the tests. It only records what
// the editor sends; the rules of the edit are tested in Rust.
export class FakePaletteApi implements PaletteApi {
  applied: {
    name: string;
    colors: ColorEdit[];
    replacements: Replacement[];
  }[] = [];
  // When set, `applyEdit` fails with this error.
  failWith: { code: string; message?: string } | null = null;

  constructor(
    public colors: PaletteEntry[],
    public name = "dungeon16",
  ) {}

  async loadPalette(name: string): Promise<PaletteInfo> {
    return { name, colors: this.colors.map((c) => ({ ...c })) };
  }

  async applyEdit(
    name: string,
    colors: ColorEdit[],
    replacements: Replacement[],
  ): Promise<void> {
    if (this.failWith) throw this.failWith;
    this.applied.push({ name, colors, replacements });
  }
}

export const sampleColors: PaletteEntry[] = [
  { key: "N", name: "black", hex: "#000000", used: true },
  { key: "R", name: "red", hex: "#F04545", used: true },
  { key: "V", name: "green", hex: "#4ADE80", used: false },
];
