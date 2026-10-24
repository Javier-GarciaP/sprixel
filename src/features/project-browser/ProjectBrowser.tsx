import { useState } from "react";
import { Button } from "@/shared/components";
import {
  createSprite,
  spriteFrame,
  type SpriteSummary,
  type SpriteTarget,
} from "@/shared/lib/core";
import { useT } from "@/shared/lib/i18n";
import { FrameCache } from "@/shared/lib/render";
import { FolderTree } from "./FolderTree";
import { NewSpriteDialog } from "./NewSpriteDialog";
import { PaletteList } from "./PaletteList";
import { DetailsPanel } from "./DetailsPanel";
import { SpriteGrid } from "./SpriteGrid";
import {
  allSprites,
  findFolder,
  folderOptions,
  pathKey,
  spriteKey,
} from "./paths";
import { useProjectTree } from "./useProjectTree";

export type { SpriteTarget };

type Props = {
  onOpenSprite: (sprite: SpriteTarget) => void;
  onOpenPalette: (name: string) => void;
  onNewPalette: () => void;
};

// The Project tab: folders on the left, sprites in the middle and the
// details on the right.
export function ProjectBrowser({
  onOpenSprite,
  onOpenPalette,
  onNewPalette,
}: Props) {
  const t = useT();
  const [cache] = useState(() => new FrameCache(spriteFrame));
  const tree = useProjectTree(cache);
  const [folderPath, setFolderPath] = useState<string[]>([]);
  const [expanded, setExpanded] = useState<Set<string>>(new Set());
  const [selected, setSelected] = useState<SpriteSummary[]>([]);
  const [dialog, setDialog] = useState<"sprite" | null>(null);

  if (!tree) return null;

  // A folder that was deleted or renamed is gone: show the root instead.
  const folder = findFolder(tree, folderPath);
  const currentPath = folder ? folderPath : [];

  const selectFolder = (path: string[]) => {
    setFolderPath(path);
    setSelected([]);
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
  const selectSprite = (sprite: SpriteSummary) => {
    setFolderPath(sprite.folder);
    setSelected([sprite]);
  };

  const sprites = folder ? folder.sprites : tree.root.sprites;
  const hasSprites = allSprites(tree.root).length > 0;
  const crumb = currentPath[currentPath.length - 1] ?? tree.name;
  const newSprite = () => setDialog("sprite");

  return (
    <div className="flex h-full">
      <aside className="flex w-64 shrink-0 flex-col gap-4 overflow-auto border-r border-border p-3">
        <FolderTree
          tree={tree}
          selectedFolder={currentPath}
          selectedSprites={new Set(selected.map(spriteKey))}
          expanded={expanded}
          onSelectFolder={selectFolder}
          onToggle={toggle}
          onSelectSprite={selectSprite}
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
                  <h2 className="font-title text-xl font-bold">{crumb}</h2>
                </div>
                <div className="flex gap-2">
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
              selected={new Set(selected.map(spriteKey))}
              onSelect={selectSprite}
              onOpen={onOpenSprite}
              onNewSprite={newSprite}
            />
          </>
        ) : (
          <div className="flex h-full flex-col items-center justify-center gap-3 text-center">
            <h2 className="font-title text-xl font-bold">
              {t("browse.empty.title")}
            </h2>
            <p className="text-sm text-text-muted">{t("browse.empty.hint")}</p>
            <div className="flex gap-2">
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
            onOpen={onOpenSprite}
          />
        </aside>
      )}
      {dialog === "sprite" && (
        <NewSpriteDialog
          folders={folderOptions(tree)}
          palettes={tree.palettes}
          defaultFolder={currentPath}
          create={createSprite}
          onCreated={onOpenSprite}
          onClose={() => setDialog(null)}
        />
      )}
    </div>
  );
}
