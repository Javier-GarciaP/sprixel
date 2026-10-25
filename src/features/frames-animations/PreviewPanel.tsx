import { useEffect, useMemo, useRef, useState } from "react";
import type { FrameData } from "@/shared/lib/core";
import { useT } from "@/shared/lib/i18n";
import { drawFrame } from "@/shared/lib/render";
import { stepAt, stepsOf } from "./playback";
import type { FramesApi, FramesInfo, SpriteRef, Tag } from "./types";
import { useSpriteChanges } from "./useSpriteChanges";

// Where the time and the next frame of the screen come from. The tests give
// their own, so they do not wait for real time.
export type Clock = {
  now: () => number;
  frame: (callback: () => void) => () => void;
};

const screenClock: Clock = {
  now: () => performance.now(),
  frame: (callback) => {
    const id = requestAnimationFrame(callback);
    return () => cancelAnimationFrame(id);
  },
};

const DEFAULT_FPS = 12;
const BOX = 200;

type Props = {
  api: FramesApi;
  sprite: SpriteRef;
  info: FramesInfo | null;
  // The animation that is played, or null to play all the frames.
  tag: Tag | null;
  clock?: Clock;
};

// A small canvas that plays the animation, or all the frames, with play and
// pause and a speed from 1 to 60 fps. The frames are read again when the
// sprite changes, so what is drawn shows on the next pass.
export function PreviewPanel({
  api,
  sprite,
  info,
  tag,
  clock = screenClock,
}: Props) {
  const t = useT();
  const revision = useSpriteChanges(api, sprite);
  const canvas = useRef<HTMLCanvasElement>(null);
  const [fps, setFps] = useState(tag?.fps ?? DEFAULT_FPS);
  const [playing, setPlaying] = useState(false);
  const [data, setData] = useState<Map<number, FrameData>>(new Map());

  // A different animation starts with its own speed.
  useEffect(() => setFps(tag?.fps ?? DEFAULT_FPS), [tag?.name, tag?.fps]);

  const count = info?.frameCount ?? 0;
  const from = tag?.from ?? 1;
  const to = Math.min(tag?.to ?? count, count);
  const mode = tag?.mode ?? "loop";
  const durations = info?.durations;
  const steps = useMemo(
    () =>
      stepsOf(
        Array.from({ length: Math.max(0, to - from + 1) }, (_, i) => from + i),
        durations ?? [],
        fps,
        mode,
      ),
    [from, to, durations, fps, mode],
  );
  const [shown, setShown] = useState(from);

  const path = JSON.stringify([...sprite.folder, sprite.name]);
  useEffect(() => {
    let stopped = false;
    const wanted = Array.from(
      { length: Math.max(0, to - from + 1) },
      (_, i) => from + i,
    );
    Promise.all(wanted.map((n) => api.loadFrameData(sprite, n)))
      .then((frames) => {
        if (!stopped) setData(new Map(wanted.map((n, i) => [n, frames[i]])));
      })
      .catch(() => {});
    return () => {
      stopped = true;
    };
  }, [api, path, from, to, revision, count]);

  // The time that passed decides the frame, not the number of ticks.
  // A different range stops and starts on its first frame. Pausing, or the end of a
  // "once" animation, leaves the frame that is shown.
  useEffect(() => {
    setShown(from);
    setPlaying(false);
  }, [from, to]);

  useEffect(() => {
    if (!playing || steps.length === 0) return;
    const start = clock.now();
    let cancel = () => {};
    const tick = () => {
      const { index, done } = stepAt(steps, clock.now() - start, mode);
      setShown(steps[index].frame);
      if (done) setPlaying(false);
      else cancel = clock.frame(tick);
    };
    tick();
    return () => cancel();
  }, [playing, steps, mode, clock]);

  useEffect(() => {
    const picture = data.get(shown);
    if (picture && canvas.current) drawFrame(canvas.current, picture, BOX);
  }, [data, shown]);

  return (
    <div data-testid="preview" data-frame={shown} data-ready={data.size > 0}>
      <div
        className="flex items-center justify-center rounded bg-surface-2"
        style={{ height: BOX }}
      >
        <canvas
          ref={canvas}
          role="img"
          aria-label={t("preview.canvas", { frame: shown })}
          className="[image-rendering:pixelated]"
        />
      </div>
      <div className="mt-2 flex items-center gap-2">
        <button
          type="button"
          aria-label={playing ? t("preview.pause") : t("preview.play")}
          onClick={() => setPlaying(!playing)}
          className="flex h-7 w-7 items-center justify-center rounded-full bg-amber text-ink"
        >
          {playing ? "❚❚" : "▶"}
        </button>
        <input
          type="range"
          min={1}
          max={60}
          value={fps}
          aria-label={t("preview.speed")}
          onChange={(event) => setFps(Number(event.target.value))}
          className="flex-1 accent-amber"
        />
        <span className="w-14 text-right font-mono text-xs">
          {t("preview.fps", { fps })}
        </span>
      </div>
    </div>
  );
}
