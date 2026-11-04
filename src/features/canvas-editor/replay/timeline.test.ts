import { describe, expect, it } from "vitest";
import {
  MAX_REPLAY_MS,
  MIN_REPLAY_MS,
  pixelsShown,
  replayDuration,
  tooManyWaiting,
} from "./timeline";

describe("replayDuration", () => {
  // Checks the "Big write" scenario: 5000 pixels (or any huge number) never
  // last more than 8 seconds.
  it("never lasts more than 8 seconds", () => {
    expect(replayDuration(5000)).toBe(MAX_REPLAY_MS);
    expect(replayDuration(1_000_000)).toBe(8000);
  });

  // Checks that a small write still lasts long enough to be seen, and that
  // nothing to show takes no time.
  it("gives a short write a minimum time and an empty one none", () => {
    expect(replayDuration(1)).toBe(MIN_REPLAY_MS);
    expect(replayDuration(0)).toBe(0);
  });

  // Checks that more pixels never take less time.
  it("grows with the number of pixels", () => {
    let last = 0;
    for (const count of [1, 5, 10, 50, 100, 300, 1000]) {
      const duration = replayDuration(count);
      expect(duration).toBeGreaterThanOrEqual(last);
      last = duration;
    }
  });
});

describe("pixelsShown", () => {
  // Checks that the replay starts with nothing and ends with every pixel,
  // also when the time is past the end.
  it("goes from none to all", () => {
    const end = replayDuration(100);
    expect(pixelsShown(100, 0)).toBe(0);
    expect(pixelsShown(100, end)).toBe(100);
    expect(pixelsShown(100, end + 5000)).toBe(100);
  });

  // Checks that pixels only get added with time (the cursor never goes back).
  it("never goes back", () => {
    let last = 0;
    for (let ms = 0; ms <= 8200; ms += 100) {
      const shown = pixelsShown(5000, ms);
      expect(shown).toBeGreaterThanOrEqual(last);
      last = shown;
    }
    expect(last).toBe(5000);
  });

  // Checks that 5000 pixels are all shown at the 8 second cap, not before.
  it("shows the 5000 pixels of a big write at 8 seconds", () => {
    expect(pixelsShown(5000, 7999)).toBeLessThan(5000);
    expect(pixelsShown(5000, 8000)).toBe(5000);
  });

  // Checks the empty write: no pixels, nothing shown.
  it("shows nothing for an empty write", () => {
    expect(pixelsShown(0, 1000)).toBe(0);
  });
});

describe("tooManyWaiting", () => {
  // Checks the queue rule: up to 3 waiting replays play in order, more than
  // 3 jump to the final state.
  it("jumps when more than 3 wait", () => {
    expect(tooManyWaiting(3)).toBe(false);
    expect(tooManyWaiting(4)).toBe(true);
  });
});
