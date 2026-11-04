import { describe, expect, it } from "vitest";
import { median, wavePoints } from "./measureDrawing";

describe("median", () => {
  // Checks the middle value of an odd list and the mean of the two middle
  // values of an even list.
  it("returns the middle value", () => {
    expect(median([5, 1, 3])).toBe(3);
    expect(median([4, 1, 3, 2])).toBe(2.5);
  });
});

describe("wavePoints", () => {
  // Checks that the stroke of the benchmark stays inside the sprite, has the
  // asked number of points and crosses the whole width.
  it("makes a stroke across the sprite that stays inside it", () => {
    const points = wavePoints(512, 320);
    expect(points).toHaveLength(320);
    expect(points[0].x).toBeLessThan(10);
    expect(points[319].x).toBeGreaterThan(500);
    expect(
      points.every((p) => p.x >= 0 && p.x < 512 && p.y >= 0 && p.y < 512),
    ).toBe(true);
  });
});
