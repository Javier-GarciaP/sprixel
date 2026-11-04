import { Icon } from "@/shared/components";
import { useT, type TranslationKey } from "@/shared/lib/i18n";
import type { Group } from "./groups";
import type { RepeatMode } from "./types";

// How an animation ends, drawn on a 24x24 grid like the shared icons.
const modePaths: Record<RepeatMode, string> = {
  loop: "M17 2l4 4-4 4M3 11V9a3 3 0 013-3h15M7 22l-4-4 4-4M21 13v2a3 3 0 01-3 3H3",
  once: "M5 12h14M13 6l6 6-6 6",
  pingpong: "M7 7l-4 5 4 5M17 7l4 5-4 5M3 12h18",
};

const modeKeys: Record<RepeatMode, TranslationKey> = {
  loop: "anim.modeLoop",
  once: "anim.modeOnce",
  pingpong: "anim.modePingpong",
};

type Props = {
  group: Group;
  open: boolean;
  onToggle: () => void;
  // True for the animation the preview is set to.
  chosen: boolean;
  onPlay: () => void;
  onMenu: (box: DOMRect) => void;
};

const small =
  "flex h-5 w-5 shrink-0 items-center justify-center rounded text-text-muted hover:bg-surface-2 hover:text-text";

// The title of a group of the strip. The height is fixed, so opening or
// closing groups never moves the headers around.
export function AnimationHeader({
  group,
  open,
  onToggle,
  chosen,
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

  return (
    <div
      data-testid={`group-${group.id}`}
      className={`flex h-9 w-full flex-col justify-center rounded-md border px-1 ${chosen ? "border-amber" : "border-transparent"}`}
    >
      <div className="flex items-center gap-0.5">
        <button
          type="button"
          aria-expanded={open}
          aria-label={t(open ? "frames.groupClose" : "frames.groupOpen", {
            name,
          })}
          title={t(open ? "frames.groupClose" : "frames.groupOpen", { name })}
          onClick={onToggle}
          className="flex min-w-0 flex-1 items-center gap-0.5 rounded text-left text-xs font-semibold text-text hover:bg-surface-2"
        >
          <Icon
            name="chevronDown"
            className={`h-3 w-3 shrink-0 ${open ? "" : "-rotate-90"}`}
          />
          <span className="truncate">{name}</span>
        </button>
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
              ⋯
            </button>
          </>
        )}
      </div>
      <div className="flex items-center gap-1 pl-3.5 font-mono text-[10px] text-text-faint">
        {tag ? (
          <>
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
          </>
        ) : null}
        <span>{t("frames.count", { count: group.frames.length })}</span>
      </div>
    </div>
  );
}
