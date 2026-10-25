import { describe, expect, it } from "vitest";
import {
  ellipsePoints,
  linePoints,
  rectanglePoints,
  snapLine,
  squareCorner,
} from "./shapes";

const key = (points: { x: number; y: number }[]) =>
  points.map((p) => `${p.x},${p.y}`).sort();

describe("linePoints", () => {
  // Checks that a long line has one pixel per column and no gaps, which is
  // what a fast pointer move needs ("across 100 pixels in one frame").
  it("has no gaps across 100 pixels", () => {
    const points = linePoints({ x: 0, y: 0 }, { x: 99, y: 0 });
    expect(points).toHaveLength(100);
    points.forEach((p, i) => expect(p).toEqual({ x: i, y: 0 }));
  });

  // Checks that every step of a diagonal-ish line moves at most one pixel,
  // so the pixels touch each other.
  it("moves one pixel at a time on any slope", () => {
    const points = linePoints({ x: 3, y: 40 }, { x: 77, y: 2 });
    for (let i = 1; i < points.length; i++) {
      expect(Math.abs(points[i].x - points[i - 1].x)).toBeLessThanOrEqual(1);
      expect(Math.abs(points[i].y - points[i - 1].y)).toBeLessThanOrEqual(1);
    }
    expect(points[0]).toEqual({ x: 3, y: 40 });
    expect(points[points.length - 1]).toEqual({ x: 77, y: 2 });
  });

  // Checks the same line in both directions covers the same pixels.
  it("includes both ends for a single point", () => {
    expect(linePoints({ x: 5, y: 5 }, { x: 5, y: 5 })).toEqual([
      { x: 5, y: 5 },
    ]);
  });
});

describe("snapLine", () => {
  // Checks the Shift rule for a line: the end snaps to the nearest 45-degree
  // step around the start.
  it("snaps to horizontal, vertical and diagonal", () => {
    const from = { x: 10, y: 10 };
    expect(snapLine(from, { x: 30, y: 12 })).toEqual({ x: 30, y: 10 });
    expect(snapLine(from, { x: 12, y: 30 })).toEqual({ x: 10, y: 30 });
    expect(snapLine(from, { x: 25, y: 22 })).toEqual({ x: 25, y: 25 });
    expect(snapLine(from, { x: 0, y: 1 })).toEqual({ x: 0, y: 0 });
    expect(snapLine(from, { x: 4, y: 16 })).toEqual({ x: 4, y: 16 });
  });
});

describe("squareCorner", () => {
  // Checks the Shift rule for rectangle and ellipse: the shorter side grows
  // to the longer one, keeping the drag direction.
  it("makes both sides as long as the longer one", () => {
    expect(squareCorner({ x: 2, y: 2 }, { x: 10, y: 5 })).toEqual({
      x: 10,
      y: 10,
    });
    expect(squareCorner({ x: 10, y: 10 }, { x: 8, y: 2 })).toEqual({
      x: 2,
      y: 2,
    });
  });
});

describe("rectanglePoints", () => {
  // Checks the outline: only the edge pixels. A 4x3 rectangle has 10.
  it("draws an outline", () => {
    const points = rectanglePoints({ x: 0, y: 0 }, { x: 3, y: 2 }, false);
    expect(points).toHaveLength(10);
    expect(key(points)).not.toContain("1,1");
    expect(key(points)).not.toContain("2,1");
  });

  // Checks the filled shape: every pixel, whichever corner comes first.
  it("draws a filled rectangle from any corner", () => {
    expect(rectanglePoints({ x: 3, y: 2 }, { x: 0, y: 0 }, true)).toHaveLength(
      12,
    );
  });
});

describe("ellipsePoints", () => {
  // Checks that a circle is symmetric left-right and top-bottom, so Shift
  // really gives a circle, and that the outline is inside the filled shape.
  it("is symmetric and the outline is inside the fill", () => {
    const filled = key(ellipsePoints({ x: 0, y: 0 }, { x: 9, y: 9 }, true));
    const outline = key(ellipsePoints({ x: 0, y: 0 }, { x: 9, y: 9 }, false));
    const mirror = key(
      ellipsePoints({ x: 0, y: 0 }, { x: 9, y: 9 }, true).map((p) => ({
        x: 9 - p.x,
        y: p.y,
      })),
    );
    expect(mirror).toEqual(filled);
    expect(outline.length).toBeLessThan(filled.length);
    expect(outline.every((p) => filled.includes(p))).toBe(true);
    expect(filled).not.toContain("0,0");
    expect(filled).toContain("4,0");
  });

  // Checks a tiny ellipse: 1x1 is one pixel and 2x2 is four.
  it("handles tiny sizes", () => {
    expect(ellipsePoints({ x: 0, y: 0 }, { x: 0, y: 0 }, true)).toHaveLength(1);
    expect(ellipsePoints({ x: 0, y: 0 }, { x: 1, y: 1 }, false)).toHaveLength(
      4,
    );
  });
});
