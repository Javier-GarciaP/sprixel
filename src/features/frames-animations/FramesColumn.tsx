import { useRef, useState } from "react";
import { Menu } from "@/shared/components";
import { useT } from "@/shared/lib/i18n";
import { AnimationSelect } from "./AnimationSelect";
import { rangeOf } from "./animation";
import { FrameThumb } from "./FrameThumb";
import type { FramesApi, SpriteRef, Tag } from "./types";
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
  // The animations of the sprite and the chosen one (null is "All"). Only the
  // frames of the chosen animation are listed.
  tags: Tag[];
  animation: string | null;
  onAnimation: (animation: string | null) => void;
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
  tags,
  animation,
  onAnimation,
}: Props) {
  const t = useT();
  const revision = useSpriteChanges(api, sprite);
  const range = rangeOf(tags, animation);
  const first = range?.from ?? 1;
  const last = Math.min(range?.to ?? frameCount, frameCount);
  const frames = Array.from(
    { length: Math.max(0, last - first + 1) },
    (_, i) => first + i,
  );

  const [menu, setMenu] = useState<{
    frame: number;
    x: number;
    y: number;
  } | null>(null);
  const dragged = useRef<number | null>(null);

  // A change that fails (the core knows better) just reads the sprite again.
  const run = (change: Promise<unknown>, select?: number) =>
    change.then(() => onChanged(select)).catch(() => onChanged());

  const remove = (n: number) =>
    // After the delete the current frame keeps its picture when it can.
    run(
      api.deleteFrame(sprite, n),
      n < frame ? frame - 1 : Math.min(frame, frameCount - 1),
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
      <AnimationSelect tags={tags} value={animation} onChange={onAnimation} />
      {frames.map((n) => (
        <div
          key={n}
          className="relative"
          draggable
          onDragStart={(event) => {
            dragged.current = n;
            event.dataTransfer?.setData("text/plain", String(n));
          }}
          onDragOver={(event) => event.preventDefault()}
          onDrop={(event) => {
            event.preventDefault();
            const from = dragged.current;
            dragged.current = null;
            if (from !== null && from !== n) {
              void run(api.moveFrame(sprite, from, n), n);
            }
          }}
          onContextMenu={(event) => {
            event.preventDefault();
            setMenu({ frame: n, x: event.clientX, y: event.clientY });
          }}
        >
          <button
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
          <button
            type="button"
            aria-label={t("frames.menu", { n })}
            onClick={(event) => {
              const box = event.currentTarget.getBoundingClientRect();
              setMenu({ frame: n, x: box.left, y: box.bottom });
            }}
            className="absolute top-1.5 right-1.5 flex h-4 w-4 items-center justify-center rounded bg-black/50 text-[10px] leading-none text-white"
          >
            ⋯
          </button>
        </div>
      ))}
      {menu && (
        <div className="fixed z-30" style={{ left: menu.x, top: menu.y }}>
          <Menu
            onClose={() => setMenu(null)}
            items={[
              {
                id: "duplicate",
                label: t("frames.duplicate"),
                onSelect: () =>
                  void run(
                    api.insertFrame(sprite, menu.frame + 1, menu.frame),
                    menu.frame + 1,
                  ),
              },
              {
                id: "delete",
                label: t("frames.delete"),
                // The last frame stays: a sprite always has one.
                disabled: frameCount <= 1,
                onSelect: () => void remove(menu.frame),
              },
            ]}
          />
        </div>
      )}
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
