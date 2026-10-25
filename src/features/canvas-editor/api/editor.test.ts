// @vitest-environment jsdom
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

const invoke = vi.hoisted(() => vi.fn());
vi.mock("@tauri-apps/api/core", () => ({ invoke }));

import { coreEditorApi } from "./editor";

beforeEach(() => {
  (window as unknown as Record<string, unknown>).__TAURI_INTERNALS__ = {};
  invoke.mockImplementation(async (command: string) =>
    command === "editor_history"
      ? { canUndo: false, canRedo: true, revision: 7 }
      : undefined,
  );
});
afterEach(() => {
  delete (window as unknown as Record<string, unknown>).__TAURI_INTERNALS__;
  invoke.mockReset();
});

describe("core editor api, undo and redo", () => {
  // Checks that there is one history path: undo and redo of the editor are
  // the commands of the project (the same as the other tabs, which also send
  // the project://changed event), followed by a read of the history flags.
  it("uses project_undo and project_redo, then reads the history", async () => {
    expect(await coreEditorApi.undo()).toEqual({
      canUndo: false,
      canRedo: true,
      revision: 7,
    });
    await coreEditorApi.redo();
    expect(invoke.mock.calls.map((c) => c[0])).toEqual([
      "project_undo",
      "editor_history",
      "project_redo",
      "editor_history",
    ]);
  });
});
