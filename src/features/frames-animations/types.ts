import type { ChangedEvent, FrameData } from "@/shared/lib/core";

export type SpriteRef = { folder: string[]; name: string };

export type RepeatMode = "loop" | "once" | "pingpong";

// An animation: a range of frames (from 1, both ends included) with a speed.
export type Tag = {
  name: string;
  from: number;
  to: number;
  fps: number;
  mode: RepeatMode;
};

export type FramesInfo = {
  frameCount: number;
  // One duration in milliseconds per frame. The default means "none of its own".
  durations: number[];
  defaultMs: number;
  tags: Tag[];
};

// Everything the frames, animations and preview ask of the Rust core. The real
// calls are in `api.ts`; the tests use a fake with the same shape. Changes are
// normal operations of the project, so the undo is the one of the project.
export interface FramesApi {
  loadInfo(sprite: SpriteRef): Promise<FramesInfo>;
  loadFrameData(sprite: SpriteRef, frame: number): Promise<FrameData>;
  // Puts a frame at `at`, blank or a copy of `copyOf`. Returns its number.
  insertFrame(
    sprite: SpriteRef,
    at: number,
    copyOf: number | null,
  ): Promise<number>;
  deleteFrame(sprite: SpriteRef, frame: number): Promise<void>;
  moveFrame(sprite: SpriteRef, from: number, to: number): Promise<void>;
  addTag(sprite: SpriteRef, tag: Tag): Promise<void>;
  setTag(sprite: SpriteRef, name: string, tag: Tag): Promise<void>;
  removeTag(sprite: SpriteRef, name: string): Promise<void>;
  // Calls back after every change of the project (`project://changed`).
  onChanged(listener: (event: ChangedEvent) => void): Promise<() => void>;
}

export type FramesFailure = { code: string; message?: string };
