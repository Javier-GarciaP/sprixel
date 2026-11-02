// Calls to the Rust core for the Text view. The core owns the project and the
// grid rules; these functions only ask and show. Outside the app
// (`npm run dev`) there is no core, so every call fails.

import type { GridCheck, GridFailure, TextApi } from "./types";

const insideApp = () => "__TAURI_INTERNALS__" in window;

export function asGridFailure(error: unknown): GridFailure {
  if (typeof error === "object" && error !== null && "code" in error) {
    return error as GridFailure;
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

export const coreTextApi: TextApi = {
  loadText: (sprite, frame) => call<string>("grid_text_of", { sprite, frame }),
  check: (sprite, text) => call<GridCheck>("grid_check", { sprite, text }),
  apply: (sprite, text) => call<void>("grid_apply", { sprite, text }),
  readClipboard: () => navigator.clipboard.readText(),
  writeClipboard: (text) => navigator.clipboard.writeText(text),
};
