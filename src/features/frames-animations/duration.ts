// Same rule as `frame_duration_ms` in the core: a frame that still has the
// default duration has none of its own, so the speed of the animation decides.
export const DEFAULT_FRAME_MS = 100;

export function frameDurationMs(durationMs: number, fps: number): number {
  return durationMs === DEFAULT_FRAME_MS
    ? Math.round(1000 / Math.max(1, fps))
    : durationMs;
}
