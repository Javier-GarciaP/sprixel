// Calls to the Rust core for the contents of the open project. The core owns
// the project; the UI asks and shows. Outside the app (`npm run dev`) there
// is no core, so every call fails with a `no_core` error.

import type {
  ChangedEvent,
  CoreFailure,
  FrameData,
  NameProblem,
  ProjectTree,
  SpriteTarget,
} from "./types";

export type * from "./types";

const insideApp = () => "__TAURI_INTERNALS__" in window;

// `invoke` rejects with whatever the core sent. Anything that is not an
// error with a code becomes one with its text.
export function asCoreFailure(error: unknown): CoreFailure {
  if (
    typeof error === "object" &&
    error !== null &&
    "code" in error &&
    typeof error.code === "string"
  ) {
    const { code, message } = error as { code: string; message?: unknown };
    return { code, message: typeof message === "string" ? message : "" };
  }
  return { code: "io", message: String(error) };
}

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

export const projectTree = () => call<ProjectTree>("project_tree");

// Frames are numbered from 1.
export const spriteFrame = (folder: string[], name: string, frame: number) =>
  call<FrameData>("sprite_frame", { folder, name, frame });

// Asks if a name can be used inside the folder `parent`. `current` is the
// name of the item that is being renamed.
export const checkName = (parent: string[], name: string, current?: string) =>
  call<NameProblem | null>("check_name", {
    parent,
    name,
    current: current ?? null,
  });

// With no palette the core uses `main` (and creates it when it is missing).
export const createSprite = (
  folder: string[],
  name: string,
  width: number,
  height: number,
  palette: string | null,
) =>
  call<SpriteTarget>("create_sprite", { folder, name, width, height, palette });

export const createFolder = (parent: string[], name: string) =>
  call<string[]>("create_folder", { parent, name });

// Changes the name of a sprite. It keeps its content and its history.
export const renameSprite = (folder: string[], name: string, newName: string) =>
  call<SpriteTarget>("rename_sprite", { folder, name, newName });

// Returns the new path of the folder.
export const renameFolder = (path: string[], newName: string) =>
  call<string[]>("rename_folder", { path, newName });

// Adds a copy named "goblin copy", "goblin copy 2"... Returns the copy.
export const duplicateSprite = (folder: string[], name: string) =>
  call<SpriteTarget>("duplicate_sprite", { folder, name });

// Moves sprites to the trash in one step. Returns the revision of the
// project after the change.
export const deleteSprites = (sprites: SpriteTarget[]) =>
  call<number>("delete_sprites", { sprites });

// Moves sprites to another folder (an empty path is the project root), in one
// step. Fails with `name_exists` if a name is taken in the destination.
export const moveSprites = (sprites: SpriteTarget[], to: string[]) =>
  call<void>("move_sprites", { sprites, to });

// Undo and redo the last change of the project, whoever made it. They fail
// with the code `nothing_to_undo` or `nothing_to_redo`.
export const projectUndo = () => call<void>("project_undo");
export const projectRedo = () => call<void>("project_redo");

// Runs `listener` after every change to the open project. Returns a function
// that stops listening. Outside the app nothing is ever emitted.
export async function onProjectChanged(
  listener: (event: ChangedEvent) => void,
): Promise<() => void> {
  if (!insideApp()) return () => {};
  const { listen } = await import("@tauri-apps/api/event");
  return listen<ChangedEvent>("project://changed", (event) =>
    listener(event.payload),
  );
}
