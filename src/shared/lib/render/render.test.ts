// @vitest-environment jsdom
import { describe, expect, it, vi } from "vitest";
import type { FrameData } from "@/shared/lib/core";
import {
  FrameCache,
  drawFrame,
  frameToRgba,
  integerScale,
  parseHex,
  scaleRgba,
} from ".";

const frame = (rows: string[], revision = 1): FrameData => ({
  revision,
  width: rows[0].length,
  height: rows.length,
  rows,
  colors: [
    { key: "R", hex: "#F04545" },
    { key: "N", hex: "#000000" },
  ],
});

describe("frameToRgba", () => {
  // Checks that each key becomes its palette color and "." stays transparent.
  it("paints keys with their color and leaves '.' transparent", () => {
    const rgba = frameToRgba(frame(["R."]));

    expect(Array.from(rgba)).toEqual([240, 69, 69, 255, 0, 0, 0, 0]);
  });

  // Checks that a key that is not in the palette is drawn transparent, not as a crash.
  it("treats an unknown key as transparent", () => {
    expect(Array.from(frameToRgba(frame(["Z"])))).toEqual([0, 0, 0, 0]);
  });

  // Checks the hex reader: valid colors are read, anything else gives null.
  it("reads #RRGGBB colors only", () => {
    expect(parseHex("#0a0B0c")).toEqual([10, 11, 12]);
    expect(parseHex("red")).toBeNull();
    expect(parseHex("#12345")).toBeNull();
  });
});

describe("integer scaling", () => {
  // Checks that the scale is a whole number that fits the box, never below 1.
  it("picks the biggest whole scale that fits", () => {
    expect(integerScale(16, 16, 64)).toBe(4);
    expect(integerScale(30, 10, 100)).toBe(3);
    expect(integerScale(512, 512, 64)).toBe(1);
  });

  // Checks that every pixel is repeated scale x scale times (no blur).
  it("repeats each pixel in both directions", () => {
    const one = new Uint8ClampedArray([1, 2, 3, 4, 5, 6, 7, 8]); // 2x1
    const out = scaleRgba(one, 2, 1, 2); // 4x2

    expect(out.length).toBe(4 * 2 * 4);
    const pixel = (x: number, y: number) =>
      Array.from(out.subarray((y * 4 + x) * 4, (y * 4 + x) * 4 + 4));
    expect(pixel(0, 0)).toEqual([1, 2, 3, 4]);
    expect(pixel(1, 1)).toEqual([1, 2, 3, 4]);
    expect(pixel(2, 0)).toEqual([5, 6, 7, 8]);
    expect(pixel(3, 1)).toEqual([5, 6, 7, 8]);
  });
});

describe("drawFrame", () => {
  // Checks that the canvas takes the scaled size and gets the pixels. jsdom
  // has no canvas, so a small fake context records what was drawn.
  it("sizes the canvas and puts the scaled pixels", () => {
    const putImageData = vi.fn();
    const canvas = document.createElement("canvas");
    canvas.getContext = (() => ({ putImageData })) as never;
    vi.stubGlobal(
      "ImageData",
      class {
        constructor(
          public data: Uint8ClampedArray,
          public width: number,
          public height: number,
        ) {}
      },
    );

    const scale = drawFrame(canvas, frame(["R.", ".N"]), 8);

    expect(scale).toBe(4);
    expect([canvas.width, canvas.height]).toEqual([8, 8]);
    expect(putImageData).toHaveBeenCalledOnce();
    vi.unstubAllGlobals();
  });
});

describe("FrameCache", () => {
  // Checks that asking twice for the same frame calls the core once.
  it("loads a frame once", async () => {
    const load = vi.fn(async () => frame(["R"]));
    const cache = new FrameCache(load);

    await Promise.all([cache.get(["A"], "goblin"), cache.get(["A"], "goblin")]);

    expect(load).toHaveBeenCalledTimes(1);
  });

  // Checks the spec idea "the thumbnail updates when a sprite changes": a
  // change that touches the sprite makes the next ask load it again, and a
  // sprite that was not touched keeps its frame.
  it("loads a touched sprite again and keeps the others", async () => {
    let revision = 1;
    const load = vi.fn(async () => frame(["R"], revision));
    const cache = new FrameCache(load);
    await cache.get(["A"], "goblin");
    await cache.get(["A"], "orc");
    revision = 2;

    const forgot = cache.invalidate([
      { kind: "sprite", path: ["A", "goblin"] },
    ]);

    expect(forgot).toBe(true);
    expect((await cache.get(["A"], "goblin")).revision).toBe(2);
    expect((await cache.get(["A"], "orc")).revision).toBe(1);
  });

  // Checks that touching a folder forgets the sprites inside it, at any depth.
  it("forgets the sprites inside a touched folder", async () => {
    const load = vi.fn(async () => frame(["R"]));
    const cache = new FrameCache(load);
    await cache.get(["A", "B"], "deep");
    await cache.get(["C"], "other");

    cache.invalidate([{ kind: "folder", path: ["A"] }]);
    await cache.get(["A", "B"], "deep");
    await cache.get(["C"], "other");

    expect(load).toHaveBeenCalledTimes(3);
  });

  // Checks that a failed call is not kept, so the next ask tries again.
  it("does not keep a failed load", async () => {
    const load = vi
      .fn<() => Promise<FrameData>>()
      .mockRejectedValueOnce({ code: "io" })
      .mockResolvedValue(frame(["R"]));
    const cache = new FrameCache(load);

    await expect(cache.get([], "a")).rejects.toBeDefined();
    await vi.waitFor(async () => {
      expect((await cache.get([], "a")).rows).toEqual(["R"]);
    });
  });
});
