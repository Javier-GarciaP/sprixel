import { useState } from "react";
import { Button } from "@/shared/components";
import { spriteFrame, type SpriteSummary } from "@/shared/lib/core";
import { useT } from "@/shared/lib/i18n";
import { FrameCache } from "@/shared/lib/render";
import { FolderTree } from "./FolderTree";
import { PaletteList } from "./PaletteList";
import { DetailsPanel } from "./DetailsPanel";
import { SpriteGrid } from "./SpriteGrid";
import { allSprites, findFolder, pathKey, spriteKey } from "./paths";
import { useProjectTree } from "./useProjectTree";

export type SpriteTarget = { folder: string[]; name: string };

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
  const newSprite = () => {};

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
              <div className="flex items-baseline gap-2">
                {currentPath.length > 0 && (
                  <span className="text-sm text-text-muted">{tree.name} /</span>
                )}
                <h2 className="font-title text-xl font-bold">{crumb}</h2>
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
            <Button variant="primary" onClick={newSprite}>
              {t("browse.newSprite")}
            </Button>
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
    </div>
  );
}
