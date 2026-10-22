import type { TranslationKey } from "@/shared/lib/i18n";
import { commands, type Group } from "./registry";
import { shortcutKeys, type Platform } from "./shortcut";

export type ShortcutGroup = {
  titleKey: TranslationKey;
  rows: { id: string; labelKey: TranslationKey; keys: string[] }[];
};

const titles: Record<Group, TranslationKey> = {
  project: "shortcuts.group.project",
  tools: "shortcuts.group.tools",
  editor: "shortcuts.group.editor",
};

// The data of the shortcuts page: every command that has a shortcut, grouped
// by area, with the key caps for this platform.
export function shortcutGroups(platform: Platform): ShortcutGroup[] {
  return (Object.keys(titles) as Group[]).map((group) => ({
    titleKey: titles[group],
    rows: commands
      .filter((c) => c.group === group && c.shortcut)
      .map((c) => ({
        id: c.id,
        labelKey: c.labelKey,
        keys: shortcutKeys(c.shortcut as string, platform),
      })),
  }));
}
