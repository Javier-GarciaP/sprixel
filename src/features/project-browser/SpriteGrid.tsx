import type { MenuItem } from "@/shared/components";
import type { SpriteSummary } from "@/shared/lib/core";
import { useT } from "@/shared/lib/i18n";
import type { FrameCache } from "@/shared/lib/render";
import { BrowserIcon } from "./icons";
import { SpriteCard } from "./SpriteCard";
import { spriteKey } from "./paths";

type Props = {
  sprites: SpriteSummary[];
  cache: FrameCache;
  revision: number;
  selected: Set<string>;
  onSelect: (sprite: SpriteSummary) => void;
  onOpen: (sprite: SpriteSummary) => void;
  onNewSprite: () => void;
  // The "New sprite" card is not shown among search results.
  showNew: boolean;
  menuFor: (sprite: SpriteSummary) => MenuItem[];
};

// The sprites of a folder as cards, with a "New sprite" card at the end.
export function SpriteGrid({
  sprites,
  cache,
  revision,
  selected,
  onSelect,
  onOpen,
  onNewSprite,
  showNew,
  menuFor,
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
            onOpen={onOpen}
            menuItems={menuFor(sprite)}
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
