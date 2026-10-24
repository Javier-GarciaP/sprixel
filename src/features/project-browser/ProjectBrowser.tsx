import { useEffect, useState } from "react";
import { Button, Toast } from "@/shared/components";
import {
  asCoreFailure,
  createFolder,
  createSprite,
  deleteFolder,
  deleteSprites,
  duplicateSprite,
  moveSprites,
  projectRedo,
  projectUndo,
  renameFolder,
  renameSprite,
  spriteFrame,
  type FolderNode,
  type ProjectTree,
  type SpriteSummary,
  type SpriteTarget,
} from "@/shared/lib/core";
import { useT } from "@/shared/lib/i18n";
import { FrameCache } from "@/shared/lib/render";
import { folderMenuItems, spriteMenuItems, type HintFor } from "./actions";
import { ConfirmDeleteDialog } from "./ConfirmDeleteDialog";
import { DetailsPanel } from "./DetailsPanel";
import { FolderTree } from "./FolderTree";
import { NewFolderDialog } from "./NewFolderDialog";
import { NewSpriteDialog } from "./NewSpriteDialog";
import { PaletteList } from "./PaletteList";
import { BrowserIcon } from "./icons";
import { RenameDialog } from "./RenameDialog";
import { SearchResultsList } from "./SearchResultsList";
import { search } from "./search";
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

// How long the message after a delete offers Undo.
export const UNDO_MS = 8000;

