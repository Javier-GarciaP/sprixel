// Calls to the Rust core for the palette editor. The core owns the project;
// these functions only ask and show. Outside the app (`npm run dev`) there is
// no core, so every call fails.

import type { PaletteApi, PaletteFailure, PaletteInfo, Preset } from "./types";

const insideApp = () => "__TAURI_INTERNALS__" in window;

export function asPaletteFailure(error: unknown): PaletteFailure {
  if (typeof error === "object" && error !== null && "code" in error) {
    return error as PaletteFailure;
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

export const corePaletteApi: PaletteApi = {
  loadPalette: (name) => call<PaletteInfo>("palette_info_of", { name }),
  applyEdit: (name, colors, replacements) =>
    call<void>("palette_apply", { name, colors, replacements }),
  listPresets: () => call<Preset[]>("palette_presets"),
  createPalette: (name, preset) =>
    call<string>("palette_create", { name, preset }),
  async pickPaletteFile() {
    if (!insideApp()) return null;
    const { open } = await import("@tauri-apps/plugin-dialog");
    const chosen = await open({
      filters: [{ name: "Palette", extensions: ["gpl", "hex"] }],
    });
    return typeof chosen === "string" ? chosen : null;
  },
  importPalette: (name, path) => call<string>("palette_import", { name, path }),
};
