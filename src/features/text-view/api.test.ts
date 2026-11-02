// @vitest-environment jsdom
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

const invoke = vi.hoisted(() => vi.fn());
vi.mock("@tauri-apps/api/core", () => ({ invoke }));

import { asGridFailure, coreTextApi } from "./api";

const sprite = { folder: ["Personajes"], name: "goblin" };

beforeEach(() => {
  (window as unknown as Record<string, unknown>).__TAURI_INTERNALS__ = {};
});
afterEach(() => {
  delete (window as unknown as Record<string, unknown>).__TAURI_INTERNALS__;
  invoke.mockReset();
});

describe("core text api", () => {
  // Checks that each call is the command of the core with the arguments it
  // expects, and that the answer of the core goes back unchanged.
  it("calls the grid commands of the core", async () => {
    invoke.mockResolvedValue("palette: main\n");
    expect(await coreTextApi.loadText(sprite, 2)).toBe("palette: main\n");
    invoke.mockResolvedValue({ errors: [], summary: null, preview: null });
    await coreTextApi.check(sprite, "frame 1\n.\n");
    invoke.mockResolvedValue(undefined);
    await coreTextApi.apply(sprite, "frame 1\n.\n");
    expect(invoke.mock.calls).toEqual([
      ["grid_text_of", { sprite, frame: 2 }],
      ["grid_check", { sprite, text: "frame 1\n.\n" }],
      ["grid_apply", { sprite, text: "frame 1\n.\n" }],
    ]);
  });

  // Checks that a grid the core refuses rejects with the list of errors, so
  // the view can show every one of them.
  it("rejects with the errors of an invalid grid", async () => {
    const failure = {
      code: "row_length",
      message: "line 3 (row 1 of frame 1): 1 columns, expected 2",
      errors: [
        {
          code: "row_length",
          message: "line 3 (row 1 of frame 1): 1 columns, expected 2",
          line: 3,
          row: 1,
          column: null,
        },
      ],
    };
    invoke.mockRejectedValue(failure);
    const error = await coreTextApi.apply(sprite, "x").catch(asGridFailure);
    expect(error.errors).toHaveLength(1);
    expect(error.code).toBe("row_length");
  });

  // Checks that outside the app (no core) a call fails with a clear code,
  // and that anything that is not an error object becomes an `io` failure.
  it("fails with no_core outside the app", async () => {
    delete (window as unknown as Record<string, unknown>).__TAURI_INTERNALS__;
    const error = await coreTextApi.check(sprite, "x").catch(asGridFailure);
    expect(error.code).toBe("no_core");
    expect(invoke).not.toHaveBeenCalled();
    expect(asGridFailure("boom")).toEqual({ code: "io", message: "boom" });
  });
});
