// Calls to the Rust core for the open project. The core owns the project;
// the UI only asks and shows. Outside the app (`npm run dev`) there is no
// core, so every call fails with an `io` error.

import { insideApp } from "./preferences";

export type ProjectInfo = {
  name: string;
  path: string;
  // Files that could not be loaded. The project opens without them.
  damagedFiles: number;
};

// The codes the core sends. The UI translates them.
export type ProjectFailure =
  | { code: "folder_exists" }
  | { code: "not_a_project" }
  | { code: "invalid_name"; message: string }
  | { code: "io"; message: string };

export type SaveStatus =
  | { status: "saved" }
  | { status: "saving" }
  | { status: "error"; reason: string };

// `invoke` rejects with whatever the core sent. Anything that is not one of
// the codes becomes an `io` error with its text.
export function asProjectFailure(error: unknown): ProjectFailure {
  if (typeof error === "object" && error !== null && "code" in error) {
    return error as ProjectFailure;
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

export const defaultProjectLocation = () =>
  call<string>("default_project_location");

export const createProject = (parent: string, name: string) =>
  call<ProjectInfo>("create_project", { parent, name });

export const openProject = (path: string) =>
  call<ProjectInfo>("open_project", { path });

export const closeProject = () => call<void>("close_project");

// `null` when no project is open.
export const projectStatus = () => call<SaveStatus | null>("project_status");

export const retrySave = () => call<SaveStatus | null>("retry_save");

// The native folder picker. `null` when the user cancels.
export async function pickFolder(): Promise<string | null> {
  if (!insideApp()) return null;
  const { open } = await import("@tauri-apps/plugin-dialog");
  const chosen = await open({ directory: true });
  return typeof chosen === "string" ? chosen : null;
}
