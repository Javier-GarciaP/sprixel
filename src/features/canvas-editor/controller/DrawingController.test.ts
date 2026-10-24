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
  const picks: [string, number][] = [];
  const colors = new ColorBook([
    { key: "R", hex: "#FF0000" },
    { key: "B", hex: "#0000FF" },
  ]);
  const controller = new DrawingController(width, height, pixels, colors, {
    onDirty: (area) => dirty.push(area),
    onCommit: (changes) => commits.push(changes),
    onPick: (hex, button) => picks.push([hex, button]),
  });
  controller.main = "#FF0000";
  controller.secondary = "#0000FF";
  const at = (x: number, y: number) =>
    String.fromCharCode(pixels[y * width + x]);
  return { controller, pixels, commits, dirty, picks, colors, at };
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

describe("Colors", () => {
  // Checks the "Right click" scenario: drawing with the right button uses the
  // secondary color, the left button the main color.
  it("draws with the secondary color on the right button", () => {
    const { controller, commits, at } = setup();
    controller.pointerDown(1, 1, 2, plain);
    controller.pointerUp();
    controller.pointerDown(2, 1, 0, plain);
    controller.pointerUp();
    expect(at(1, 1)).toBe("B");
    expect(at(2, 1)).toBe("R");
    expect(commits[0]).toEqual([{ x: 1, y: 1, color: "#0000FF" }]);
  });

  // Checks that a color is found by value whatever the letter case, so
  // "#ff0000" draws with the key R of the palette.
  it("matches colors ignoring the letter case", () => {
    const { controller, at } = setup();
    controller.main = "#ff0000";
    controller.pointerDown(0, 0, 0, plain);
    controller.pointerUp();
    expect(at(0, 0)).toBe("R");
  });

  // Checks that the Eraser ignores the colors, also with the right button.
  it("erases with the right button too", () => {
    const { controller, at } = setup();
    controller.pointerDown(0, 0, 0, plain);
    controller.pointerUp();
    controller.tool = "eraser";
    controller.pointerDown(0, 0, 2, plain);
    controller.pointerUp();
    expect(at(0, 0)).toBe(".");
  });
});

describe("Eyedropper", () => {
  // Checks the "Pick a color" scenario: clicking a red pixel reports that red,
  // and nothing is drawn.
  it("picks the color under the pointer", () => {
    const { controller, picks, commits } = setup();
    controller.pointerDown(3, 3, 0, plain);
    controller.pointerUp();
    controller.tool = "eyedropper";
    controller.pointerDown(3, 3, 0, plain);
    controller.pointerUp();
    expect(picks).toEqual([["#FF0000", 0]]);
    expect(commits).toHaveLength(1);
  });

  // Checks that the right button picks the secondary color.
  it("reports the right button", () => {
    const { controller, picks } = setup();
    controller.pointerDown(3, 3, 0, plain);
    controller.pointerUp();
    controller.tool = "eyedropper";
    controller.pointerDown(3, 3, 2, plain);
    expect(picks).toEqual([["#FF0000", 2]]);
  });

  // Checks that a transparent pixel, or a click outside the sprite, leaves
  // the colors unchanged.
  it("does nothing on a transparent pixel or outside the sprite", () => {
    const { controller, picks } = setup();
    controller.tool = "eyedropper";
    controller.pointerDown(3, 3, 0, plain);
    controller.pointerDown(-1, 3, 0, plain);
    controller.pointerDown(3, 99, 0, plain);
    expect(picks).toEqual([]);
  });
});

