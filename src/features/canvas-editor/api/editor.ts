// Calls to the Rust core for the editor. The core owns the project; these
// functions only ask and show. Outside the app (`npm run dev`) there is no
// core, so every call fails.

import type {
  AgentActivity,
  ApplyResult,
  EditorApi,
  EditorFailure,
  HistoryState,
  PixelChange,
  SpriteInfo,
  SpriteRef,
} from "./types";

const insideApp = () => "__TAURI_INTERNALS__" in window;

export function asEditorFailure(error: unknown): EditorFailure {
  if (typeof error === "object" && error !== null && "code" in error) {
    return error as EditorFailure;
  }
  return { code: "io", message: String(error) };
}

async function call<T>(
  command: string,
  args?: Record<string, unknown>,
): Promise<T> {
  if (!insideApp()) {
    throw { code: "io", message: "Not running inside the Sprixel app." };
  }
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<T>(command, args);
}

export const coreEditorApi: EditorApi = {
  loadSprite: (sprite: SpriteRef) =>
    call<SpriteInfo>("editor_sprite", { sprite }),
  loadFrame: (sprite: SpriteRef, frame: number) =>
    call<string>("editor_frame", { sprite, frame }),
  applyChanges: (sprite: SpriteRef, frame: number, changes: PixelChange[]) =>
    call<ApplyResult>("editor_apply", { sprite, frame, changes }),
  // Undo and redo are the ones of the project, the same for every tab. They
  // send `project://changed`; then the editor reads the history flags.
  undo: async () => {
    await call<void>("project_undo");
    return call<HistoryState>("editor_history");
  },
  redo: async () => {
    await call<void>("project_redo");
    return call<HistoryState>("editor_history");
  },
  agentActivity: () => call<AgentActivity | null>("agent_activity"),
  stopAgent: () => call<void>("agent_stop"),
  async onAgentActivity(callback) {
    if (!insideApp()) return () => {};
    const { listen } = await import("@tauri-apps/api/event");
    return listen<AgentActivity | null>("agent-activity", (event) =>
      callback(event.payload),
    );
  },
};
