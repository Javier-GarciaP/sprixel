// The timing of a replay, as plain functions (no DOM, no timers), so they are
// easy to test. The replay shows the pixels of an agent write one after the
// other; this says how many should be visible after some time.

// A replay never takes longer than this, whatever the number of pixels.
export const MAX_REPLAY_MS = 8000;
// A short write still needs a moment to be seen.
export const MIN_REPLAY_MS = 250;
// Each pixel gets this much time, until the cap.
export const MS_PER_PIXEL = 30;
// When more replays than this wait, the editor skips to the final state.
export const MAX_WAITING = 3;

// How long the replay of `count` pixels lasts.
export function replayDuration(count: number): number {
  if (count <= 0) return 0;
  return Math.min(MAX_REPLAY_MS, Math.max(MIN_REPLAY_MS, count * MS_PER_PIXEL));
}

// How many pixels are visible after `elapsed` milliseconds: a straight line
// from none to all of them.
export function pixelsShown(count: number, elapsed: number): number {
  const duration = replayDuration(count);
  if (duration === 0 || elapsed >= duration) return Math.max(0, count);
  if (elapsed <= 0) return 0;
  return Math.floor((elapsed / duration) * count);
}

// True when so many replays wait that it is better to show the final state.
export function tooManyWaiting(waiting: number): boolean {
  return waiting > MAX_WAITING;
}
