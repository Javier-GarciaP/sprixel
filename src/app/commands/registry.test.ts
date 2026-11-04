import { describe, expect, it } from "vitest";
import { formatShortcut } from "./shortcut";
import { commands, projectMenuCommands } from "./registry";

describe("command registry", () => {
  // Checks that every command id is unique, so a shortcut or a menu click
  // can never run the wrong command.
  it("has unique ids", () => {
    const ids = commands.map((c) => c.id);
    expect(new Set(ids).size).toBe(ids.length);
  });

  // Checks the spec: the project menu has exactly the listed commands and no
  // Save command, because the project saves by itself.
  it("has the project menu commands and no Save", () => {
    expect(projectMenuCommands.map((c) => c.id)).toEqual([
      "project.new",
      "project.open",
      "project.openRecent",
      "project.rename",
      "project.saveCopy",
      "project.showInFolder",
      "project.export",
      "project.close",
    ]);
    expect(commands.some((c) => /\.save$/i.test(c.id))).toBe(false);
  });
});

describe("formatShortcut", () => {
  // Checks that Mod shows as Ctrl on Windows and Linux.
  it("uses Ctrl outside macOS", () => {
    expect(formatShortcut("Mod+Shift+Z", "other")).toBe("Ctrl+Shift+Z");
    expect(formatShortcut("F2", "other")).toBe("F2");
  });

  // Checks that Mod shows as the Command symbol on macOS.
  it("uses Command on macOS", () => {
    expect(formatShortcut("Mod+N", "mac")).toBe("⌘N");
    expect(formatShortcut("Mod+Shift+Z", "mac")).toBe("⌘⇧Z");
  });
});

describe("editor shortcuts", () => {
  // Checks that no two commands share a shortcut, so a key press can never
  // run two commands (the power tools added many keys).
  it("has no shortcut used twice", () => {
    const keys = commands.flatMap((c) => (c.shortcut ? [c.shortcut] : []));
    expect(keys.filter((key, i) => keys.indexOf(key) !== i)).toEqual([]);
  });

  // Checks that the selection commands exist with the planned default keys.
  it("has the selection commands with their default keys", () => {
    const key = (id: string) => commands.find((c) => c.id === id)?.shortcut;
    expect(key("selection.flipHorizontal")).toBe("H");
    expect(key("selection.flipVertical")).toBe("Shift+H");
    expect(key("selection.moveRight")).toBe("ArrowRight");
    expect(key("selection.mirrorRightToLeft")).toBeUndefined();
  });

  // Checks that arrow keys are shown as arrows on the shortcuts page.
  it("shows arrow keys as arrows", () => {
    expect(formatShortcut("ArrowLeft", "other")).toBe("←");
  });
});