describe("Bucket", () => {
  // Draws a closed 4x4 outline of blue in an 8x8 sprite, starting at (2, 2).
  function outline() {
    const made = setup();
    made.controller.main = "#0000FF";
    for (const [x, y] of [
      [2, 2],
      [3, 2],
      [4, 2],
      [5, 2],
      [2, 5],
      [3, 5],
      [4, 5],
      [5, 5],
      [2, 3],
      [2, 4],
      [5, 3],
      [5, 4],
    ]) {
      made.controller.pointerDown(x, y, 0, plain);
      made.controller.pointerUp();
    }
    made.commits.length = 0;
    made.controller.main = "#FF0000";
    made.controller.tool = "bucket";
    return made;
  }

  // Checks the "Fill an outline" scenario: filling inside a closed outline
  // changes only the 4 connected pixels inside, in one commit.
  it("fills only the connected pixels inside an outline", () => {
    const { controller, commits, pixels } = outline();
    controller.pointerDown(3, 3, 0, plain);
    controller.pointerUp();
    expect(commits).toHaveLength(1);
    expect(commits[0].map((c) => [c.x, c.y]).sort()).toEqual(
      [
        [3, 3],
        [4, 3],
        [3, 4],
        [4, 4],
      ].sort(),
    );
    expect(pixels.filter((c) => c === code("R"))).toHaveLength(4);
    expect(pixels[0]).toBe(46);
  });

  // Checks that clicking an area that is already the active color changes
  // nothing and sends nothing.
  it("changes nothing on an area that already has the color", () => {
    const { controller, commits } = outline();
    controller.pointerDown(3, 3, 0, plain);
    controller.pointerUp();
    commits.length = 0;
    controller.pointerDown(3, 3, 0, plain);
    controller.pointerUp();
    expect(commits).toEqual([]);
  });

  // Checks that the right button fills with the secondary color.
  it("fills with the secondary color on the right button", () => {
    const { controller, at } = outline();
    controller.pointerDown(3, 3, 2, plain);
    controller.pointerUp();
    expect(at(3, 3)).toBe("B");
  });

  // Checks the speed budget on a 512x512 sprite: a fill of the whole frame
  // is one commit of 262,144 pixels and takes well under half a second.
  it("fills a 512x512 sprite in a short time", () => {
    const { controller, commits } = setup(512, 512);
    controller.tool = "bucket";
    const start = performance.now();
    controller.pointerDown(100, 100, 0, plain);
    controller.pointerUp();
    expect(performance.now() - start).toBeLessThan(500);
    expect(commits[0]).toHaveLength(512 * 512);
  });
});

