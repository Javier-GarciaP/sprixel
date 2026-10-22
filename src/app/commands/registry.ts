import type { TranslationKey } from "@/shared/lib/i18n";

export type Command = {
  id: string;
  labelKey: TranslationKey;
  // "Mod+N": Mod is Control, or Command on macOS. See shortcut.ts.
  shortcut?: string;
  // The commands of the project menu and of the native window menu.
  inProjectMenu?: boolean;
  // Draws a separator line above this item in the menus.
  separator?: boolean;
};

// The single list of commands. The project menu, the native window menu, the
// shortcuts and the shortcuts page all read from it. There is no "Save"
// command on purpose: the project saves by itself.
export const commands: Command[] = [
  {
    id: "project.new",
    labelKey: "command.project.new",
    shortcut: "Mod+N",
    inProjectMenu: true,
  },
  {
    id: "project.open",
    labelKey: "command.project.open",
    shortcut: "Mod+O",
    inProjectMenu: true,
  },
  {
    id: "project.openRecent",
    labelKey: "command.project.openRecent",
    inProjectMenu: true,
  },
  {
    id: "project.rename",
    labelKey: "command.project.rename",
    inProjectMenu: true,
    separator: true,
  },
  {
    id: "project.saveCopy",
    labelKey: "command.project.saveCopy",
    inProjectMenu: true,
  },
  {
    id: "project.showInFolder",
    labelKey: "command.project.showInFolder",
    inProjectMenu: true,
  },
  {
    id: "project.export",
    labelKey: "command.project.export",
    shortcut: "Mod+E",
    inProjectMenu: true,
    separator: true,
  },
  {
    id: "project.close",
    labelKey: "command.project.close",
    inProjectMenu: true,
    separator: true,
  },
];

export const projectMenuCommands = commands.filter((c) => c.inProjectMenu);
