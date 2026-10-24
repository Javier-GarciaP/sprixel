import { describe, expect, it } from "vitest";
import {
  MAX_ZOOM,
  MIN_ZOOM,
  centeredView,
  fitZoom,
  pixelAt,
  stepZoom,
  zoomAt,
} from "./viewMath";

describe("stepZoom", () => {
  // Checks that zoom moves one whole step at a time.
  it("goes up and down by one whole step", () => {
    expect(stepZoom(4, 1)).toBe(5);
    expect(stepZoom(4, -1)).toBe(3);
  });

  // Checks the limits of the spec: 100% (1) to 3200% (32).
  it("stays between 100% and 3200%", () => {
    expect(MIN_ZOOM).toBe(1);
    expect(MAX_ZOOM).toBe(32);
    expect(stepZoom(1, -1)).toBe(1);
    expect(stepZoom(32, 1)).toBe(32);
  });
});

describe("fitZoom", () => {
  // Checks the "Fit" scenario: a 120x30 sprite in a 500x300 area is scaled by
  // the largest whole zoom that fits (4: 480 wide, and 10 would be 1200).
  it("uses the largest whole zoom that fits the area", () => {
    expect(fitZoom(120, 30, 500, 300)).toBe(4);
    expect(fitZoom(16, 16, 640, 480)).toBe(30);
  });

  // Checks that a sprite bigger than the area still gets 100%, and a tiny
  // sprite is limited to 3200%.
  it("never goes outside 100% to 3200%", () => {
    expect(fitZoom(512, 512, 300, 300)).toBe(1);
    expect(fitZoom(2, 2, 4000, 4000)).toBe(32);
  });
});

describe("centeredView", () => {
  // Checks that the sprite is centered with whole screen pixels.
  it("centers the sprite in the area", () => {
    expect(centeredView(16, 16, 8, 200, 150)).toEqual({
      zoom: 8,
      x: 36,
      y: 11,
    });
  });
});

describe("zoomAt", () => {
  // Checks the "Zoom at the pointer" scenario for many pointer positions and
  // zoom changes: the sprite pixel under the pointer is the same before and
  // after.
  it("keeps the pixel under the pointer", () => {
    const view = { zoom: 5, x: 13, y: -7 };
    for (let px = 0; px < 300; px += 7) {
      for (let py = 0; py < 200; py += 11) {
        for (const next of [1, 2, 6, 9, 16, 32]) {
          const pointer = { x: px, y: py };
          const before = pixelAt(view, pointer);
          const after = pixelAt(zoomAt(view, next, pointer), pointer);
          expect(after).toEqual(before);
        }
      }
    }
  });

  // Checks that the zoom of the new view is the one asked for.
  it("sets the new zoom", () => {
    expect(zoomAt({ zoom: 2, x: 0, y: 0 }, 6, { x: 10, y: 10 }).zoom).toBe(6);
  });
});

describe("pixelAt", () => {
  // Checks the pixel math: with zoom 8 and the sprite at (10, 20), the screen
  // point (10 + 3*8, 20 + 4*8 + 7) is the pixel (3, 4).
  it("finds the sprite pixel under a screen point", () => {
    const view = { zoom: 8, x: 10, y: 20 };
    expect(pixelAt(view, { x: 34, y: 59 })).toEqual({ x: 3, y: 4 });
    expect(pixelAt(view, { x: 9, y: 20 })).toEqual({ x: -1, y: 0 });
  });
});
