import { describe, expect, it } from "vitest";
import { axisLines, mirrorPoints, nextSymmetry } from "./symmetry";

describe("symmetry math", () => {
  // Checks the example of the spec: horizontal mode on a 16 pixel wide
  // sprite draws x 1 and x 14 (the same rule as `symmetry` of the agent).
  it("mirrors left to right around the center", () => {
    expect(mirrorPoints(1, 5, 16, 16, "horizontal")).toEqual([
      { x: 1, y: 5 },
      { x: 14, y: 5 },
    ]);
  });

  // Checks vertical (top to bottom) and both (four points), and that off
  // gives only the pixel.
  it("mirrors top to bottom, both ways, and not at all", () => {
    expect(mirrorPoints(1, 2, 16, 8, "vertical")).toEqual([
      { x: 1, y: 2 },
      { x: 1, y: 5 },
    ]);
    expect(mirrorPoints(1, 2, 16, 8, "both")).toEqual([
      { x: 1, y: 2 },
      { x: 14, y: 2 },
      { x: 1, y: 5 },
      { x: 14, y: 5 },
    ]);
    expect(mirrorPoints(1, 2, 16, 8, "off")).toEqual([{ x: 1, y: 2 }]);
  });

  // Checks an odd size: the middle column is its own mirror.
  it("keeps the middle pixel of an odd size", () => {
    const [self, mirror] = mirrorPoints(3, 0, 7, 7, "horizontal");
    expect(mirror).toEqual(self);
  });

  // Checks the cycle of the S key and the lines drawn on the canvas.
  it("cycles the modes and gives the axis lines", () => {
    expect(nextSymmetry("off")).toBe("horizontal");
    expect(nextSymmetry("horizontal")).toBe("vertical");
    expect(nextSymmetry("vertical")).toBe("both");
    expect(nextSymmetry("both")).toBe("off");
    expect(axisLines(16, 8, "horizontal")).toEqual({ x: 8, y: null });
    expect(axisLines(16, 8, "both")).toEqual({ x: 8, y: 4 });
    expect(axisLines(16, 8, "off")).toEqual({ x: null, y: null });
  });
});
