import { Icon } from "@/shared/components";
import { useT, type TranslationKey } from "@/shared/lib/i18n";
import type { Group } from "./groups";
import type { RepeatMode } from "./types";

// How an animation ends, drawn on a 24x24 grid like the shared icons.
export const modePaths: Record<RepeatMode, string> = {
  loop: "M17 2l4 4-4 4M3 11V9a3 3 0 013-3h15M7 22l-4-4 4-4M21 13v2a3 3 0 01-3 3H3",
  once: "M5 12h14M13 6l6 6-6 6",
  pingpong: "M7 7l-4 5 4 5M17 7l4 5-4 5M3 12h18",
};

export const modeKeys: Record<RepeatMode, TranslationKey> = {
  loop: "anim.modeLoop",
  once: "anim.modeOnce",
  pingpong: "anim.modePingpong",
};

type Props = {
  group: Group;
  // True for the group that is chosen: it is open and shows its speed.
  selected: boolean;
  onSelect: () => void;
  onPlay: () => void;
  onMenu: (box: DOMRect) => void;
};

const small =
  "flex h-6 w-6 shrink-0 items-center justify-center rounded text-text-muted outline-none hover:bg-surface-2 hover:text-text focus-visible:ring-2 focus-visible:ring-amber";

// The title of a group of the strip: [arrow] name ... count, play and menu.
// The row has a fixed height, so choosing a group never moves the headers.
// The speed and the end mode go in the tooltip and, for the chosen group
// only, in a quiet line under the row.
export function AnimationHeader({
  group,
  selected,
  onSelect,
  onPlay,
  onMenu,
}: Props) {
  const t = useT();
  const { tag } = group;
  const name =
    group.kind === "base"
      ? t("frames.groupBase")
      : group.kind === "none"
        ? t("frames.groupNone")
        : (tag?.name ?? "");
  const count = t("frames.count", { count: group.frames.length });
  const detail = tag
    ? `${name} · ${t("preview.fps", { fps: tag.fps })} · ${t(modeKeys[tag.mode])}`
    : `${name} · ${count}`;

  return (
    <div
      data-testid={`group-${group.id}`}
      className={`rounded-md border-l-2 ${selected ? "sticky top-0 z-10 border-amber bg-surface-2" : "border-transparent"}`}
    >
      <div className="flex h-8 items-center gap-0.5 pr-0.5 pl-1">
        <button
          type="button"
          data-group-header
          aria-expanded={selected}
          aria-label={t("frames.groupSelect", { name })}
          title={detail}
          onClick={onSelect}
          className="flex h-6 min-w-0 flex-1 items-center gap-1 rounded text-left text-xs font-semibold text-text outline-none hover:text-amber focus-visible:ring-2 focus-visible:ring-amber"
        >
          <Icon
            name="chevronDown"
            className={`h-3 w-3 shrink-0 text-text-muted ${selected ? "" : "-rotate-90"}`}
          />
          <span className="truncate">{name}</span>
        </button>
        <span
          title={count}
          className="shrink-0 rounded bg-bg px-1 font-mono text-[10px] text-text-faint"
        >
          {group.frames.length}
        </span>
        {tag && (
          <>
            <button
              type="button"
              aria-label={t("anim.playNamed", { name })}
              title={t("anim.playNamed", { name })}
              onClick={onPlay}
              className={small}
            >
              <svg viewBox="0 0 24 24" className="h-3 w-3" aria-hidden="true">
                <path d="M7 4l13 8-13 8z" fill="currentColor" />
              </svg>
            </button>
            <button
              type="button"
              aria-label={t("anim.menuNamed", { name })}
              title={t("anim.menuNamed", { name })}
              onClick={(event) =>
                onMenu(event.currentTarget.getBoundingClientRect())
              }
              className={small}
            >
              <svg
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                strokeWidth={3}
                strokeLinecap="round"
                aria-hidden="true"
                className="h-3 w-3"
              >
                <path d="M5 12h.01M12 12h.01M19 12h.01" />
              </svg>
            </button>
          </>
        )}
      </div>
      {selected && tag && (
        <div className="flex items-center gap-1 pb-1.5 pl-6 font-mono text-[10px] text-text-faint">
          <span>{t("preview.fps", { fps: tag.fps })}</span>
          <svg
            viewBox="0 0 24 24"
            role="img"
            aria-label={t(modeKeys[tag.mode])}
            className="h-3 w-3"
            fill="none"
            stroke="currentColor"
            strokeWidth={2}
            strokeLinecap="round"
            strokeLinejoin="round"
          >
            <title>{t(modeKeys[tag.mode])}</title>
            <path d={modePaths[tag.mode]} />
          </svg>
          <span>{t(modeKeys[tag.mode])}</span>
        </div>
      )}
    </div>
  );
}
