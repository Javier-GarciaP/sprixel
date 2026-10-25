import { useT } from "@/shared/lib/i18n";
import { FrameThumb } from "./FrameThumb";
import type { FramesApi, SpriteRef } from "./types";
import { useSpriteChanges } from "./useSpriteChanges";

type Props = {
  api: FramesApi;
  sprite: SpriteRef;
  frameCount: number;
  frame: number;
  onFrame: (frame: number) => void;
  // After a change: the editor reads the sprite again and, with a number,
  // goes to that frame.
  onChanged: (select?: number) => void;
  // The frame an AI agent is writing to, if any.
  busyFrame?: number | null;
  // Only the frames of the chosen animation, or all of them.
  range?: { from: number; to: number } | null;
};

const button =
  "flex h-7 flex-1 items-center justify-center rounded border border-border bg-surface text-sm text-text hover:bg-surface-2";

// The frames of the sprite, one thumbnail each. "+" adds a frame right after
// the current one, blank or as a copy of it.
export function FramesColumn({
  api,
  sprite,
  frameCount,
  frame,
  onFrame,
  onChanged,
  busyFrame,
  range,
}: Props) {
  const t = useT();
  const revision = useSpriteChanges(api, sprite);
  const first = range?.from ?? 1;
  const last = Math.min(range?.to ?? frameCount, frameCount);
  const frames = Array.from(
    { length: Math.max(0, last - first + 1) },
    (_, i) => first + i,
  );

  const add = (copyOf: number | null) =>
    api
      .insertFrame(sprite, frame + 1, copyOf)
      .then((created) => onChanged(created))
      .catch(() => onChanged());

  return (
    <aside
      aria-label={t("editor.frames")}
      className="flex w-28 shrink-0 flex-col items-center gap-2 overflow-y-auto border-r border-border bg-bg py-3"
    >
      <h2 className="text-xs font-semibold text-text-muted">
        {t("editor.frames")}
      </h2>
      {frames.map((n) => (
        <button
          key={n}
          type="button"
          aria-label={t("editor.frame", { n })}
          aria-current={n === frame ? "true" : undefined}
          aria-busy={n === busyFrame ? true : undefined}
          title={n === busyFrame ? t("marker.frameBusy") : undefined}
          onClick={() => onFrame(n)}
          className={`relative rounded-md border p-1 ${n === frame ? "border-amber bg-surface-2" : "border-border bg-surface"}`}
        >
          <FrameThumb
            api={api}
            sprite={sprite}
            frame={n}
            revision={revision}
            box={64}
          />
          <span className="absolute bottom-1.5 left-2 font-mono text-[10px] text-white">
            {n}
          </span>
          {n === busyFrame && (
            <span
              aria-hidden
              className="absolute top-1.5 right-2 h-2 w-2 rounded-full bg-amber"
            />
          )}
        </button>
      ))}
      <div className="flex w-20 gap-1">
        <button
          type="button"
          className={button}
          aria-label={t("frames.add")}
          title={t("frames.add")}
          onClick={() => add(null)}
        >
          +
        </button>
        <button
          type="button"
          className={button}
          aria-label={t("frames.addCopy")}
          title={t("frames.addCopy")}
          onClick={() => add(frame)}
        >
          ⧉
        </button>
      </div>
    </aside>
  );
}
