import { describe, expect, it } from "vitest";
import type { PixelChange } from "../api/types";
import type { Rect } from "../types";
import { ColorBook } from "./colorBook";
import { DrawingController } from "./DrawingController";

const code = (key: string) => key.charCodeAt(0);

// A controller on a blank sprite, with the palette R (red) and B (blue).
export function setup(width = 8, height = 8) {
  const pixels = new Uint8Array(width * height).fill(46);
  const commits: PixelChange[][] = [];
  const dirty: Rect[] = [];
  const colors = new ColorBook([
    { key: "R", hex: "#FF0000" },
    { key: "B", hex: "#0000FF" },
  ]);
  const controller = new DrawingController(width, height, pixels, colors, {
    onDirty: (area) => dirty.push(area),
    onCommit: (changes) => commits.push(changes),
  });
  controller.main = "#FF0000";
  controller.secondary = "#0000FF";
  const at = (x: number, y: number) =>
    String.fromCharCode(pixels[y * width + x]);
  return { controller, pixels, commits, dirty, colors, at };
}

const plain = { shift: false };

describe("Pencil", () => {
  // Checks that a click paints one pixel with the main color and ends with
  // one commit that holds only that pixel.
  it("paints the pixel under the pointer with the main color", () => {
    const { controller, commits, at } = setup();
    controller.pointerDown(2, 3, 0, plain);
    controller.pointerUp();
    expect(at(2, 3)).toBe("R");
    expect(commits).toEqual([[{ x: 2, y: 3, color: "#FF0000" }]]);
  });

  // Checks the "Fast stroke" scenario: the pointer jumps across 100 pixels in
  // one move and every pixel on the path is painted.
  it("paints every pixel on the path of a fast move", () => {
    const { controller, commits, at } = setup(128, 4);
    controller.pointerDown(0, 1, 0, plain);
    controller.pointerMove(99, 1, plain);
    controller.pointerUp();
    for (let x = 0; x < 100; x++) expect(at(x, 1)).toBe("R");
    expect(at(100, 1)).toBe(".");
    expect(commits).toHaveLength(1);
    expect(commits[0]).toHaveLength(100);
  });

  // Checks the diagonal case: a fast diagonal move has no gaps either.
  it("leaves no gaps on a diagonal", () => {
    const { controller, at } = setup();
    controller.pointerDown(0, 0, 0, plain);
    controller.pointerMove(7, 7, plain);
    controller.pointerUp();
    for (let i = 0; i < 8; i++) expect(at(i, i)).toBe("R");
  });

  // Checks the "Pointer leaves the canvas" scenario: the stroke goes out of
  // the sprite and back, and only pixels inside are painted.
  it("paints only the pixels inside the sprite", () => {
    const { controller, commits, pixels } = setup(8, 8);
    controller.pointerDown(6, 4, 0, plain);
    controller.pointerMove(12, 4, plain);
    controller.pointerMove(-3, 4, plain);
    controller.pointerUp();
    expect(pixels.filter((c) => c === code("R"))).toHaveLength(8);
    expect(commits[0].every((c) => c.x >= 0 && c.x < 8)).toBe(true);
  });

  // Checks that the commit does not include pixels that already had the color
  // (painting over the same color changes nothing).
  it("does not send pixels that did not change", () => {
    const { controller, commits } = setup();
    controller.pointerDown(1, 1, 0, plain);
    controller.pointerUp();
    controller.pointerDown(1, 1, 0, plain);
    controller.pointerMove(2, 1, plain);
    controller.pointerUp();
    expect(commits[1]).toEqual([{ x: 2, y: 1, color: "#FF0000" }]);
  });

  // Checks that the area to repaint covers the touched pixels only.
  it("asks to repaint only the changed area", () => {
    const { controller, dirty } = setup(64, 64);
    controller.pointerDown(10, 10, 0, plain);
    controller.pointerMove(12, 11, plain);
    controller.pointerUp();
    const union = dirty.reduce((a, b) => ({
      x: Math.min(a.x, b.x),
      y: Math.min(a.y, b.y),
      width: Math.max(a.x + a.width, b.x + b.width) - Math.min(a.x, b.x),
      height: Math.max(a.y + a.height, b.y + b.height) - Math.min(a.y, b.y),
    }));
    expect(union).toEqual({ x: 10, y: 10, width: 3, height: 2 });
  });

  // Checks that moving the pointer without pressing does nothing.
  it("ignores moves while no stroke is on", () => {
    const { controller, commits, pixels } = setup();
    controller.pointerMove(3, 3, plain);
    controller.pointerUp();
    expect(pixels.every((c) => c === 46)).toBe(true);
    expect(commits).toEqual([]);
  });
});

describe("Eraser", () => {
  // Checks that the Eraser sets the touched pixels to transparent (null in
  // the commit) and ignores the color.
  it("makes the touched pixels transparent", () => {
    const { controller, commits, at } = setup();
    controller.pointerDown(0, 0, 0, plain);
    controller.pointerMove(3, 0, plain);
    controller.pointerUp();
    controller.tool = "eraser";
    controller.pointerDown(1, 0, 0, plain);
    controller.pointerMove(2, 0, plain);
    controller.pointerUp();
    expect(at(0, 0)).toBe("R");
    expect(at(1, 0)).toBe(".");
    expect(at(2, 0)).toBe(".");
    expect(at(3, 0)).toBe("R");
    expect(commits[1]).toEqual([
      { x: 1, y: 0, color: null },
      { x: 2, y: 0, color: null },
    ]);
  });

  // Checks that erasing a pixel that is already transparent sends nothing.
  it("sends nothing when there is nothing to erase", () => {
    const { controller, commits } = setup();
    controller.tool = "eraser";
    controller.pointerDown(4, 4, 0, plain);
    controller.pointerUp();
    expect(commits).toEqual([]);
  });
});
