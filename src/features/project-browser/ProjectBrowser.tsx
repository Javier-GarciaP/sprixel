import { useEffect, useState } from "react";
import { Button, Toast } from "@/shared/components";
import {
  asCoreFailure,
  createFolder,
  createSprite,
  duplicateSprite,
  renameFolder,
  renameSprite,
  spriteFrame,
  type ProjectTree,
  type SpriteSummary,
  type SpriteTarget,
} from "@/shared/lib/core";
import { useT } from "@/shared/lib/i18n";
import { FrameCache } from "@/shared/lib/render";
import { folderMenuItems, spriteMenuItems, type HintFor } from "./actions";
import { DetailsPanel } from "./DetailsPanel";
import { FolderTree } from "./FolderTree";
import { NewFolderDialog } from "./NewFolderDialog";
import { NewSpriteDialog } from "./NewSpriteDialog";
import { PaletteList } from "./PaletteList";
import { RenameDialog } from "./RenameDialog";
import { SpriteGrid } from "./SpriteGrid";
import {
  allSprites,
  findFolder,
  folderOptions,
  followRename,
  pathKey,
  spriteKey,
} from "./paths";
import { useProjectTree } from "./useProjectTree";

export type { SpriteTarget };

// How the app lets the Project tab use the keyboard. The app owns the list of
// shortcuts (one place for the shortcuts page, the menus and the keys), so it
// gives this feature a hook that listens to them and a way to name a shortcut.
export type Shortcuts = {
  useHandlers: (handlers: Record<string, () => void>) => void;
  hint: HintFor;
};

const noShortcuts: Shortcuts = { useHandlers: () => {}, hint: () => undefined };

type Props = {
  onOpenSprite: (sprite: SpriteTarget) => void;
  onOpenPalette: (name: string) => void;
  onNewPalette: () => void;
  shortcuts?: Shortcuts;
};

type Dialog =
  | { kind: "sprite" }
  | { kind: "folder" }
  | { kind: "rename-sprite"; sprite: SpriteTarget }
  | { kind: "rename-folder"; path: string[] };

// The Project tab: folders on the left, sprites in the middle and the
// details on the right.
export function ProjectBrowser(props: Props) {
  const [cache] = useState(() => new FrameCache(spriteFrame));
  const tree = useProjectTree(cache);
  if (!tree) return null;
  return <Browser {...props} tree={tree} cache={cache} />;
}

