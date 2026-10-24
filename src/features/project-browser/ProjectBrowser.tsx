import { useState } from "react";
import { spriteFrame, type SpriteSummary } from "@/shared/lib/core";
import { useT } from "@/shared/lib/i18n";
import { FrameCache } from "@/shared/lib/render";
import { FolderTree } from "./FolderTree";
import { PaletteList } from "./PaletteList";
import { findFolder, pathKey, spriteKey } from "./paths";
import { useProjectTree } from "./useProjectTree";

export type SpriteTarget = { folder: string[]; name: string };

type Props = {
  onOpenSprite: (sprite: SpriteTarget) => void;
  onOpenPalette: (name: string) => void;
  onNewPalette: () => void;
};

// The Project tab: folders on the left, sprites in the middle and the
// details on the right.
export function ProjectBrowser({ onOpenPalette, onNewPalette }: Props) {
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
        <h2 className="font-title text-xl font-bold">
          {currentPath.length ? currentPath[currentPath.length - 1] : tree.name}
        </h2>
      </section>
    </div>
  );
}
