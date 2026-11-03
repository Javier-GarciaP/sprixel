import type {
  GridCheck,
  GridError,
  GridFailure,
  SpriteRef,
  TextApi,
} from "./types";

// The 8x8 goblin of the documentation, as the core writes it.
export const GOBLIN_ROWS = [
  "....RRRR",
  "...RRRRR",
  "...NNNN.",
  "..NPNPPN",
  "..NPPPPN",
  "..NNNNNN",
  ".NAAAAN.",
  ".NA..AN.",
];

export const goblinText = (rows = GOBLIN_ROWS, frame = 1) =>
  [
    "palette: dungeon16",
    "",
    "legend",
    "  . transparent",
    "  N black  #000000",
    "  P skin   #FFCC99",
    "  A blue   #2F6BFF",
    "  R red    #F04545",
    "",
    `frame ${frame}`,
    ...rows,
    "",
  ].join("\n");

const COLORS = [
  { key: "N", hex: "#000000" },
  { key: "P", hex: "#FFCC99" },
  { key: "A", hex: "#2F6BFF" },
  { key: "R", hex: "#F04545" },
];

// An in-memory stand-in for the Rust core, for the tests. It checks the two
// simplest rules (the length of the rows and the keys) the way the core does
// and records what the view sends. The real rules are tested in Rust.
export class FakeTextApi implements TextApi {
  size = { width: 8, height: 8 };
  // The text of each frame, as the core would write it.
  texts: Record<number, string> = { 1: goblinText() };
  loaded: number[] = [];
  checked: string[] = [];
  applied: string[] = [];
  clipboard = "";
  // When set, `apply` fails with this.
  failApply: GridFailure | null = null;
  // When set, the clipboard cannot be read.
  clipboardBlocked = false;

  async loadText(_sprite: SpriteRef, frame: number) {
    this.loaded.push(frame);
    return this.texts[frame] ?? goblinText(GOBLIN_ROWS, frame);
  }

  async check(_sprite: SpriteRef, text: string): Promise<GridCheck> {
    this.checked.push(text);
    const errors: GridError[] = [];
    const lines = text.split("\n");
    const start = lines.findIndex((line) => line.startsWith("frame "));
    const rows = lines
      .slice(start + 1)
      .map((content, i) => ({ content, line: start + 2 + i }))
      .filter((row) => row.content.trim() !== "");
    const keys = new Set([".", ...COLORS.map((c) => c.key)]);
    rows.forEach((row, i) => {
      if (row.content.length !== this.size.width) {
        errors.push({
          code: "row_length",
          message: `line ${row.line} (row ${i + 1} of frame 1): ${row.content.length} columns, expected ${this.size.width}`,
          line: row.line,
          row: i + 1,
          column: null,
        });
      }
      [...row.content].forEach((key, column) => {
        if (!keys.has(key)) {
          errors.push({
            code: "unknown_key",
            message: `line ${row.line} (row ${i + 1}, column ${column + 1}): key "${key}" is not in palette dungeon16`,
            line: row.line,
            row: i + 1,
            column: column + 1,
          });
        }
      });
    });
    if (errors.length > 0) return { errors, summary: null, preview: null };
    const used = new Set(rows.flatMap((row) => [...row.content]));
    used.delete(".");
    return {
      errors,
      summary: {
        width: this.size.width,
        height: rows.length,
        keys: used.size,
      },
      preview: {
        width: this.size.width,
        height: rows.length,
        rows: rows.map((row) => row.content),
        colors: COLORS,
      },
    };
  }

  async apply(_sprite: SpriteRef, text: string) {
    if (this.failApply) throw this.failApply;
    this.applied.push(text);
  }

  async readClipboard() {
    if (this.clipboardBlocked) throw new Error("denied");
    return this.clipboard;
  }

  async writeClipboard(text: string) {
    this.clipboard = text;
  }
}
