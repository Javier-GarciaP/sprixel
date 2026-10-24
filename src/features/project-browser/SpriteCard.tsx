import type { MenuItem } from "@/shared/components";
import type { SpriteSummary } from "@/shared/lib/core";
import { useT } from "@/shared/lib/i18n";
import type { FrameCache } from "@/shared/lib/render";
import { ItemMenu } from "./ItemMenu";
import { SpriteThumb } from "./SpriteThumb";

type Props = {
  sprite: SpriteSummary;
  cache: FrameCache;
  revision: number;
  selected: boolean;
  onSelect: (sprite: SpriteSummary) => void;
  onOpen: (sprite: SpriteSummary) => void;
  // The actions of the "..." button.
  menuItems: MenuItem[];
};

// A sprite of the selected folder: its thumbnail, name, size and frames.
// A click selects it; a double click opens it in the editor.
export function SpriteCard({
  sprite,
  cache,
  revision,
  selected,
  onSelect,
  onOpen,
  menuItems,
}: Props) {
  const t = useT();
  const frames = t(sprite.frames === 1 ? "browse.frameOne" : "browse.frames", {
    count: sprite.frames,
  });
  return (
    <article
      className={`relative rounded-lg border-2 bg-surface p-2 ${selected ? "border-amber" : "border-border"}`}
    >
      <button
        type="button"
        aria-pressed={selected}
        onClick={() => onSelect(sprite)}
        onDoubleClick={() => onOpen(sprite)}
        className="block w-full text-left"
      >
        <SpriteThumb
          sprite={sprite}
          cache={cache}
          revision={revision}
          box={120}
        />
        <span className="mt-2 block truncate text-sm font-bold">
          {sprite.name}
        </span>
        <span className="block font-mono text-xs text-text-muted">
          {sprite.width}x{sprite.height} - {frames}
        </span>
      </button>
      <ItemMenu
        label={t("browse.moreActions", { name: sprite.name })}
        items={menuItems}
        className="absolute right-3 top-3"
      />
    </article>
  );
}
