import { onProjectChanged, projectTree } from "@/shared/lib/core";
import type { ExportApi, Preview } from "./types";

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

// The core sends the preview as 8 bytes (the width and the height, 4 bytes
// each, little endian) and then the RGBA pixels.
function readPreview(bytes: ArrayBuffer): Preview {
  const header = new DataView(bytes, 0, 8);
  return {
    width: header.getUint32(0, true),
    height: header.getUint32(4, true),
    rgba: new Uint8ClampedArray(bytes, 8),
  };
}

export const coreExportApi: ExportApi = {
  loadTree: projectTree,
  summary: (scope, format, options) =>
    call("export_summary", { scope, format, options }),
  preview: async (scope, options) =>
    readPreview(await call<ArrayBuffer>("export_preview", { scope, options })),
  run: (scope, format, options, output) =>
    call("export_run", { scope, format, options, output }),
  reveal: (output) => call<void>("export_reveal", { output }),
  onChanged: (listener) => onProjectChanged(() => listener()),
};
