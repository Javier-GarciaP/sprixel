import { useEffect, useRef, useState, type RefObject } from "react";
import type { Clock } from "@/features/frames-animations";
import type { AgentOps, EditorApi, SpriteRef } from "../api/types";
import type { PixelCanvasHandle } from "../components/PixelCanvas";
import { pixelsShown, tooManyWaiting } from "../replay/timeline";
import type { Rect } from "../types";
import { sameSprite } from "./useAgentActivity";

// The real clock of the screen. The tests give a manual one.
export const screenClock: Clock = {
  now: () => performance.now(),
  frame: (callback) => {
    const id = requestAnimationFrame(callback);
    return () => cancelAnimationFrame(id);
  },
};

// Where the agent cursor is, in sprite pixels.
export type AgentCursor = { x: number; y: number; agent: string };

type Options = {
  api: EditorApi;
  sprite: SpriteRef;
  frame: number;
  // The pixels the canvas shows. The replay writes into this same array and
  // repaints the small area it changed.
  pixels: Uint8Array;
  width: number;
  height: number;
  canvasRef: RefObject<PixelCanvasHandle | null>;
  clock: Clock;
  // With reduced motion the final state appears at once.
  reducedMotion: boolean;
  // True while the person has a stroke in progress: the view must not change.
  strokeActive: RefObject<boolean>;
  // Shows another frame of the same sprite (the agent works there).
  onFrame: (frame: number) => void;
  // Changes each time an agent ends or is stopped: any replay jumps to the end.
  endCount: number;
};

type Running = { ops: AgentOps; shown: number; cancel: () => void };

// Replays the pixels of the agent writes (event `agent-ops`) on the canvas, in
// order, with a cursor. The core already stored the write: this is only what
// the person sees. When a replay ends the editor tells the core (`playbackDone`)
// and the core frees the frame. Replays of one sprite play one after the other.
export function useAgentPlayback(options: Options) {
  const [cursor, setCursor] = useState<AgentCursor | null>(null);
  // The newest options, read by the listener that was made once.
  const latest = useRef(options);
  latest.current = options;
  const queue = useRef<AgentOps[]>([]);
  const running = useRef<Running | null>(null);

  // Puts pixels `from` to `to` of a write on the canvas and repaints them.
  const apply = (ops: AgentOps, from: number, to: number) => {
    const { pixels, width, height, canvasRef } = latest.current;
    let area: Rect | null = null;
    for (let i = from; i < to; i++) {
      const [x, y, key] = ops.pixels[i];
      if (x < 0 || y < 0 || x >= width || y >= height) continue;
      pixels[y * width + x] = key.charCodeAt(0);
      area = area
        ? {
            x: Math.min(area.x, x),
            y: Math.min(area.y, y),
            width: Math.max(area.x + area.width, x + 1) - Math.min(area.x, x),
            height: Math.max(area.y + area.height, y + 1) - Math.min(area.y, y),
          }
        : { x, y, width: 1, height: 1 };
    }
    if (area) canvasRef.current?.invalidate(area);
    if (to > from && to <= ops.pixels.length) {
      const [x, y] = ops.pixels[to - 1];
      setCursor({ x, y, agent: ops.agent });
    }
  };

  const done = (ops: AgentOps) => {
    latest.current.api.playbackDone(ops.id).catch(() => {});
  };

  // Shows the final state of everything that plays or waits.
  const finishAll = () => {
    const current = running.current;
    running.current = null;
    if (current) {
      current.cancel();
      apply(current.ops, current.shown, current.ops.pixels.length);
      done(current.ops);
    }
    for (const ops of queue.current.splice(0)) {
      apply(ops, 0, ops.pixels.length);
      done(ops);
    }
    setCursor(null);
  };

  const pump = () => {
    if (running.current) return;
    const ops = queue.current.shift();
    if (!ops) {
      setCursor(null);
      return;
    }
    const { clock } = latest.current;
    const start = clock.now();
    const state: Running = { ops, shown: 0, cancel: () => {} };
    running.current = state;
    const step = () => {
      if (running.current !== state) return;
      const to = pixelsShown(ops.pixels.length, clock.now() - start);
      apply(ops, state.shown, to);
      state.shown = to;
      if (to >= ops.pixels.length) {
        running.current = null;
        done(ops);
        pump();
      } else {
        state.cancel = clock.frame(step);
      }
    };
    state.cancel = clock.frame(step);
  };

  const handle = (ops: AgentOps) => {
    const { sprite, frame, reducedMotion, strokeActive, onFrame } =
      latest.current;
    // Another sprite is not this editor's business here (see the follow rule
    // in EditorScreen).
    if (!sameSprite(ops.sprite, sprite)) return;
    if (ops.frame !== frame) {
      // Another frame of this sprite: show it only if the person is free.
      // The frame is read from the core, which has the final state.
      if (!strokeActive.current) onFrame(ops.frame);
      done(ops);
      return;
    }
    if (reducedMotion) {
      apply(ops, 0, ops.pixels.length);
      done(ops);
      setCursor(null);
      return;
    }
    queue.current.push(ops);
    if (tooManyWaiting(queue.current.length)) finishAll();
    else pump();
  };

  useEffect(() => {
    let stopped = false;
    let unlisten: (() => void) | undefined;
    options.api
      .onAgentOps((ops) => {
        if (!stopped) handle(ops);
      })
      .then((stop) => {
        if (stopped) stop();
        else unlisten = stop;
      })
      .catch(() => {});
    return () => {
      stopped = true;
      unlisten?.();
      // Leaving the sprite: nothing may keep the frame busy.
      finishAll();
    };
    // The listener is made once for this api.
  }, [options.api]);

  // The agent ended or the person pressed Stop: show the final state at once.
  useEffect(() => {
    if (options.endCount > 0) finishAll();
  }, [options.endCount]);

  return { cursor };
}
