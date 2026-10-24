// The data the Rust core sends about the open project. The names are the
// ones of the Rust structs (see src-tauri/src/ipc/browser.rs).

export type Tag = {
  name: string;
  from: number;
  to: number;
  fps: number;
  mode: "loop" | "once" | "pingpong";
};

export type SpriteSummary = {
  name: string;
  folder: string[];
  width: number;
  height: number;
  frames: number;
  palette: string;
  tags: Tag[];
  // Who changed the sprite last in this session. `author` is "user" or the
  // name of an AI client.
  lastEditor: { author: string; atMs: number } | null;
};

export type FolderNode = {
  name: string;
  path: string[];
  folders: FolderNode[];
  sprites: SpriteSummary[];
};

export type PaletteSummary = { name: string; colors: number };

export type ProjectTree = {
  name: string;
  revision: number;
  // The project root. Its own name is not used.
  root: FolderNode;
  palettes: PaletteSummary[];
};

// One frame, ready to draw: rows of palette keys (`.` is transparent) and
// the colors of the palette.
export type FrameData = {
  revision: number;
  width: number;
  height: number;
  rows: string[];
  colors: { key: string; hex: string }[];
};

// The payload of the `project://changed` event.
export type ChangedEvent = {
  revision: number;
  touched: { kind: "sprite" | "folder" | "palette"; path: string[] }[];
};

// An error of the core, with a stable code the UI can translate.
export type CoreFailure = { code: string; message: string };
