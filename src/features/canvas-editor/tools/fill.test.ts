import { describe, expect, it } from "vitest";
import { Draft } from "./draft";
import { floodFill } from "./fill";

// Builds pixels from rows of characters.
function rows(lines: string[]) {
  const pixels = new Uint8Array(
    [...lines.join("")].map((c) => c.charCodeAt(0)),
  );
  return { pixels, width: lines[0].length, height: lines.length };
}

function fill(lines: string[], x: number, y: number, code: string) {
  const { pixels, width, height } = rows(lines);
  const draft = new Draft(pixels, width, height);
  floodFill(draft, pixels, x, y, code.charCodeAt(0));
  const out: string[] = [];
  for (let row = 0; row < height; row++) {
    out.push(
      String.fromCharCode(...pixels.slice(row * width, (row + 1) * width)),
    );
  }
  return { out, changes: draft.changes() };
}

// A fill of a whole 512x512 frame takes some tens of milliseconds. The limit
// is loose on purpose, because the test files run at the same time: it only
// catches a fill that has become slow in a bad way (for example one that
// scans the frame again for each pixel).
const FILL_LIMIT_MS = 5000;

describe("floodFill", () => {
  // Checks the "Fill an outline" scenario: filling inside a closed outline
  // changes only the connected pixels inside; the outside stays.
  it("fills only the inside of a closed outline", () => {
    const { out, changes } = fill(
      ["......", ".KKKK.", ".K..K.", ".K..K.", ".KKKK.", "......"],
      2,
      2,
      "R",
    );
    expect(out).toEqual([
      "......",
      ".KKKK.",
      ".KRRK.",
      ".KRRK.",
      ".KKKK.",
      "......",
    ]);
    expect(changes).toHaveLength(4);
  });

  // Checks that a diagonal gap does not leak: connection is by edges, not by
  // corners, so a diagonal outline holds the fill.
  it("does not cross a diagonal line", () => {
    const { out } = fill(["..K", ".K.", "K.."], 0, 0, "R");
    expect(out).toEqual(["RRK", "RK.", "K.."]);
  });

  // Checks that a click on an area that already has the color changes
  // nothing.
  it("changes nothing when the area already has that color", () => {
    const { out, changes } = fill(["RR", "R."], 0, 0, "R");
    expect(out).toEqual(["RR", "R."]);
    expect(changes).toEqual([]);
  });

  // Checks that the fill stops at other colors, not only at transparency, and
  // that a click outside the sprite does nothing.
  it("stops at pixels of other colors", () => {
    expect(fill(["RRBRR"], 0, 0, "G").out).toEqual(["GGBRR"]);
    expect(fill(["RRBRR"], 9, 0, "G").changes).toEqual([]);
  });

  // Checks a winding corridor (a spiral) that needs many row seeds: every
  // pixel of the corridor is filled and the walls stay.
  it("fills a winding area", () => {
    const maze = [
      "KKKKKKK",
      "K.....K",
      "K.KKK.K",
      "K.K...K",
      "K.KKKKK",
      "K......",
      "KKKKKKK",
    ];
    const { out, changes } = fill(maze, 1, 1, "R");
    expect(out).toEqual(maze.map((row) => row.replace(/\./g, "R")));
    expect(changes).toHaveLength(maze.join("").split(".").length - 1);
  });

  // Checks the speed budget for a large sprite: filling all of an empty
  // 512x512 frame (262,144 pixels) and a fine checkerboard (worst case for
  // seeds) both finish quickly and without a stack overflow.
  it("fills a 512x512 frame in a short time", () => {
    const size = 512;
    const empty = new Uint8Array(size * size).fill(46);
    let draft = new Draft(empty, size, size);
    let start = performance.now();
    floodFill(draft, empty, 0, 0, 82);
    expect(performance.now() - start).toBeLessThan(FILL_LIMIT_MS);
    expect(draft.changes()).toHaveLength(size * size);

    const checker = new Uint8Array(size * size);
    for (let i = 0; i < checker.length; i++) {
      checker[i] = ((i % size) + Math.floor(i / size)) % 2 === 0 ? 46 : 75;
    }
    draft = new Draft(checker, size, size);
    start = performance.now();
    floodFill(draft, checker, 0, 0, 82);
    expect(performance.now() - start).toBeLessThan(FILL_LIMIT_MS);
    expect(draft.changes()).toHaveLength(1);
  });
});
