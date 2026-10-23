import type { TranslationKey } from "@/shared/lib/i18n";

export type Group = "project" | "tools" | "editor";

export type Command = {
  id: string;
  labelKey: TranslationKey;
  // The area of the shortcuts page.
  group: Group;
  // "Mod+N": Mod is Control, or Command on macOS. See shortcut.ts.
  shortcut?: string;
  // The commands of the project menu.
  inProjectMenu?: boolean;
  // Also done with Control (Command on macOS) and the mouse wheel. The wheel
  // is only shown on the shortcuts page; the real zoom belongs to the canvas
  // spec.
  withWheel?: boolean;
  // Draws a separator line above this item in the project menu.
  separator?: boolean;
};

// The single list of commands. The project menu, the shortcuts and the shortcuts page all read from it. There is no "Save"
// command on purpose: the project saves by itself.
export const commands: Command[] = [
  {
    group: "project",
    id: "project.new",
    labelKey: "command.project.new",
    shortcut: "Mod+N",
    inProjectMenu: true,
  },
  {
    group: "project",
    id: "project.open",
    labelKey: "command.project.open",
    shortcut: "Mod+O",
    inProjectMenu: true,
  },
  {
    group: "project",
    id: "project.openRecent",
    labelKey: "command.project.openRecent",
    inProjectMenu: true,
  },
  {
    group: "project",
    id: "project.rename",
    labelKey: "command.project.rename",
    inProjectMenu: true,
    separator: true,
  },
  {
    group: "project",
    id: "project.saveCopy",
    labelKey: "command.project.saveCopy",
    inProjectMenu: true,
  },
  {
    group: "project",
    id: "project.showInFolder",
    labelKey: "command.project.showInFolder",
    inProjectMenu: true,
  },
  {
    group: "project",
    id: "project.export",
    labelKey: "command.project.export",
    shortcut: "Mod+E",
    inProjectMenu: true,
    separator: true,
  },
  {
    group: "project",
    id: "project.close",
    labelKey: "command.project.close",
    inProjectMenu: true,
    separator: true,
  },
];

const tool = (id: string, key: string): Command => ({
  id: `tool.${id}`,
  labelKey: `command.tool.${id}` as TranslationKey,
  group: "tools",
  shortcut: key,
});

commands.push(
  {
    id: "sprite.open",
    labelKey: "command.sprite.open",
    group: "project",
    shortcut: "Enter",
  },
  {
    id: "sprite.rename",
    labelKey: "command.sprite.rename",
    group: "project",
    shortcut: "F2",
  },
  {
    id: "sprite.duplicate",
    labelKey: "command.sprite.duplicate",
    group: "project",
    shortcut: "Mod+D",
  },
  {
    id: "sprite.delete",
    labelKey: "command.sprite.delete",
    group: "project",
    shortcut: "Delete",
  },
  tool("pencil", "B"),
  tool("eraser", "E"),
  tool("bucket", "G"),
  tool("eyedropper", "I"),
  tool("line", "L"),
  tool("rectangle", "R"),
  tool("ellipse", "C"),
  tool("select", "M"),
  tool("move", "V"),
  {
    id: "edit.undo",
    labelKey: "command.edit.undo",
    group: "editor",
    shortcut: "Mod+Z",
  },
  {
    id: "edit.redo",
    labelKey: "command.edit.redo",
    group: "editor",
    shortcut: "Mod+Shift+Z",
  },
  {
    id: "frame.previous",
    labelKey: "command.frame.previous",
    group: "editor",
    shortcut: ",",
  },
  {
    id: "frame.next",
    labelKey: "command.frame.next",
    group: "editor",
    shortcut: ".",
  },
  {
    id: "view.zoomIn",
    labelKey: "command.view.zoomIn",
    group: "editor",
    shortcut: "+",
    withWheel: true,
  },
  {
    id: "view.zoomOut",
    labelKey: "command.view.zoomOut",
    group: "editor",
    shortcut: "-",
  },
  {
    id: "view.fit",
    labelKey: "command.view.fit",
    group: "editor",
    shortcut: "Mod+0",
  },
  {
    id: "view.pan",
    labelKey: "command.view.pan",
    group: "editor",
    shortcut: "Space",
  },
  {
    id: "view.onionSkin",
    labelKey: "command.view.onionSkin",
    group: "editor",
    shortcut: "O",
  },
);

export const projectMenuCommands = commands.filter((c) => c.inProjectMenu);
