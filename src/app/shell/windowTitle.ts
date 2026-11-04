import { useEffect } from "react";

// "Dungeon Kit - Sprixel", or just "Sprixel" when no project is open.
export function windowTitle(projectName: string | null): string {
  return projectName ? `${projectName} - Sprixel` : "Sprixel";
}

// Sets the title of the page and, inside the app, of the native window.
export function useWindowTitle(projectName: string | null) {
  useEffect(() => {
    const title = windowTitle(projectName);
    document.title = title;
    if ("__TAURI_INTERNALS__" in window) {
      import("@tauri-apps/api/window").then(({ getCurrentWindow }) =>
        getCurrentWindow().setTitle(title),
      );
    }
  }, [projectName]);
}
