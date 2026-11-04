// @vitest-environment jsdom
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

const invoke = vi.hoisted(() => vi.fn());
vi.mock("@tauri-apps/api/core", () => ({ invoke }));

import { corePowerApi } from "./power";

const sprite = { folder: [], name: "goblin" };

beforeEach(() => {
  (window as unknown as Record<string, unknown>).__TAURI_INTERNALS__ = {};
  invoke.mockResolvedValue(undefined);
});
afterEach(() => {
  delete (window as unknown as Record<string, unknown>).__TAURI_INTERNALS__;
  invoke.mockReset();
});

describe("core power api", () => {
  // Checks that each call uses the command and the argument names the Rust
  // side expects (Tauri turns snake_case into camelCase), so a typo here is
  // found by a test and not by a person with a broken button.
  it("sends the commands with the names of the core", async () => {
    const clip = { palette: "main", width: 1, height: 1, keys: "R" };
    await corePowerApi.region(sprite, 2, null, { kind: "flip_vertical" });
    await corePowerApi.paste(sprite, 2, clip, { x: 3, y: 4 }, true);
    await corePowerApi.review(sprite, "K");
    expect(invoke.mock.calls).toEqual([
      [
        "power_region",
        { sprite, frame: 2, region: null, action: { kind: "flip_vertical" } },
      ],
      [
        "power_paste",
        { sprite, frame: 2, clip, x: 3, y: 4, skipTransparent: true },
      ],
      ["power_review", { sprite, outlineKey: "K" }],
    ]);
  });

  // Checks that outside the app a call fails with a code the UI can show.
  it("fails outside the app", async () => {
    delete (window as unknown as Record<string, unknown>).__TAURI_INTERNALS__;
    await expect(corePowerApi.anchors(sprite)).rejects.toMatchObject({
      code: "io",
    });
  });
});
