import { useEffect } from "react";
import { useT, type TranslationKey } from "@/shared/lib/i18n";
import { projectMenuCommands } from "./registry";

type Translate = (key: TranslationKey) => string;

// "Mod+Shift+Z" becomes "CmdOrCtrl+Shift+Z", the form Tauri expects.
export function toAccelerator(shortcut: string): string {
  return shortcut.replace("Mod", "CmdOrCtrl");
}

// The commands sent to the native menu: the same list as the project menu.
export function nativeMenuCommands(
  t: Translate,
  handlers: Record<string, () => void>,
) {
  return projectMenuCommands.map((command) => ({
    id: command.id,
    label: t(command.labelKey),
    accelerator: command.shortcut ? toAccelerator(command.shortcut) : null,
    enabled: Boolean(handlers[command.id]),
    separator: command.separator ?? false,
  }));
}

const insideApp = () => "__TAURI_INTERNALS__" in window;

// Keeps the native window menu in step with the registry, the language and
// the available handlers, and runs the command that the user chooses there.
export function useNativeMenu(handlers: Record<string, () => void>) {
  const t = useT();

  useEffect(() => {
    if (!insideApp()) return;
    import("@tauri-apps/api/core").then(({ invoke }) =>
      invoke("set_menu", {
        fileTitle: t("menu.file"),
        editTitle: t("menu.edit"),
        commands: nativeMenuCommands(t, handlers),
      }),
    );
  }, [t, handlers]);

  useEffect(() => {
    if (!insideApp()) return;
    const unlisten = import("@tauri-apps/api/event").then(({ listen }) =>
      listen<string>("menu-command", (event) => handlers[event.payload]?.()),
    );
    return () => {
      unlisten.then((stop) => stop());
    };
  }, [handlers]);
}
