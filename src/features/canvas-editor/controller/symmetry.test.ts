import { describe, expect, it } from "vitest";
import type { PixelChange } from "../api/types";
import { ColorBook } from "./colorBook";
import { DrawingController } from "./DrawingController";
import type { ToolId } from "../tools/tools";
import type { Symmetry } from "../tools/symmetry";

const plain = { shift: false };

// A blank sprite, a red main color and a symmetry mode.
function setup(symmetry: Symmetry, tool: ToolId, width = 8, height = 8) {
  const pixels = new Uint8Array(width * height).fill(46);
  const commits: PixelChange[][] = [];
  const controller = new DrawingController(
    width,
    height,
    pixels,
    new ColorBook([{ key: "R", hex: "#FF0000" }]),
    { onDirty: () => {}, onCommit: (c) => commits.push(c), onPick: () => {} },
  );
  controller.main = "#FF0000";
  controller.symmetry = symmetry;
  controller.tool = tool;
  const at = (x: number, y: number) =>
    String.fromCharCode(pixels[y * width + x]);
  const red = () =>
    [...pixels].flatMap((p, i) =>
      p === 82 ? [[i % width, Math.floor(i / width)]] : [],
    );
  return { controller, commits, at, red };
}

describe("Drawing with symmetry", () => {
  // Checks the pencil: one click draws the pixel and its mirror, and the
  // stroke is ONE commit (so one history entry) with both pixels.
  it("mirrors a pencil stroke into one commit", () => {
    const { controller, commits, red } = setup("horizontal", "pencil");
    controller.pointerDown(1, 2, 0, plain);
    controller.pointerMove(2, 2, plain);
    controller.pointerUp();
    expect(red()).toEqual([
      [1, 2],
      [2, 2],
      [5, 2],
      [6, 2],
    ]);
    expect(commits).toHaveLength(1);
    expect(commits[0]).toHaveLength(4);
  });

  // Checks that "both" draws four pixels and "off" only one.
  it("draws four pixels with both axes and one with symmetry off", () => {
    const both = setup("both", "pencil");
    both.controller.pointerDown(1, 1, 0, plain);
    both.controller.pointerUp();
    expect(both.red()).toHaveLength(4);
    const off = setup("off", "pencil");
    off.controller.pointerDown(1, 1, 0, plain);
    off.controller.pointerUp();
    expect(off.red()).toEqual([[1, 1]]);
  });

  // Checks the shapes and the line: a line from (0,0) to (2,0) is mirrored,
  // while the preview moves (the old preview is taken back, mirrored too).
  it("mirrors a line and keeps one commit while the preview moves", () => {
    const { controller, commits, red } = setup("vertical", "line");
    controller.pointerDown(0, 0, 0, plain);
    controller.pointerMove(5, 0, plain);
    controller.pointerMove(2, 0, plain);
    controller.pointerUp();
    expect(red()).toEqual([
      [0, 0],
      [1, 0],
      [2, 0],
      [0, 7],
      [1, 7],
      [2, 7],
    ]);
    expect(commits).toHaveLength(1);
  });

  // Checks that rectangle and ellipse are mirrored too.
  it("mirrors a rectangle and an ellipse", () => {
    for (const tool of ["rectangle", "ellipse"] as const) {
      const { controller, red, at } = setup("horizontal", tool);
      controller.pointerDown(0, 0, 0, plain);
      controller.pointerMove(2, 2, plain);
      controller.pointerUp();
      for (const [x, y] of red()) {
        expect(at(7 - x, y)).toBe("R");
      }
      expect(red().some(([x]) => x > 4)).toBe(true);
    }
  });

  // Checks the bucket: a click fills its area and also the mirrored area.
  it("fills the mirrored area with the bucket", () => {
    const { controller, commits, red } = setup("horizontal", "bucket", 8, 2);
    // A wall of red at x 3 and x 4 splits the sprite in two halves.
    controller.tool = "pencil";
    controller.symmetry = "off";
    controller.pointerDown(3, 0, 0, plain);
    controller.pointerMove(3, 1, plain);
    controller.pointerUp();
    controller.pointerDown(4, 0, 0, plain);
    controller.pointerMove(4, 1, plain);
    controller.pointerUp();
    commits.length = 0;
    controller.tool = "bucket";
    controller.symmetry = "horizontal";
    controller.main = "#FF0000";
    // The left half is empty: the click fills it and its mirror, the right half.
    controller.pointerDown(0, 0, 0, plain);
    expect(red()).toHaveLength(16);
    expect(commits).toHaveLength(1);
  });

  // Checks that the Move tool is not mirrored.
  it("does not mirror the Move tool", () => {
    const { controller, red } = setup("both", "pencil");
    controller.symmetry = "off";
    controller.pointerDown(1, 1, 0, plain);
    controller.pointerUp();
    controller.symmetry = "both";
    controller.tool = "move";
    controller.pointerDown(1, 1, 0, plain);
    controller.pointerMove(2, 1, plain);
    controller.pointerUp();
    expect(red()).toEqual([[2, 1]]);
  });
});
