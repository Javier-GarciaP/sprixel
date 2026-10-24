import type { MenuItem } from "@/shared/components";
import type { SpriteSummary } from "@/shared/lib/core";
import type { TranslationKey } from "@/shared/lib/i18n";
import { pathKey, type FolderOption } from "./paths";

// The shortcut of a command of the app, as text ("F2", "Ctrl+D"), or nothing.
export type HintFor = (commandId: string) => string | undefined;

type Translate = (
  key: TranslationKey,
  params?: Record<string, string | number>,
) => string;

// What a person can do with sprites. The "..." button, the right-click menu
// and the keyboard all use this one list, so they cannot disagree. The actions
// get the sprites they apply to: the whole selection when the sprite is part
// of it, or only that sprite.
export type SpriteActions = {
  open: (sprite: SpriteSummary) => void;
  rename: (sprite: SpriteSummary) => void;
  duplicate: (sprite: SpriteSummary) => void;
  move: (sprites: SpriteSummary[], to: string[]) => void;
  moveToNewFolder: (sprites: SpriteSummary[]) => void;
  delete: (sprites: SpriteSummary[]) => void;
};

type Context = {
  actions: SpriteActions;
  t: Translate;
  hint: HintFor;
  // Every folder of the project, for "Move to folder".
  folders: FolderOption[];
};

// The places the sprites can move to: every folder where they are not all
// already, and "New folder...".
export function moveMenuItems(
  targets: SpriteSummary[],
  { actions, t, folders }: Pick<Context, "actions" | "t" | "folders">,
): MenuItem[] {
  const here = (path: string[]) =>
    targets.every((s) => pathKey(s.folder) === pathKey(path));
  const destinations: MenuItem[] = folders
    .filter((folder) => !here(folder.path))
    .map((folder) => ({
      id: `move:${pathKey(folder.path)}`,
      label: folder.label,
      onSelect: () => actions.move(targets, folder.path),
    }));
  return [
    ...destinations,
    {
      id: "move:new",
      label: t("move.newFolder"),
      separator: destinations.length > 0,
      onSelect: () => actions.moveToNewFolder(targets),
    },
  ];
}

// The menu of a sprite. `targets` are the sprites the actions apply to (more
// than one when several are selected); `sprite` is the one the menu opened on.
export function spriteMenuItems(
  sprite: SpriteSummary,
  targets: SpriteSummary[],
  context: Context,
): MenuItem[] {
  const { actions, t, hint } = context;
  const many = targets.length > 1;
  return [
    // Open, rename and duplicate work on one sprite.
    ...(many
      ? []
      : [
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
        ]),
    {
      id: "move",
      label: t("command.sprite.move"),
      onSelect: () => {},
      submenu: moveMenuItems(targets, context),
    },
    {
      id: "delete",
      label: many
        ? t("command.sprite.deleteMany", { count: targets.length })
        : t("command.sprite.delete"),
      hint: hint("sprite.delete"),
      separator: true,
      onSelect: () => actions.delete(targets),
    },
  ];
}

// What a person can do with a folder.
export type FolderActions = {
  rename: (path: string[]) => void;
  delete: (path: string[]) => void;
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
    {
      id: "delete",
      label: t("command.folder.delete"),
      hint: hint("sprite.delete"),
      separator: true,
      onSelect: () => actions.delete(path),
    },
  ];
}
