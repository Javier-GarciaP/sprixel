import { describe, expect, it } from "vitest";
import { TRANSPARENT_CODE } from "../types";
import { buildColorTable } from "./colors";
import {
  CHECKER_DARK,
  CHECKER_LIGHT,
  renderScene,
  type Scene,
  type View,
} from "./renderer";

const code = (key: string) => key.charCodeAt(0);

// A scene from rows of keys, for example ["R.", ".B"].
function scene(rows: string[], extra: Partial<Scene> = {}): Scene {
  const width = rows[0].length;
  const pixels = new Uint8Array([...rows.join("")].map((key) => code(key)));
  return {
    width,
    height: rows.length,
    pixels,
    colors: buildColorTable([
      { key: "R", hex: "#FF0000" },
      { key: "B", hex: "#0000FF" },
      { key: "K", hex: "#000000" },
      { key: "W", hex: "#FFFFFF" },
    ]),
    onion: null,
    grid: false,
    selection: null,
    ...extra,
  };
}

function paint(s: Scene, view: View, size = { width: 40, height: 40 }) {
  const buffer = new Uint8ClampedArray(size.width * size.height * 4);
  renderScene(buffer, size.width, size.height, s, view);
  const at = (x: number, y: number) => {
    const o = (y * size.width + x) * 4;
    return [buffer[o], buffer[o + 1], buffer[o + 2], buffer[o + 3]];
  };
  return { buffer, at };
}

describe("renderScene", () => {
  // Checks the "Zoomed in" scenario: at 800% every sprite pixel is an 8x8
  // block of one color, and the next block starts with no blending.
  it("draws each pixel as a sharp block of zoom x zoom pixels", () => {
    const { at } = paint(scene(["RB"]), { zoom: 8, x: 0, y: 0 });
    for (let y = 0; y < 8; y++) {
      for (let x = 0; x < 8; x++) {
        expect(at(x, y)).toEqual([255, 0, 0, 255]);
        expect(at(x + 8, y)).toEqual([0, 0, 255, 255]);
      }
    }
    // Just outside the sprite nothing is painted.
    expect(at(16, 0)[3]).toBe(0);
    expect(at(0, 8)[3]).toBe(0);
  });

  // Checks that the view position moves the sprite on the screen and that a
  // sprite that is partly outside the viewport is clipped, not wrapped.
  it("places the sprite at the view position and clips it", () => {
    const { at } = paint(
      scene(["RB"]),
      { zoom: 4, x: -2, y: 3 },
      {
        width: 10,
        height: 10,
      },
    );
    expect(at(0, 3)).toEqual([255, 0, 0, 255]);
    expect(at(1, 3)).toEqual([255, 0, 0, 255]);
    expect(at(2, 3)).toEqual([0, 0, 255, 255]);
    expect(at(0, 2)[3]).toBe(0);
  });

  // Checks the transparent background: transparent pixels show a gray
  // checkerboard of two grays.
  it("shows transparent pixels over a gray checkerboard", () => {
    const { at } = paint(
      scene(["."]),
      { zoom: 16, x: 0, y: 0 },
      {
        width: 16,
        height: 16,
      },
    );
    expect(at(0, 0).slice(0, 3)).toEqual([...CHECKER_LIGHT]);
    expect(at(8, 0).slice(0, 3)).toEqual([...CHECKER_DARK]);
    expect(at(8, 8).slice(0, 3)).toEqual([...CHECKER_LIGHT]);
  });

  // Checks the "Black outline in the dark theme" scenario. The renderer has
  // no theme input, so the picture is the same in both themes; here we check
  // that black and white are clearly different from both checker grays.
  it("keeps black and white pixels visible on the checkerboard", () => {
    const contrast = (a: readonly number[], b: readonly number[]) =>
      Math.abs(a[0] - b[0]);
    for (const gray of [CHECKER_LIGHT, CHECKER_DARK]) {
      expect(contrast(gray, [0, 0, 0])).toBeGreaterThan(100);
      expect(contrast(gray, [255, 255, 255])).toBeGreaterThan(60);
    }
    const { at } = paint(scene(["K."]), { zoom: 8, x: 0, y: 0 });
    expect(at(0, 0).slice(0, 3)).toEqual([0, 0, 0]);
  });

  // Checks that the checkerboard is the same for any view position inside the
  // sprite, so it does not depend on a theme or on where the sprite is.
  it("draws the same checkerboard wherever the sprite is", () => {
    const a = paint(
      scene(["...."]),
      { zoom: 8, x: 0, y: 0 },
      { width: 80, height: 20 },
    );
    const b = paint(
      scene(["...."]),
      { zoom: 8, x: 5, y: 5 },
      { width: 80, height: 20 },
    );
    expect(b.at(5, 5)).toEqual(a.at(0, 0));
    expect(b.at(13, 5)).toEqual(a.at(8, 0));
  });

  // Checks that painting only a dirty area changes only that area.
  it("repaints only the dirty area when one is given", () => {
    const s = scene(["RR", "RR"]);
    const view = { zoom: 4, x: 0, y: 0 };
    const buffer = new Uint8ClampedArray(8 * 8 * 4);
    renderScene(buffer, 8, 8, s, view);
    s.pixels[3] = code("B"); // bottom right
    s.pixels[0] = code("B"); // top left, not marked dirty
    renderScene(buffer, 8, 8, s, view, { x: 1, y: 1, width: 1, height: 1 });
    const at = (x: number, y: number) => buffer[(y * 8 + x) * 4 + 2];
    expect(at(5, 5)).toBe(255); // blue: repainted
    expect(at(0, 0)).toBe(0); // still red: not repainted
  });
});

