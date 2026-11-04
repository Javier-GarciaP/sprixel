import type { PaletteColor, Rect } from "../types";

// A sprite is found by its folder path and its name (see `SpriteRef` in the
// core).
export type SpriteRef = { folder: string[]; name: string };

export type HistoryState = {
  canUndo: boolean;
  canRedo: boolean;
  revision: number;
};

export type SpriteInfo = HistoryState & {
  width: number;
  height: number;
  frameCount: number;
  // The name of the palette of the sprite, for the palette editor.
  paletteName: string;
  palette: PaletteColor[];
};

// One changed pixel. `color` is `#RRGGBB`, or null for a transparent pixel.
// The core finds or adds the palette key for the color.
export type PixelChange = { x: number; y: number; color: string | null };

export type ApplyResult = HistoryState & { palette: PaletteColor[] };

// An AI agent writing to a frame. The UI shows the marker while it lasts.
export type AgentActivity = { agent: string; sprite: SpriteRef; frame: number };

// The pixels of one agent write in drawing order (event `agent-ops`). `id` is
// the number of the write: the editor sends it back when the replay is done.
// Each pixel is `[x, y, key]`.
export type AgentOps = {
  id: number;
  agent: string;
  sprite: SpriteRef;
  frame: number;
  tool: string;
  pixels: [number, number, string][];
};

// The codes of the errors the core sends for the editor.
export type EditorFailure =
  | { code: "palette_full" }
  | { code: "frame_busy" }
  | { code: string; message?: string };

// Everything the editor asks of the Rust core. The real calls are in
// `editor.ts`; the tests use a fake with the same shape.
export interface EditorApi {
  loadSprite(sprite: SpriteRef): Promise<SpriteInfo>;
  // The pixels of a frame: one palette key per pixel, row by row.
  loadFrame(sprite: SpriteRef, frame: number): Promise<string>;
  // One stroke, fill, shape or move: one history entry.
  applyChanges(
    sprite: SpriteRef,
    frame: number,
    changes: PixelChange[],
  ): Promise<ApplyResult>;
  undo(): Promise<HistoryState>;
  redo(): Promise<HistoryState>;
  agentActivity(): Promise<AgentActivity | null>;
  // Calls back when an agent starts or ends a write (null). Returns the
  // function that stops listening.
  onAgentActivity(
    callback: (activity: AgentActivity | null) => void,
  ): Promise<() => void>;
  stopAgent(): Promise<void>;
  // Calls back with the pixels of every agent write, to replay them.
  onAgentOps(callback: (ops: AgentOps) => void): Promise<() => void>;
  // Tells the core that the replay of write `id` ended, so the frame is free.
  playbackDone(id: number): Promise<void>;
}

// ---- The power tools (see `power.rs` in the core) ----

export type Direction =
  "left_to_right" | "right_to_left" | "top_to_bottom" | "bottom_to_top";

// A change of a region of one frame.
export type RegionAction =
  | { kind: "flip_horizontal" }
  | { kind: "flip_vertical" }
  | { kind: "rotate"; degrees: 90 | 180 | 270 }
  | { kind: "move"; dx: number; dy: number; wrap: boolean }
  | { kind: "mirror"; direction: Direction };

// What a write of the power tools answers.
export type Applied = HistoryState & { pixelsChanged: number };

// A copied region: one key per pixel, and the palette they belong to.
export type Clip = {
  palette: string;
  width: number;
  height: number;
  keys: string;
};

export type OutlineArgs = {
  key: string;
  connectivity: 4 | 8;
  outsideOnly: boolean;
  inner: boolean;
};

export type Light =
  | "top-left"
  | "top"
  | "top-right"
  | "left"
  | "right"
  | "bottom-left"
  | "bottom"
  | "bottom-right";

export type ShadeArgs = {
  light: Light;
  base: string;
  // The ramp from dark to light; it includes `base`.
  keys: string[];
  region: Rect | null;
};

// The pixels a preview would change: [x, y, key]. Nothing is written.
export type Preview = { pixels: [number, number, string][] };

export type FrameReview = {
  frame: number;
  loose: [number, number][];
  gaps: [number, number][];
};

export type ChangeReview = {
  from: number;
  to: number;
  changedPixels: number;
  bbox: Rect | null;
  groundKept: boolean;
};

export type Review = {
  used: string[];
  unused: string[];
  frames: FrameReview[];
  changes: ChangeReview[];
};

export type TileMismatch = {
  edge: "left_right" | "top_bottom";
  a: [number, number];
  b: [number, number];
};

export type TileCheck = { seamless: boolean; mismatches: TileMismatch[] };

// A named point of a sprite, or of one frame of it.
export type Anchor = {
  name: string;
  x: number;
  y: number;
  frame: number | null;
};

// Which anchor a save replaces (a rename or a move).
export type AnchorKey = { name: string; frame: number | null };

// What the editor asks of the core for the power tools. The real calls are in
// `power.ts`; the tests use the fake.
export interface PowerApi {
  region(
    sprite: SpriteRef,
    frame: number,
    region: Rect | null,
    action: RegionAction,
  ): Promise<Applied>;
  copy(sprite: SpriteRef, frame: number, region: Rect | null): Promise<Clip>;
  paste(
    sprite: SpriteRef,
    frame: number,
    clip: Clip,
    at: { x: number; y: number },
    skipTransparent: boolean,
  ): Promise<Applied>;
  outlinePreview(
    sprite: SpriteRef,
    frame: number,
    args: OutlineArgs,
  ): Promise<Preview>;
  outlineApply(
    sprite: SpriteRef,
    frame: number,
    args: OutlineArgs,
  ): Promise<Applied>;
  shadePreview(
    sprite: SpriteRef,
    frame: number,
    args: ShadeArgs,
  ): Promise<Preview>;
  shadeApply(
    sprite: SpriteRef,
    frame: number,
    args: ShadeArgs,
  ): Promise<Applied>;
  review(sprite: SpriteRef, outlineKey: string | null): Promise<Review>;
  tileCheck(sprite: SpriteRef, frame: number): Promise<TileCheck>;
  anchors(sprite: SpriteRef): Promise<Anchor[]>;
  saveAnchor(
    sprite: SpriteRef,
    anchor: Anchor,
    replaces: AnchorKey | null,
  ): Promise<void>;
  deleteAnchor(sprite: SpriteRef, key: AnchorKey): Promise<void>;
}
