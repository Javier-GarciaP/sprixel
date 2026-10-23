import type { TranslationKey } from "@/shared/lib/i18n";
import { commands, type Group } from "./registry";
import { t } from "@/shared/lib/i18n";
import { shortcutKeys, type KeyName, type Platform } from "./shortcut";

export type ShortcutGroup = {
  titleKey: TranslationKey;
  rows: {
    id: string;
    labelKey: TranslationKey;
    keys: string[];
    // "+ or Ctrl wheel": the word and the keys after it.
    alternative?: { word: string; keys: string[] };
  }[];
};

const titles: Record<Group, TranslationKey> = {
  project: "shortcuts.group.project",
  tools: "shortcuts.group.tools",
  editor: "shortcuts.group.editor",
};

// The data of the shortcuts page: every command that has a shortcut, grouped
// by area, with the key caps for this platform.
export function shortcutGroups(
  platform: Platform,
  translate: KeyName = (key) => t("en", key),
): ShortcutGroup[] {
  return (Object.keys(titles) as Group[]).map((group) => ({
    titleKey: titles[group],
    rows: commands
      .filter((c) => c.group === group && c.shortcut)
      .map((c) => ({
        id: c.id,
        labelKey: c.labelKey,
        keys: shortcutKeys(c.shortcut as string, platform, translate),
        alternative: c.withWheel
          ? {
              word: translate("key.or"),
              keys: [
                ...shortcutKeys("Mod", platform, translate),
                translate("key.wheel"),
              ],
            }
          : undefined,
      })),
  }));
}
