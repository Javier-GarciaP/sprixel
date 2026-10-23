import { describe, expect, it } from "vitest";
import { t } from "@/shared/lib/i18n";
import { matchesShortcut, shortcutKeys } from "./shortcut";
import { shortcutGroups } from "./shortcutGroups";

const press = (key: string, mods: Partial<KeyboardEvent> = {}) => ({
  key,
  ctrlKey: false,
  metaKey: false,
  shiftKey: false,
  altKey: false,
  ...mods,
});

describe("matchesShortcut", () => {
  // Checks the spec: Control is the main key on Windows and Linux, so Ctrl+Z
  // matches Undo and a plain Z does not.
  it("uses Control outside macOS", () => {
    expect(
      matchesShortcut(press("z", { ctrlKey: true }), "Mod+Z", "other"),
    ).toBe(true);
    expect(matchesShortcut(press("z"), "Mod+Z", "other")).toBe(false);
    expect(
      matchesShortcut(press("z", { metaKey: true }), "Mod+Z", "other"),
    ).toBe(false);
  });

  // Checks the spec: on macOS the Command key is used instead of Control.
  it("uses Command on macOS", () => {
    expect(matchesShortcut(press("z", { metaKey: true }), "Mod+Z", "mac")).toBe(
      true,
    );
    expect(matchesShortcut(press("z", { ctrlKey: true }), "Mod+Z", "mac")).toBe(
      false,
    );
  });

  // Checks that Redo needs Shift, and Undo does not match when Shift is held.
  it("tells Ctrl+Z and Ctrl+Shift+Z apart", () => {
    const redo = press("Z", { ctrlKey: true, shiftKey: true });
    expect(matchesShortcut(redo, "Mod+Shift+Z", "other")).toBe(true);
    expect(matchesShortcut(redo, "Mod+Z", "other")).toBe(false);
  });

  // Checks single keys (tools), named keys and the symbol keys.
  it("matches single and named keys", () => {
    expect(matchesShortcut(press("b"), "B", "other")).toBe(true);
    expect(matchesShortcut(press("b", { ctrlKey: true }), "B", "other")).toBe(
      false,
    );
    expect(matchesShortcut(press("F2"), "F2", "other")).toBe(true);
    expect(matchesShortcut(press(" "), "Space", "other")).toBe(true);
    expect(matchesShortcut(press("+", { shiftKey: true }), "+", "other")).toBe(
      true,
    );
    expect(matchesShortcut(press("-"), "-", "other")).toBe(true);
  });
});

describe("shortcutGroups", () => {
  // Checks the spec: the shortcuts page data has the three areas and lists the
  // main shortcuts, with Ctrl on Linux and Command on macOS.
  it("lists the shortcuts by area for the platform", () => {
    const [project, tools, editor] = shortcutGroups("other");
    expect(project.rows.map((r) => r.keys.join("+"))).toEqual([
      "Ctrl+N",
      "Ctrl+O",
      "Ctrl+E",
      "Enter",
      "F2",
      "Ctrl+D",
      "Delete",
    ]);
    expect(tools.rows.map((r) => r.keys.join(""))).toEqual([
      "B",
      "E",
      "G",
      "I",
      "L",
      "R",
      "C",
      "M",
      "V",
    ]);
    expect(editor.rows.find((r) => r.id === "edit.undo")?.keys).toEqual([
      "Ctrl",
      "Z",
    ]);

    const mac = shortcutGroups("mac")[2];
    expect(mac.rows.find((r) => r.id === "edit.undo")?.keys).toEqual([
      "⌘",
      "Z",
    ]);
  });
});

describe("translated key names", () => {
  const es = (key: Parameters<typeof t>[1]) => t("es", key);
  const en = (key: Parameters<typeof t>[1]) => t("en", key);

  // Checks that Spanish key caps use Intro, Supr, Mayús and Espacio.
  it("translates Enter, Delete, Shift and Space to Spanish", () => {
    expect(shortcutKeys("Enter", "other", es)).toEqual(["Intro"]);
    expect(shortcutKeys("Delete", "other", es)).toEqual(["Supr"]);
    expect(shortcutKeys("Mod+Shift+Z", "other", es)).toEqual([
      "Ctrl",
      "Mayús",
      "Z",
    ]);
    expect(shortcutKeys("Space", "other", es)).toEqual(["Espacio"]);
  });

  // Checks that English key caps keep Enter, Delete, Shift and Space, and
  // that this is also the default with no translate function.
  it("keeps the English names in English and by default", () => {
    expect(shortcutKeys("Enter", "other", en)).toEqual(["Enter"]);
    expect(shortcutKeys("Delete", "other", en)).toEqual(["Delete"]);
    expect(shortcutKeys("Mod+Shift+Z", "other")).toEqual([
      "Ctrl",
      "Shift",
      "Z",
    ]);
    expect(shortcutKeys("Space", "other", en)).toEqual(["Space"]);
  });

  // Checks that Ctrl and the macOS symbols are not translated.
  it("does not translate Ctrl or the macOS symbols", () => {
    expect(shortcutKeys("Mod+N", "other", es)).toEqual(["Ctrl", "N"]);
    expect(shortcutKeys("Mod+Shift+Z", "mac", es)).toEqual(["⌘", "⇧", "Z"]);
  });
});