describe("renderScene onion skin", () => {
  // Checks that the previous frame shows in blue where the frame is
  // transparent, at the chosen opacity, and never over a drawn pixel.
  it("tints the previous frame blue under the drawing", () => {
    const s = scene(["R.", ".."], {
      onion: {
        pixels: new Uint8Array([..."RR.."].map(code)),
        opacity: 0.4,
      },
    });
    const { at } = paint(s, { zoom: 8, x: 0, y: 0 }, { width: 16, height: 16 });
    expect(at(0, 0)).toEqual([255, 0, 0, 255]); // drawing wins
    const blue = at(8, 0);
    expect(blue[2]).toBeGreaterThan(blue[0]);
    expect(blue[2]).toBeGreaterThan(blue[1]);
    // Where the previous frame is empty the checkerboard is untouched.
    expect(at(0, 8).slice(0, 3)).toEqual([...CHECKER_DARK]);
  });

  // Checks that a higher opacity is bluer.
  it("uses the opacity", () => {
    const make = (opacity: number) =>
      paint(
        scene(["."], {
          onion: { pixels: new Uint8Array([code("R")]), opacity },
        }),
        { zoom: 1, x: 0, y: 0 },
        { width: 1, height: 1 },
      ).at(0, 0);
    expect(make(0.8)[2]).toBeGreaterThan(make(0.1)[2]);
  });
});

describe("renderScene grid", () => {
  const withGrid = (zoom: number, grid: boolean) =>
    paint(
      scene(["RR"], { grid }),
      { zoom, x: 0, y: 0 },
      {
        width: zoom * 2,
        height: zoom,
      },
    );

  // Checks the "Grid at 800%" scenario: with the option on, lines appear
  // between pixels (the first row and column of each cell).
  it("draws grid lines at 800%", () => {
    const { at } = withGrid(8, true);
    expect(at(0, 4)).not.toEqual([255, 0, 0, 255]);
    expect(at(8, 4)).not.toEqual([255, 0, 0, 255]);
    expect(at(4, 4)).toEqual([255, 0, 0, 255]);
  });

  // Checks the 600% rule: no lines below 600%, lines from 600%.
  it("draws no lines below 600% and draws them from 600%", () => {
    expect(withGrid(5, true).at(0, 2)).toEqual([255, 0, 0, 255]);
    expect(withGrid(6, true).at(0, 2)).not.toEqual([255, 0, 0, 255]);
  });

  // Checks that the grid is off when the option is off.
  it("draws no lines when the option is off", () => {
    expect(withGrid(8, false).at(0, 4)).toEqual([255, 0, 0, 255]);
  });
});

describe("renderScene selection", () => {
  // Checks that the selection gets an outline on the edge of its area only.
  it("outlines the selection", () => {
    const s = scene(["RRR", "RRR", "RRR"], {
      selection: { x: 1, y: 1, width: 1, height: 1 },
    });
    const { at } = paint(s, { zoom: 8, x: 0, y: 0 }, { width: 24, height: 24 });
    const edge = at(8, 8);
    expect(edge[0] === edge[1] && edge[1] === edge[2]).toBe(true);
    expect(at(12, 12)).toEqual([255, 0, 0, 255]);
  });
});

describe("TRANSPARENT_CODE", () => {
  // Checks that a transparent code is the character code of ".".
  it("is the code of the dot", () => {
    expect(TRANSPARENT_CODE).toBe(46);
  });
});
