import type { MenuItem } from "@/shared/components";
import type { SpriteSummary } from "@/shared/lib/core";
import { useT } from "@/shared/lib/i18n";
import type { FrameCache } from "@/shared/lib/render";
import { ItemMenu } from "./ItemMenu";
import { DRAG_TYPE } from "./paths";
import { SpriteThumb } from "./SpriteThumb";

export type Props = {
  sprite: SpriteSummary;
  cache: FrameCache;
  revision: number;
  selected: boolean;
  // `extend` is true with Ctrl (or Command) held, `range` with Shift held.
  onSelect: (
    sprite: SpriteSummary,
    modifiers: { extend: boolean; range: boolean },
  ) => void;
  // The checkbox of the card adds or removes it from the selection.
  onToggle: (sprite: SpriteSummary) => void;
  onOpen: (sprite: SpriteSummary) => void;
  // A right click on the card, with the place of the pointer.
  onContextMenu: (sprite: SpriteSummary, at: { x: number; y: number }) => void;
  // The actions of the "..." button.
  menuItems: MenuItem[];
  // The sprites that move when this card is dragged: the whole selection when
  // the card is part of it.
  dragSprites: SpriteSummary[];
};

// A sprite of the selected folder: its thumbnail, name, size and frames.
// A click selects it; a double click opens it in the editor.
export function SpriteCard({
  sprite,
  cache,
  revision,
  selected,
  onSelect,
  onToggle,
  onOpen,
  onContextMenu,
  menuItems,
  dragSprites,
}: Props) {
  const t = useT();
  const frames = t(sprite.frames === 1 ? "browse.frameOne" : "browse.frames", {
    count: sprite.frames,
  });
  return (
    <article
      onContextMenu={(event) => {
        event.preventDefault();
        onContextMenu(sprite, { x: event.clientX, y: event.clientY });
      }}
      draggable
      onDragStart={(event) => {
        const sprites = dragSprites.map(({ folder, name }) => ({
          folder,
          name,
        }));
        event.dataTransfer.setData(DRAG_TYPE, JSON.stringify(sprites));
        event.dataTransfer.effectAllowed = "move";
      }}
      className={`relative rounded-lg border-2 bg-surface p-2 ${selected ? "border-amber" : "border-border"}`}
    >
      <button
        type="button"
        aria-pressed={selected}
        onClick={(event) =>
          onSelect(sprite, {
            extend: event.ctrlKey || event.metaKey,
            range: event.shiftKey,
          })
        }
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
      <label
        className={`absolute left-3 top-3 flex h-5 w-5 items-center justify-center rounded border border-border-strong bg-surface focus-within:opacity-100 hover:opacity-100 ${selected ? "opacity-100" : "opacity-0"}`}
      >
        <input
          type="checkbox"
          aria-label={t("browse.select", { name: sprite.name })}
          checked={selected}
          onChange={() => onToggle(sprite)}
          className="h-3.5 w-3.5 accent-[var(--color-amber)]"
        />
      </label>
      <ItemMenu
        label={t("browse.moreActions", { name: sprite.name })}
        items={menuItems}
        className="absolute right-3 top-3"
      />
    </article>
  );
}
