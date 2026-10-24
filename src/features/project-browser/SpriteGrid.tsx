import type { MenuItem } from "@/shared/components";
import type { SpriteSummary } from "@/shared/lib/core";
import { useT } from "@/shared/lib/i18n";
import type { FrameCache } from "@/shared/lib/render";
import { BrowserIcon } from "./icons";
import { SpriteCard, type Props as SpriteCardProps } from "./SpriteCard";
import { spriteKey } from "./paths";

type Props = {
  sprites: SpriteSummary[];
  cache: FrameCache;
  revision: number;
  selected: Set<string>;
  onSelect: SpriteCardProps["onSelect"];
  onToggle: (sprite: SpriteSummary) => void;
  onOpen: (sprite: SpriteSummary) => void;
  onContextMenu: SpriteCardProps["onContextMenu"];
  onNewSprite: () => void;
  // The "New sprite" card is not shown among search results.
  showNew: boolean;
  menuFor: (sprite: SpriteSummary) => MenuItem[];
  dragSpritesFor: (sprite: SpriteSummary) => SpriteSummary[];
};

// The sprites of a folder as cards, with a "New sprite" card at the end.
export function SpriteGrid({
  sprites,
  cache,
  revision,
  selected,
  onSelect,
  onToggle,
  onOpen,
  onContextMenu,
  onNewSprite,
  showNew,
  menuFor,
  dragSpritesFor,
}: Props) {
  const t = useT();
  return (
    <ul className="grid grid-cols-[repeat(auto-fill,minmax(150px,1fr))] gap-4">
      {sprites.map((sprite) => (
        <li key={spriteKey(sprite)}>
          <SpriteCard
            sprite={sprite}
            cache={cache}
            revision={revision}
            selected={selected.has(spriteKey(sprite))}
            onSelect={onSelect}
            onToggle={onToggle}
            onOpen={onOpen}
            onContextMenu={onContextMenu}
            menuItems={menuFor(sprite)}
            dragSprites={dragSpritesFor(sprite)}
          />
        </li>
      ))}
      {showNew && (
        <li>
          <button
            type="button"
            onClick={onNewSprite}
            className="flex h-full min-h-[190px] w-full flex-col items-center justify-center gap-2 rounded-lg border-2 border-dashed border-border text-sm text-text-muted hover:bg-surface"
          >
            <BrowserIcon name="plus" className="h-8 w-8" />
            {t("browse.newSprite")}
          </button>
        </li>
      )}
    </ul>
  );
}