function Browser({
  tree,
  cache,
  onOpenSprite,
  onOpenPalette,
  onNewPalette,
  shortcuts = noShortcuts,
}: Props & { tree: ProjectTree; cache: FrameCache }) {
  const t = useT();
  const [folderPath, setFolderPath] = useState<string[]>([]);
  const [expanded, setExpanded] = useState<Set<string>>(new Set());
  // The selection is kept as keys and looked up in the tree each time, so the
  // details always show what the core has now.
  const [selectedKeys, setSelectedKeys] = useState<string[]>([]);
  const [dialog, setDialog] = useState<Dialog | null>(null);
  const [notice, setNotice] = useState<string | null>(null);

  useEffect(() => {
    if (!notice) return;
    const timer = setTimeout(() => setNotice(null), 6000);
    return () => clearTimeout(timer);
  }, [notice]);

  const everySprite = allSprites(tree.root);
  const selected = selectedKeys
    .map((key) => everySprite.find((s) => spriteKey(s) === key))
    .filter((sprite): sprite is SpriteSummary => sprite !== undefined);

  // A folder that was deleted or renamed is gone: show the root instead.
  const folder = findFolder(tree, folderPath);
  const currentPath = folder ? folderPath : [];
  const sprites = folder ? folder.sprites : tree.root.sprites;
  const hasSprites = everySprite.length > 0;
  const crumb = currentPath[currentPath.length - 1] ?? tree.name;

  const fail = (error: unknown) =>
    setNotice(t("error.io", { message: asCoreFailure(error).message }));

  const selectFolder = (path: string[]) => {
    setFolderPath(path);
    setSelectedKeys([]);
    setExpanded((open) => new Set(open).add(pathKey(path)));
  };
  const toggle = (path: string[]) =>
    setExpanded((open) => {
      const next = new Set(open);
      const key = pathKey(path);
      if (next.has(key)) next.delete(key);
      else next.add(key);
      return next;
    });
  const selectSprite = (sprite: Pick<SpriteSummary, "folder" | "name">) => {
    setFolderPath(sprite.folder);
    setSelectedKeys([spriteKey(sprite)]);
  };

  // The actions on sprites and folders. Each menu, button and key calls these.
  const open = (sprite: SpriteTarget) => onOpenSprite(sprite);
  const askRename = (sprite: SpriteTarget) =>
    setDialog({ kind: "rename-sprite", sprite });
  const askRenameFolder = (path: string[]) =>
    setDialog({ kind: "rename-folder", path });
  const duplicate = (sprite: SpriteTarget) =>
    duplicateSprite(sprite.folder, sprite.name).then(selectSprite, fail);

  const doRenameSprite = async (sprite: SpriteTarget, newName: string) => {
    const renamed = await renameSprite(sprite.folder, sprite.name, newName);
    selectSprite(renamed);
  };
  const doRenameFolder = async (path: string[], newName: string) => {
    const renamed = await renameFolder(path, newName);
    // What was inside the folder now has the new path.
    setFolderPath((now) => followRename(now, path, renamed));
    setSelectedKeys((keys) =>
      keys.map((key) => pathKey(followRename(key.split("/"), path, renamed))),
    );
    setExpanded(
      (open) =>
        new Set(
          [...open].map((key) =>
            pathKey(followRename(key.split("/"), path, renamed)),
          ),
        ),
    );
  };

  const { hint } = shortcuts;
  const menuFor = (sprite: SpriteSummary) =>
    spriteMenuItems(sprite, { open, rename: askRename, duplicate }, t, hint);
  const folderMenu = (path: string[]) =>
    folderMenuItems(path, { rename: askRenameFolder }, t, hint);

  // Keys: F2 renames the selected sprite, or the open folder when no sprite is
  // selected. They do nothing while a dialog is open.
  const only = selected.length === 1 ? selected[0] : null;
  shortcuts.useHandlers(
    dialog
      ? {}
      : {
          ...(only
            ? {
                "sprite.open": () => open(only),
                "sprite.rename": () => askRename(only),
                "sprite.duplicate": () => duplicate(only),
              }
            : currentPath.length > 0
              ? { "sprite.rename": () => askRenameFolder(currentPath) }
              : {}),
        },
  );

  const newSprite = () => setDialog({ kind: "sprite" });

  return (
    <div className="flex h-full">
      <aside className="flex w-64 shrink-0 flex-col gap-4 overflow-auto border-r border-border p-3">
        <FolderTree
          tree={tree}
          selectedFolder={currentPath}
          selectedSprites={new Set(selectedKeys)}
          expanded={expanded}
          onSelectFolder={selectFolder}
          onToggle={toggle}
          onSelectSprite={selectSprite}
          onRenameFolder={askRenameFolder}
          onRenameSprite={askRename}
          folderMenu={folderMenu}
        />
        <PaletteList
          palettes={tree.palettes}
          onOpen={onOpenPalette}
          onNew={onNewPalette}
        />
      </aside>
      <section
        aria-label={t("browse.sprites")}
        className="min-w-0 flex-1 overflow-auto p-6"
      >
        {hasSprites ? (
          <>
            <header className="mb-4">
              <div className="flex items-center justify-between gap-4">
                <div className="flex items-baseline gap-2">
                  {currentPath.length > 0 && (
                    <span className="text-sm text-text-muted">
                      {tree.name} /
                    </span>
                  )}
                  <h2
                    className="font-title text-xl font-bold"
                    onDoubleClick={() =>
                      currentPath.length > 0 && askRenameFolder(currentPath)
                    }
                  >
                    {crumb}
                  </h2>
                </div>
                <div className="flex gap-2">
                  <Button onClick={() => setDialog({ kind: "folder" })}>
                    {t("browse.newFolder")}
                  </Button>
                  <Button variant="primary" onClick={newSprite}>
                    {t("browse.newSprite")}
                  </Button>
                </div>
              </div>
              <p className="mt-3 text-xs font-semibold text-text-muted">
                {t("browse.spritesCount", { count: sprites.length })}
              </p>
            </header>
            <SpriteGrid
              sprites={sprites}
              cache={cache}
              revision={tree.revision}
              selected={new Set(selectedKeys)}
              onSelect={selectSprite}
              onOpen={open}
              onNewSprite={newSprite}
              menuFor={menuFor}
            />
          </>
        ) : (
          <div className="flex h-full flex-col items-center justify-center gap-3 text-center">
            <h2 className="font-title text-xl font-bold">
              {t("browse.empty.title")}
            </h2>
            <p className="text-sm text-text-muted">{t("browse.empty.hint")}</p>
            <div className="flex gap-2">
              <Button onClick={() => setDialog({ kind: "folder" })}>
                {t("browse.newFolder")}
              </Button>
              <Button variant="primary" onClick={newSprite}>
                {t("browse.newSprite")}
              </Button>
            </div>
          </div>
        )}
      </section>
      {hasSprites && (
        <aside
          aria-label={t("browse.details")}
          className="w-72 shrink-0 overflow-auto border-l border-border p-4"
        >
          <DetailsPanel
            sprite={selected[0] ?? null}
            cache={cache}
            revision={tree.revision}
            onOpen={open}
            onRename={askRename}
            onDuplicate={duplicate}
          />
        </aside>
      )}
      {dialog?.kind === "sprite" && (
        <NewSpriteDialog
          folders={folderOptions(tree)}
          palettes={tree.palettes}
          defaultFolder={currentPath}
          create={createSprite}
          onCreated={onOpenSprite}
          onClose={() => setDialog(null)}
        />
      )}
      {dialog?.kind === "folder" && (
        <NewFolderDialog
          folders={folderOptions(tree)}
          defaultParent={currentPath}
          create={createFolder}
          onClose={() => setDialog(null)}
        />
      )}
      {dialog?.kind === "rename-sprite" && (
        <RenameDialog
          kind="sprite"
          parent={dialog.sprite.folder}
          name={dialog.sprite.name}
          rename={(newName) => doRenameSprite(dialog.sprite, newName)}
          onClose={() => setDialog(null)}
        />
      )}
      {dialog?.kind === "rename-folder" && (
        <RenameDialog
          kind="folder"
          parent={dialog.path.slice(0, -1)}
          name={dialog.path[dialog.path.length - 1]}
          rename={(newName) => doRenameFolder(dialog.path, newName)}
          onClose={() => setDialog(null)}
        />
      )}
      {notice && <Toast tone="error" message={notice} />}
    </div>
  );
}
