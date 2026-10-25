import { describe, expect, it } from "vitest";
import { stepAt, stepsOf } from "./playback";

const D = 100; // the duration of a frame with none of its own
const durations = [D, D, D, D];
const frames = [1, 2, 3, 4];
// At 10 fps every frame without a duration lasts 100 ms.
const shown = (
  steps: ReturnType<typeof stepsOf>,
  mode: "loop" | "once" | "pingpong",
  at: number,
) => steps[stepAt(steps, at, mode).index].frame;

describe("stepsOf", () => {
  // Checks that the speed gives the length of each step: 12 fps is 83 ms, and
  // a frame with its own duration keeps it.
  it("uses the speed, or the duration a frame has of its own", () => {
    expect(stepsOf([1, 2], [D, 250], 12, "loop")).toEqual([
      { frame: 1, ms: 83 },
      { frame: 2, ms: 250 },
    ]);
  });

  // Checks that ping-pong does not show the two ends twice.
  it("goes forward and back in ping-pong", () => {
    expect(
      stepsOf(frames, durations, 10, "pingpong").map((s) => s.frame),
    ).toEqual([1, 2, 3, 4, 3, 2]);
    expect(
      stepsOf([1, 2], durations, 10, "pingpong").map((s) => s.frame),
    ).toEqual([1, 2]);
  });
});

describe("stepAt timing", () => {
  // Checks the repeat mode "loop": the frames follow each other by the time
  // that passed and start again after the last one.
  it("repeats in loop mode", () => {
    const steps = stepsOf(frames, durations, 10, "loop");
    const at = (ms: number) => shown(steps, "loop", ms);
    expect([0, 99, 100, 250, 399].map(at)).toEqual([1, 1, 2, 3, 4]);
    expect([400, 520, 1000, 1290].map(at)).toEqual([1, 2, 3, 1]);
    expect(stepAt(steps, 5000, "loop").done).toBe(false);
  });

  // Checks "once": it plays the frames one time and stays on the last one,
  // and says it is done.
  it("plays once and stops on the last frame", () => {
    const steps = stepsOf(frames, durations, 10, "once");
    expect(stepAt(steps, 399, "once")).toEqual({ index: 3, done: false });
    expect(stepAt(steps, 400, "once")).toEqual({ index: 3, done: true });
    expect(shown(steps, "once", 99999)).toBe(4);
  });

  // Checks "ping-pong": forward and then backward, again and again, with the
  // time of each step (1 2 3 4 3 2, 600 ms in all).
  it("goes forward and then backward, repeatedly", () => {
    const steps = stepsOf(frames, durations, 10, "pingpong");
    const at = (ms: number) => shown(steps, "pingpong", ms);
    expect([0, 100, 200, 300, 400, 500].map(at)).toEqual([1, 2, 3, 4, 3, 2]);
    expect([600, 700, 1000, 1200].map(at)).toEqual([1, 2, 3, 1]);
  });

  // Checks that the time that passed decides, not the number of ticks: a long
  // pause between two ticks jumps to the right frame.
  it("does not drift when a tick is late", () => {
    const steps = stepsOf(frames, durations, 10, "loop");
    expect(shown(steps, "loop", 1950)).toBe(4);
  });

  // Checks a different length for each frame, and one frame only.
  it("adds up different durations", () => {
    const steps = stepsOf([1, 2], [D, 300], 10, "loop");
    expect(
      [0, 99, 100, 399, 400].map((ms) => shown(steps, "loop", ms)),
    ).toEqual([1, 1, 2, 2, 1]);
    expect(stepAt(stepsOf([1], [D], 10, "once"), 5000, "once").index).toBe(0);
  });
});
