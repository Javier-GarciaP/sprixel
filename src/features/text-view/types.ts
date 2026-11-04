// A sprite is found by its folder path and its name (see `SpriteRef` in the
// core).
export type SpriteRef = { folder: string[]; name: string };

// One problem of a text grid. `line`, `row` and `column` count from 1 and are
// null when they do not apply.
export type GridError = {
  code: string;
  message: string;
  line: number | null;
  row: number | null;
  column: number | null;
};

// What the status line says about a valid grid.
export type GridSummary = { width: number; height: number; keys: number };

// The first frame of a valid grid, ready to be drawn: rows of palette keys and
// the color of every key.
export type GridPreview = {
  width: number;
  height: number;
  rows: string[];
  colors: { key: string; hex: string }[];
};

// The result of checking a text. A valid grid has a summary and a preview; an
// invalid one has errors.
export type GridCheck = {
  errors: GridError[];
  summary: GridSummary | null;
  preview: GridPreview | null;
};

// Why a grid was not applied. `errors` has every problem of the text when the
// grid is not valid.
export type GridFailure = {
  code: string;
  message?: string;
  errors?: GridError[];
};

// Everything the Text view asks of the Rust core. The real calls are in
// `api.ts`; the tests use a fake with the same shape.
export interface TextApi {
  // The text of one frame (from 1): the format the AI tools read and write.
  loadText(sprite: SpriteRef, frame: number): Promise<string>;
  // Checks a text against the sprite. Nothing changes.
  check(sprite: SpriteRef, text: string): Promise<GridCheck>;
  // Writes the text into the sprite as one undoable action. Rejects with a
  // `GridFailure` when it is not valid.
  apply(sprite: SpriteRef, text: string): Promise<void>;
  readClipboard(): Promise<string>;
  writeClipboard(text: string): Promise<void>;
}
