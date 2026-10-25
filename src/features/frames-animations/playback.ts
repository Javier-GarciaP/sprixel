import { frameDurationMs } from "./duration";
import type { RepeatMode } from "./types";

// One step of the animation: a frame and how long it is shown.
export type Step = { frame: number; ms: number };

// The steps of one pass. Ping-pong goes forward and then backward, without
// showing the two ends twice, so 1 2 3 4 plays as 1 2 3 4 3 2 and starts again.
export function stepsOf(
  frames: number[],
  durations: number[],
  fps: number,
  mode: RepeatMode,
): Step[] {
  const step = (frame: number): Step => ({
    frame,
    ms: frameDurationMs(durations[frame - 1] ?? 0, fps),
  });
  const forward = frames.map(step);
  if (mode !== "pingpong" || frames.length < 3) return forward;
  return [...forward, ...frames.slice(1, -1).reverse().map(step)];
}

// Which step is shown after `elapsedMs` of playing. It uses the time that
// passed, not a count of ticks, so a slow frame does not make it drift. With
// "once" it stops on the last step (`done`); the other modes repeat.
export function stepAt(
  steps: Step[],
  elapsedMs: number,
  mode: RepeatMode,
): { index: number; done: boolean } {
  const total = steps.reduce((sum, s) => sum + s.ms, 0);
  if (steps.length === 0) return { index: 0, done: true };
  if (mode === "once" && elapsedMs >= total) {
    return { index: steps.length - 1, done: true };
  }
  let left = total > 0 ? elapsedMs % total : 0;
  for (let index = 0; index < steps.length; index++) {
    if (left < steps[index].ms) return { index, done: false };
    left -= steps[index].ms;
  }
  return { index: steps.length - 1, done: false };
}
