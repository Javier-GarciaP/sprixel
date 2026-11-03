import type { PaletteColor } from "../types";

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