type Dialog =
  | { kind: "sprite" }
  // `thenMove`: the sprites to move into the new folder (from "Move to folder").
  | { kind: "folder"; thenMove?: SpriteSummary[] }
  | { kind: "rename-sprite"; sprite: SpriteTarget }
  | { kind: "rename-folder"; path: string[] }
  | { kind: "delete-folder"; path: string[] }
  | { kind: "delete-sprites"; sprites: SpriteSummary[] };

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
  const [query, setQuery] = useState("");
  const [notice, setNotice] = useState<string | null>(null);
  // The message after a delete. `revision` is the project revision right after
  // the delete: if anything else changes, Undo would undo that instead.
  const [trashed, setTrashed] = useState<{
    message: string;
    revision: number;
  } | null>(null);

  useEffect(() => {
    if (!notice) return;
    const timer = setTimeout(() => setNotice(null), 6000);
    return () => clearTimeout(timer);
  }, [notice]);

  useEffect(() => {
    if (!trashed) return;
    const timer = setTimeout(() => setTrashed(null), UNDO_MS);
    return () => clearTimeout(timer);
  }, [trashed]);

  useEffect(() => {
    if (trashed && tree.revision > trashed.revision) setTrashed(null);
  }, [tree.revision, trashed]);

  const everySprite = allSprites(tree.root);
  const selected = selectedKeys
    .map((key) => everySprite.find((s) => spriteKey(s) === key))
    .filter((sprite): sprite is SpriteSummary => sprite !== undefined);

  // A folder that was deleted or renamed is gone: show the root instead.
  const folder = findFolder(tree, folderPath);
  const currentPath = folder ? folderPath : [];
  // While the person searches, the middle area lists what matches in the
  // whole project instead of the sprites of the open folder.
  const searching = query.trim() !== "";
  const results = search(tree, query);
  const sprites = searching
    ? results.sprites
    : folder
      ? folder.sprites
      : tree.root.sprites;
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

  // Sends sprites to the trash and offers Undo. One sprite goes at once;
  // several ask first, because more is at stake.
  const trash = (sprites: SpriteSummary[]) => {
    if (sprites.length > 1) setDialog({ kind: "delete-sprites", sprites });
    else if (sprites.length === 1) trashNow(sprites);
  };
  const trashNow = async (sprites: SpriteSummary[]) => {
    try {
      const revision = await deleteSprites(
        sprites.map(({ folder, name }) => ({ folder, name })),
      );
      setSelectedKeys([]);
      setTrashed({
        message:
          sprites.length === 1
            ? t("trash.moved", { name: sprites[0].name })
            : t("trash.groupMoved", { count: sprites.length }),
        revision,
      });
    } catch (error) {
      fail(error);
    }
  };
  const trashFolder = async (path: string[]) => {
    try {
      const revision = await deleteFolder(path);
      setTrashed({
        message: t("trash.folderMoved", { name: path[path.length - 1] }),
        revision,
      });
    } catch (error) {
      fail(error);
    }
  };
  // Undo the last change of the project (a delete, usually). A message of the
  // core that there is nothing to undo is not an error for the person.
  const undo = (run: () => Promise<void>) => () => {
    setTrashed(null);
    run().catch((error) => {
      const { code } = asCoreFailure(error);
      if (code !== "nothing_to_undo" && code !== "nothing_to_redo") fail(error);
    });
  };

  const move = async (
    sprites: Pick<SpriteSummary, "folder" | "name">[],
    to: string[],
  ) => {
    try {
      await moveSprites(
        sprites.map(({ folder, name }) => ({ folder, name })),
        to,
      );
      setSelectedKeys([]);
    } catch (error) {
      if (asCoreFailure(error).code === "name_exists") {
        const label = folderOptions(tree).find(
          (option) => pathKey(option.path) === pathKey(to),
        )?.label;
        setNotice(t("move.conflict", { folder: label ?? "" }));
      } else {
        fail(error);
      }
    }
  };
  // Moving into a folder that is made right now: create it, then move.
  const createFolderAndMove = async (
    sprites: SpriteSummary[],
    parent: string[],
    name: string,
  ) => {
    const path = await createFolder(parent, name);
    await move(sprites, path);
  };

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
  const folders = folderOptions(tree);
  const spriteActions = {
    open,
    rename: askRename,
    duplicate,
    move,
    moveToNewFolder: (sprites: SpriteSummary[]) =>
      setDialog({ kind: "folder", thenMove: sprites }),
    delete: trash,
  };
  // A card acts on the whole selection when it is part of it (a menu, a drag),
  // and only on itself otherwise.
  const targetsFor = (sprite: SpriteSummary) =>
    selected.length > 1 &&
    selected.some((s) => spriteKey(s) === spriteKey(sprite))
      ? selected
      : [sprite];
  const menuFor = (sprite: SpriteSummary) =>
    spriteMenuItems(sprite, targetsFor(sprite), {
      actions: spriteActions,
      t,
      hint,
      folders,
    });
  // Dropping on the folder a sprite is already in does nothing.
  const dropSprites = (sprites: SpriteTarget[], to: string[]) => {
    const moving = sprites.filter((s) => pathKey(s.folder) !== pathKey(to));
    if (moving.length > 0) move(moving, to);
  };
  const folderMenu = (path: string[]) =>
    folderMenuItems(
      path,
      {
        rename: askRenameFolder,
        delete: (folderPath) =>
          setDialog({ kind: "delete-folder", path: folderPath }),
      },
      t,
      hint,
    );

  // Keys: F2 renames the selected sprite, or the open folder when no sprite is
  // selected. They do nothing while a dialog is open.
  const only = selected.length === 1 ? selected[0] : null;
  shortcuts.useHandlers(
    dialog
      ? {}
      : {
          "edit.undo": undo(projectUndo),
          "edit.redo": undo(projectRedo),
          ...(only
            ? {
                "sprite.open": () => open(only),
                "sprite.rename": () => askRename(only),
                "sprite.duplicate": () => duplicate(only),
                "sprite.delete": () => trash([only]),
              }
            : currentPath.length > 0
              ? {
                  "sprite.rename": () => askRenameFolder(currentPath),
                  "sprite.delete": () =>
                    setDialog({ kind: "delete-folder", path: currentPath }),
                }
              : {}),
        },
  );

  const newSprite = () => setDialog({ kind: "sprite" });

  return (
    <div className="flex h-full">
      <aside className="flex w-64 shrink-0 flex-col gap-4 overflow-auto border-r border-border p-3">
        <div className="relative">
          <BrowserIcon
            name="search"
            className="pointer-events-none absolute left-3 top-2.5 h-4 w-4 text-text-faint"
          />
          <input
            type="search"
            aria-label={t("browse.search")}
            placeholder={t("browse.search")}
            value={query}
            onChange={(event) => setQuery(event.target.value)}
            onKeyDown={(event) => event.key === "Escape" && setQuery("")}
            className="w-full rounded-md border border-border bg-bg py-2 pl-9 pr-8 text-sm text-text"
          />
          {searching && (
            <button
              type="button"
              aria-label={t("browse.clearSearch")}
              onClick={() => setQuery("")}
              className="absolute right-2 top-2 text-text-muted"
            >
              <BrowserIcon name="close" className="h-4 w-4" />
            </button>
          )}
        </div>
        {searching ? (
          <SearchResultsList
            results={results}
            selectedSprites={new Set(selectedKeys)}
            onSelectFolder={(folder) => {
              setQuery("");
              selectFolder(folder.path);
            }}
            onSelectSprite={selectSprite}
          />
        ) : (
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
            onDropSprites={dropSprites}
          />
        )}
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
                  {searching ? (
                    <h2 className="font-title text-xl font-bold">
                      {t("browse.resultsFor", { query: query.trim() })}
                    </h2>
                  ) : (
                    <>
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
                    </>
                  )}
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
              showNew={!searching}
              menuFor={menuFor}
              dragSpritesFor={targetsFor}
            />
            {!searching && sprites.length === 0 && (
              <p className="mb-4 text-sm text-text-muted">
                {t("browse.folderEmpty")}
              </p>
            )}
            {searching && sprites.length === 0 && (
              <p className="text-sm text-text-muted">
                {t("browse.noResultsMain", { query: query.trim() })}
              </p>
            )}
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
            onDelete={(sprite) => trash([sprite])}
          />
        </aside>
      )}
      {dialog?.kind === "sprite" && (
        <NewSpriteDialog
          folders={folders}
          palettes={tree.palettes}
          defaultFolder={currentPath}
          create={createSprite}
          onCreated={onOpenSprite}
          onClose={() => setDialog(null)}
        />
      )}
      {dialog?.kind === "folder" && (
        <NewFolderDialog
          folders={folders}
          defaultParent={currentPath}
          create={(parent, name) =>
            dialog.thenMove
              ? createFolderAndMove(dialog.thenMove, parent, name)
              : createFolder(parent, name)
          }
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
      {dialog?.kind === "delete-folder" && (
        <ConfirmFolderDelete
          folder={findFolder(tree, dialog.path)}
          path={dialog.path}
          onConfirm={() => {
            setDialog(null);
            trashFolder(dialog.path);
          }}
          onClose={() => setDialog(null)}
        />
      )}
      {dialog?.kind === "delete-sprites" && (
        <ConfirmDeleteDialog
          title={t("confirm.groupTitle", { count: dialog.sprites.length })}
          message={t("confirm.groupBody", { count: dialog.sprites.length })}
          items={dialog.sprites.map((s) => [...s.folder, s.name].join(" / "))}
          confirmLabel={t("confirm.deleteGroup", {
            count: dialog.sprites.length,
          })}
          onConfirm={() => {
            setDialog(null);
            trashNow(dialog.sprites);
          }}
          onClose={() => setDialog(null)}
        />
      )}
      {trashed ? (
        <Toast
          message={trashed.message}
          actionLabel={t("trash.undo")}
          onAction={undo(projectUndo)}
        />
      ) : (
        notice && <Toast tone="error" message={notice} />
      )}
    </div>
  );
}

// The question before a folder goes to the trash: what is inside it.
function ConfirmFolderDelete({
  folder,
  path,
  onConfirm,
  onClose,
}: {
  folder: FolderNode | null;
  path: string[];
  onConfirm: () => void;
  onClose: () => void;
}) {
  const t = useT();
  const inside = folder ? allSprites(folder) : [];
  const message =
    inside.length === 0
      ? t("confirm.folderBodyEmpty")
      : inside.length === 1
        ? t("confirm.folderBodyOne")
        : t("confirm.folderBody", { count: inside.length });
  return (
    <ConfirmDeleteDialog
      title={t("confirm.folderTitle", { name: path[path.length - 1] })}
      message={message}
      items={inside.map((s) =>
        [...s.folder.slice(path.length), s.name].join(" / "),
      )}
      confirmLabel={t("confirm.delete")}
      onConfirm={onConfirm}
      onClose={onClose}
    />
  );
}
