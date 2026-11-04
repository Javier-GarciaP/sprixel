import { describe, expect, it } from "vitest";
import { DEFAULT_FRAME_MS, frameDurationMs } from "./duration";

describe("frameDurationMs", () => {
  // Checks the "Default duration" scenario: at 12 fps a frame with no duration
  // of its own lasts about 83 ms.
  it("lets the speed decide when the frame has no duration of its own", () => {
    expect(frameDurationMs(DEFAULT_FRAME_MS, 12)).toBe(83);
    expect(frameDurationMs(DEFAULT_FRAME_MS, 1)).toBe(1000);
    expect(frameDurationMs(DEFAULT_FRAME_MS, 60)).toBe(17);
  });

  // Checks that a frame with its own duration keeps it at any speed.
  it("keeps the duration a frame has of its own", () => {
    expect(frameDurationMs(250, 12)).toBe(250);
    expect(frameDurationMs(40, 1)).toBe(40);
  });
});
