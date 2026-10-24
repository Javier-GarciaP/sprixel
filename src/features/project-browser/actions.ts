import type { MenuItem } from "@/shared/components";
import type { SpriteSummary } from "@/shared/lib/core";
import type { TranslationKey } from "@/shared/lib/i18n";

// The shortcut of a command of the app, as text ("F2", "Ctrl+D"), or nothing.
export type HintFor = (commandId: string) => string | undefined;

type Translate = (
  key: TranslationKey,
  params?: Record<string, string | number>,
) => string;

// What a person can do with a sprite. The "..." button, the right-click menu
// and the keyboard all use this one list, so they cannot disagree.
export type SpriteActions = {
  open: (sprite: SpriteSummary) => void;
  rename: (sprite: SpriteSummary) => void;
  duplicate: (sprite: SpriteSummary) => void;
};

export function spriteMenuItems(
  sprite: SpriteSummary,
  actions: SpriteActions,
  t: Translate,
  hint: HintFor,
): MenuItem[] {
  return [
    {
      id: "open",
      label: t("command.sprite.open"),
      hint: hint("sprite.open"),
      onSelect: () => actions.open(sprite),
    },
    {
      id: "rename",
      label: t("command.sprite.rename"),
      hint: hint("sprite.rename"),
      onSelect: () => actions.rename(sprite),
    },
    {
      id: "duplicate",
      label: t("command.sprite.duplicate"),
      hint: hint("sprite.duplicate"),
      onSelect: () => actions.duplicate(sprite),
    },
  ];
}

// What a person can do with a folder.
export type FolderActions = {
  rename: (path: string[]) => void;
};

export function folderMenuItems(
  path: string[],
  actions: FolderActions,
  t: Translate,
  hint: HintFor,
): MenuItem[] {
  return [
    {
      id: "rename",
      label: t("command.sprite.rename"),
      hint: hint("sprite.rename"),
      onSelect: () => actions.rename(path),
    },
  ];
}