describe("Shapes", () => {
  const shape = (
    tool: "line" | "rectangle" | "ellipse",
    from: [number, number],
    to: [number, number],
    shift = false,
  ) => {
    const made = setup(16, 16);
    made.controller.tool = tool;
    made.controller.pointerDown(from[0], from[1], 0, { shift });
    made.controller.pointerMove(to[0], to[1], { shift });
    made.controller.pointerUp();
    return made;
  };
  const painted = (pixels: Uint8Array) => {
    const list: string[] = [];
    pixels.forEach((c, i) => {
      if (c !== 46) list.push(`${i % 16},${Math.floor(i / 16)}`);
    });
    return list;
  };

  // Checks that a line goes from the press to the release with no gaps, in
  // one commit.
  it("draws a line on release", () => {
    const { commits, pixels } = shape("line", [1, 1], [8, 4]);
    expect(commits).toHaveLength(1);
    expect(painted(pixels)).toContain("1,1");
    expect(painted(pixels)).toContain("8,4");
    expect(painted(pixels)).toHaveLength(8);
  });

  // Checks the live preview: while the pointer moves, the pixels follow it and
  // the pixels of the old position are taken back; nothing is sent until the
  // release.
  it("previews the shape while dragging and sends it once", () => {
    const { controller, pixels, commits } = setup(16, 16);
    controller.tool = "rectangle";
    controller.pointerDown(2, 2, 0, plain);
    controller.pointerMove(10, 10, plain);
    expect(painted(pixels)).toContain("10,10");
    expect(commits).toEqual([]);
    controller.pointerMove(4, 4, plain);
    expect(painted(pixels)).not.toContain("10,10");
    expect(painted(pixels)).toHaveLength(8);
    controller.pointerUp();
    expect(commits).toHaveLength(1);
    expect(commits[0]).toHaveLength(8);
  });

  // Checks that a shape that ends where it started (or goes back to its
  // first pixel) is a single pixel, and that releasing outside works.
  it("clips a shape that leaves the sprite", () => {
    const { commits } = shape("rectangle", [12, 12], [30, 30]);
    expect(commits[0].every((c) => c.x < 16 && c.y < 16)).toBe(true);
  });

  // Checks the outline and fill options of the Rectangle: a 5x4 outline has
  // 14 pixels and a filled one has 20.
  it("draws an outline or a filled rectangle", () => {
    expect(painted(shape("rectangle", [1, 1], [5, 4]).pixels)).toHaveLength(14);
    const { controller, pixels } = setup(16, 16);
    controller.tool = "rectangle";
    controller.filled = true;
    controller.pointerDown(1, 1, 0, plain);
    controller.pointerMove(5, 4, plain);
    controller.pointerUp();
    expect(painted(pixels)).toHaveLength(20);
  });

  // Checks the "Square with Shift" scenario: dragging a rectangle from (2, 2)
  // to (9, 5) while holding Shift gives a square of 8 x 8 (28 outline pixels).
  it("makes a square from a rectangle with Shift", () => {
    const { pixels } = shape("rectangle", [2, 2], [9, 5], true);
    const xs = painted(pixels).map((p) => Number(p.split(",")[0]));
    const ys = painted(pixels).map((p) => Number(p.split(",")[1]));
    expect(Math.max(...xs) - Math.min(...xs)).toBe(7);
    expect(Math.max(...ys) - Math.min(...ys)).toBe(7);
    expect(painted(pixels)).toHaveLength(28);
  });

  // Checks Shift on an ellipse: the box becomes a square, so the circle is as
  // tall as it is wide.
  it("makes a circle from an ellipse with Shift", () => {
    const { pixels } = shape("ellipse", [2, 2], [11, 5], true);
    const xs = painted(pixels).map((p) => Number(p.split(",")[0]));
    const ys = painted(pixels).map((p) => Number(p.split(",")[1]));
    expect(Math.max(...xs) - Math.min(...xs)).toBe(
      Math.max(...ys) - Math.min(...ys),
    );
  });

  // Checks Shift on a line: a nearly horizontal drag becomes exactly
  // horizontal, and a nearly diagonal one becomes a 45-degree line.
  it("snaps a line to 45-degree steps with Shift", () => {
    const horizontal = painted(shape("line", [2, 5], [12, 6], true).pixels);
    expect(horizontal.every((p) => p.endsWith(",5"))).toBe(true);
    const diagonal = painted(shape("line", [2, 2], [10, 9], true).pixels);
    expect(diagonal).toContain("9,9");
  });

  // Checks that every shape tool uses the secondary color on the right button.
  it("uses the secondary color on the right button", () => {
    const { controller, at } = setup(16, 16);
    controller.tool = "line";
    controller.pointerDown(1, 1, 2, plain);
    controller.pointerMove(4, 1, plain);
    controller.pointerUp();
    expect(at(2, 1)).toBe("B");
  });
});

