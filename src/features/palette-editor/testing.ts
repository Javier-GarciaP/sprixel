import type {
  ColorEdit,
  PaletteApi,
  PaletteEntry,
  PaletteInfo,
  Preset,
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

  presets: Preset[] = [
    {
      id: "pocket-rpg-16",
      name: "Pocket RPG 16",
      colors: Array.from({ length: 16 }, (_, i) => ({
        key: String.fromCharCode(65 + i),
        hex: `#${(i * 16).toString(16).padStart(2, "0")}0000`,
      })),
    },
    {
      id: "farm-32",
      name: "Farm 32",
      colors: Array.from({ length: 32 }, (_, i) => ({
        key: String.fromCharCode(33 + i),
        hex: `#00${(i * 8).toString(16).padStart(2, "0")}00`,
      })),
    },
  ];
  created: { name: string; preset: string | null }[] = [];
  // When set, `createPalette` fails with this error.
  createFailWith: { code: string; message?: string } | null = null;

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

  async listPresets(): Promise<Preset[]> {
    return this.presets;
  }

  async createPalette(name: string, preset: string | null): Promise<string> {
    if (this.createFailWith) throw this.createFailWith;
    this.created.push({ name, preset });
    return name;
  }
}

export const sampleColors: PaletteEntry[] = [
  { key: "N", name: "black", hex: "#000000", used: true },
  { key: "R", name: "red", hex: "#F04545", used: true },
  { key: "V", name: "green", hex: "#4ADE80", used: false },
];
