import { onProjectChanged, spriteFrame } from "@/shared/lib/core";
import type { FramesApi, FramesInfo, SpriteRef, Tag } from "./types";

const insideApp = () => "__TAURI_INTERNALS__" in window;

async function call<T>(
  command: string,
  args?: Record<string, unknown>,
): Promise<T> {
  if (!insideApp()) {
    throw { code: "no_core", message: "Not running inside the Sprixel app." };
  }
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<T>(command, args);
}

export const coreFramesApi: FramesApi = {
  loadInfo: (sprite: SpriteRef) =>
    call<FramesInfo>("frames_info_of", { sprite }),
  loadFrameData: (sprite, frame) =>
    spriteFrame(sprite.folder, sprite.name, frame),
  insertFrame: (sprite, at, copyOf) =>
    call<number>("frames_insert", { sprite, at, copyOf }),
  deleteFrame: (sprite, frame) =>
    call<void>("frames_delete", { sprite, frame }),
  moveFrame: (sprite, from, to) =>
    call<void>("frames_move", { sprite, from, to }),
  addTag: (sprite, tag: Tag) => call<void>("tag_add", { sprite, tag }),
  setTag: (sprite, name, tag) => call<void>("tag_set", { sprite, name, tag }),
  removeTag: (sprite, name) => call<void>("tag_remove", { sprite, name }),
  onChanged: onProjectChanged,
};
