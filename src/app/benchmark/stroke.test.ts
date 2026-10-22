import { describe, expect, it } from "vitest";
import { median, paintBrush, strokePoints } from "./stroke";

describe("median", () => {
  it("returns the middle value of an odd list", () => {
    expect(median([5, 1, 3])).toBe(3);
  });

  it("returns the mean of the two middle values of an even list", () => {
    expect(median([4, 1, 3, 2])).toBe(2.5);
  });
});

describe("paintBrush", () => {
  it("paints a round brush and returns the changed area", () => {
    const size = 16;
    const indexes = new Uint8Array(size * size);

    const rect = paintBrush(indexes, size, 8, 8, 2, 5);

    expect(indexes[8 * size + 8]).toBe(5);
    expect(indexes[8 * size + 10]).toBe(5);
    expect(indexes[10 * size + 10]).toBe(0); // the corner is outside the circle
    expect(rect).toEqual({ x: 6, y: 6, width: 5, height: 5 });
  });

  it("clips the brush at the edges and never writes outside the sprite", () => {
    const size = 16;
    const indexes = new Uint8Array(size * size);

    const rect = paintBrush(indexes, size, 0, 0, 3, 1);

    expect(rect).toEqual({ x: 0, y: 0, width: 4, height: 4 });
    expect(indexes.length).toBe(size * size);
  });
});

describe("strokePoints", () => {
  it("makes the requested number of points, all inside the sprite", () => {
    const points = strokePoints(512, 300);

    expect(points).toHaveLength(300);
    for (const { x, y } of points) {
      expect(x).toBeGreaterThanOrEqual(0);
      expect(x).toBeLessThan(512);
      expect(y).toBeGreaterThanOrEqual(0);
      expect(y).toBeLessThan(512);
    }
  });
});
