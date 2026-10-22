// Calls to the Rust core for preferences and recent projects. In a plain
// browser (`npm run dev`) there is no core, so nothing is saved and the
// recent list is empty.

import type { Preferences } from "@/shared/lib/preferences";

export type RecentProject = { path: string; name: string };

export type StoredSettings = {
  language?: "en" | "es" | null;
  theme?: "light" | "dark" | "system" | null;
  pixelGrid?: boolean | null;
};

export type StoredPreferences = {
  settings: StoredSettings;
  recentProjects: RecentProject[];
};

export const insideApp = () => "__TAURI_INTERNALS__" in window;

async function call<T>(
  command: string,
  args: Record<string, unknown> | undefined,
  fallback: T,
): Promise<T> {
  if (!insideApp()) return fallback;
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<T>(command, args);
}

export const loadPreferences = () =>
  call<StoredPreferences>("load_preferences", undefined, {
    settings: {},
    recentProjects: [],
  });

export const saveSettings = (settings: StoredSettings) =>
  call<void>("save_settings", { settings }, undefined);

export const addRecentProject = (path: string, name: string) =>
  call<RecentProject[]>("add_recent_project", { path, name }, []);

export const removeRecentProject = (path: string) =>
  call<RecentProject[]>("remove_recent_project", { path }, []);

export const folderExists = (path: string) =>
  call<boolean>("folder_exists", { path }, true);

// Starts from the defaults and applies the choices the user already saved.
export function mergePreferences(
  defaults: Preferences,
  stored: StoredSettings,
): Preferences {
  return {
    language: stored.language ?? defaults.language,
    theme: stored.theme ?? defaults.theme,
    pixelGrid: stored.pixelGrid ?? defaults.pixelGrid,
  };
}
