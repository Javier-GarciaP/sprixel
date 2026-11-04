// Calls to the Rust core for the power tools of the editor. The core owns the
// project and does the work; these functions only ask. Outside the app
// (`npm run dev`) there is no core, so every call fails.

import { asEditorFailure } from "./editor";
import type { PowerApi } from "./types";

async function call<T>(
  command: string,
  args?: Record<string, unknown>,
): Promise<T> {
  if (!("__TAURI_INTERNALS__" in window)) {
    throw asEditorFailure({
      code: "io",
      message: "Not running inside Sprixel.",
    });
  }
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<T>(command, args);
}

export const corePowerApi: PowerApi = {
  region: (sprite, frame, region, action) =>
    call("power_region", { sprite, frame, region, action }),
  copy: (sprite, frame, region) =>
    call("power_copy", { sprite, frame, region }),
  paste: (sprite, frame, clip, at, skipTransparent) =>
    call("power_paste", {
      sprite,
      frame,
      clip,
      x: at.x,
      y: at.y,
      skipTransparent,
    }),
  outlinePreview: (sprite, frame, args) =>
    call("power_outline_preview", { sprite, frame, args }),
  outlineApply: (sprite, frame, args) =>
    call("power_outline_apply", { sprite, frame, args }),
  shadePreview: (sprite, frame, args) =>
    call("power_shade_preview", { sprite, frame, args }),
  shadeApply: (sprite, frame, args) =>
    call("power_shade_apply", { sprite, frame, args }),
  review: (sprite, outlineKey) => call("power_review", { sprite, outlineKey }),
  tileCheck: (sprite, frame) => call("power_tile_check", { sprite, frame }),
  anchors: (sprite) => call("power_anchors", { sprite }),
  saveAnchor: (sprite, anchor, replaces) =>
    call("power_anchor_save", { sprite, anchor, replaces }),
  deleteAnchor: (sprite, key) => call("power_anchor_delete", { sprite, key }),
};