describe("Select and Move", () => {
  // Draws a red pixel at each given place.
  function withPixels(places: [number, number][], size = 12) {
    const made = setup(size, size);
    for (const [x, y] of places) {
      made.controller.pointerDown(x, y, 0, plain);
      made.controller.pointerUp();
    }
    made.commits.length = 0;
    return made;
  }
  const select = (
    controller: ReturnType<typeof setup>["controller"],
    from: [number, number],
    to: [number, number],
  ) => {
    controller.tool = "select";
    controller.pointerDown(from[0], from[1], 0, plain);
    controller.pointerMove(to[0], to[1], plain);
    controller.pointerUp();
  };
  const drag = (
    controller: ReturnType<typeof setup>["controller"],
    from: [number, number],
    to: [number, number],
  ) => {
    controller.tool = "move";
    controller.pointerDown(from[0], from[1], 0, plain);
    controller.pointerMove(to[0], to[1], plain);
    controller.pointerUp();
  };

  // Checks that Select makes a rectangle between the press and the release
  // (both pixels included), changes no pixels and sends nothing.
  it("selects a rectangle", () => {
    const { controller, commits } = withPixels([]);
    select(controller, [2, 3], [5, 4]);
    expect(controller.selection).toEqual({ x: 2, y: 3, width: 4, height: 2 });
    expect(commits).toEqual([]);
  });

  // Checks that dragging in any direction gives the same rectangle, and that
  // the part outside the sprite is cut off.
  it("selects in any direction and clips to the sprite", () => {
    const { controller } = withPixels([]);
    select(controller, [5, 4], [2, 3]);
    expect(controller.selection).toEqual({ x: 2, y: 3, width: 4, height: 2 });
    select(controller, [10, 10], [20, 30]);
    expect(controller.selection).toEqual({ x: 10, y: 10, width: 2, height: 2 });
  });

  // Checks that a click without a drag clears the selection.
  it("clears the selection with a click", () => {
    const { controller } = withPixels([]);
    select(controller, [2, 2], [4, 4]);
    controller.pointerDown(8, 8, 0, plain);
    controller.pointerUp();
    expect(controller.selection).toBeNull();
  });

  // Checks the "Move a selection" scenario: a selected area dragged 3 pixels
  // to the right moves its pixels 3 pixels, the old place becomes transparent,
  // the colors are the same, and it is one commit.
  it("moves the selected pixels", () => {
    const { controller, commits, at } = withPixels([
      [2, 2],
      [3, 2],
      [3, 3],
    ]);
    select(controller, [2, 2], [3, 3]);
    drag(controller, [2, 2], [5, 2]);
    expect(at(2, 2)).toBe(".");
    expect(at(3, 2)).toBe(".");
    expect(at(3, 3)).toBe(".");
    expect(at(5, 2)).toBe("R");
    expect(at(6, 2)).toBe("R");
    expect(at(6, 3)).toBe("R");
    expect(commits).toHaveLength(1);
    expect(controller.selection).toEqual({ x: 5, y: 2, width: 2, height: 2 });
  });

  // Checks that pixels outside the selection stay where they are.
  it("leaves the pixels outside the selection alone", () => {
    const { controller, at } = withPixels([
      [2, 2],
      [9, 9],
    ]);
    select(controller, [1, 1], [3, 3]);
    drag(controller, [2, 2], [2, 4]);
    expect(at(9, 9)).toBe("R");
    expect(at(2, 4)).toBe("R");
    expect(at(2, 2)).toBe(".");
  });

  // Checks the clipping rule: what leaves the canvas is lost, and what stays
  // inside keeps its color.
  it("cuts off what leaves the canvas", () => {
    const { controller, at, commits } = withPixels([
      [10, 5],
      [11, 5],
    ]);
    select(controller, [10, 5], [11, 5]);
    drag(controller, [10, 5], [11, 5]);
    expect(at(10, 5)).toBe(".");
    expect(at(11, 5)).toBe("R");
    // (11, 5) was red and is red again, so only (10, 5) changed.
    expect(commits[0]).toEqual([{ x: 10, y: 5, color: null }]);
  });

  // Checks that without a selection the Move tool moves the whole frame.
  it("moves the whole frame when nothing is selected", () => {
    const { controller, at } = withPixels([
      [0, 0],
      [4, 4],
    ]);
    drag(controller, [0, 0], [2, 1]);
    expect(at(0, 0)).toBe(".");
    expect(at(2, 1)).toBe("R");
    expect(at(6, 5)).toBe("R");
  });

  // Checks that a moved piece does not erase what is under its transparent
  // pixels: only its drawn pixels replace the destination.
  it("does not erase with the transparent pixels of the moved piece", () => {
    const { controller, at } = withPixels([
      [1, 1],
      [4, 1],
    ]);
    select(controller, [0, 1], [1, 1]);
    drag(controller, [0, 1], [3, 1]);
    expect(at(4, 1)).toBe("R");
    expect(at(1, 1)).toBe(".");
  });

  // Checks that a move that comes back to the start sends nothing, and that a
  // press without a move sends nothing.
  it("sends nothing when the pixels end where they started", () => {
    const { controller, commits } = withPixels([[2, 2]]);
    select(controller, [1, 1], [3, 3]);
    drag(controller, [2, 2], [5, 5]);
    commits.length = 0;
    controller.tool = "move";
    controller.pointerDown(5, 5, 0, plain);
    controller.pointerMove(9, 9, plain);
    controller.pointerMove(5, 5, plain);
    controller.pointerUp();
    expect(commits).toEqual([]);
  });
});
