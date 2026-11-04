import { describe, expect, it } from "vitest";
import { cursorBox } from "./AgentCursor";

describe("cursorBox", () => {
  // Checks that the box covers its pixel for any zoom and offset: the
  // position is the view offset plus the pixel times the zoom.
  it("follows the zoom and the offset", () => {
    expect(cursorBox({ zoom: 1, x: 0, y: 0 }, 5, 7)).toEqual({
      left: 5,
      top: 7,
      size: 1,
    });
    expect(cursorBox({ zoom: 8, x: 30, y: -12 }, 2, 3)).toEqual({
      left: 46,
      top: 12,
      size: 8,
    });
  });
});
