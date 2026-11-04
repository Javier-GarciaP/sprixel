import { useEffect, useRef } from "react";

export type CommandHandlers = Record<string, () => void>;

// Gives the app the handlers of the editor commands (tool keys, zoom, undo
// and so on) while the editor is open, and takes them back when it closes.
// The app owns the shortcut keys; the editor only says what each command does.
// The handlers always call the latest `actions`, so they are registered once.
export function useEditorCommands(
  register: ((handlers: CommandHandlers | null) => void) | undefined,
  actions: CommandHandlers,
) {
  const latest = useRef(actions);
  latest.current = actions;

  useEffect(() => {
    if (!register) return;
    const handlers: CommandHandlers = {};
    for (const id of Object.keys(latest.current)) {
      handlers[id] = () => latest.current[id]?.();
    }
    register(handlers);
    return () => register(null);
  }, [register]);
}
