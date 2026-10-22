import { useEffect } from "react";
import { commands } from "./registry";
import { matchesShortcut, type Platform } from "./shortcut";

// Runs the handler of the command whose shortcut was pressed. Commands
// without a handler are ignored, so a shortcut does nothing until the
// feature that owns it registers a handler. Plain letter keys are ignored
// while the user types in a field.
export function useShortcuts(
  handlers: Record<string, () => void>,
  platform: Platform,
) {
  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      const typing = (event.target as HTMLElement | null)?.closest?.(
        "input, textarea, select, [contenteditable]",
      );
      for (const command of commands) {
        const handler = handlers[command.id];
        if (!command.shortcut || !handler) continue;
        if (typing && !command.shortcut.includes("Mod")) continue;
        if (matchesShortcut(event, command.shortcut, platform)) {
          event.preventDefault();
          handler();
          return;
        }
      }
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [handlers, platform]);
}
