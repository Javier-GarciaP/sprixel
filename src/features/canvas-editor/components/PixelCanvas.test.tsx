// @vitest-environment jsdom
import { cleanup, render, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { buildColorTable } from "../render/colors";
import type { Scene } from "../render/renderer";
import { PixelCanvas } from "./PixelCanvas";

const putImageData = vi.fn();

beforeEach(() => {
  putImageData.mockClear();
  // jsdom has no canvas: a small fake that records what is sent to it.
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
  vi.spyOn(HTMLCanvasElement.prototype, "getContext").mockReturnValue({
    putImageData,
  } as unknown as CanvasRenderingContext2D);
  vi.spyOn(HTMLElement.prototype, "clientWidth", "get").mockReturnValue(64);
  vi.spyOn(HTMLElement.prototype, "clientHeight", "get").mockReturnValue(64);
});

afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
  vi.unstubAllGlobals();
});

const scene: Scene = {
  width: 2,
  height: 2,
  pixels: new Uint8Array([82, 82, 82, 82]),
  colors: buildColorTable([{ key: "R", hex: "#FF0000" }]),
  onion: null,
  grid: false,
  selection: null,
};

describe("PixelCanvas", () => {
  // Checks that the canvas takes the size of its area and paints the scene
  // there: the first pixel of the sprite on the screen is the red of "R".
  it("paints the scene into a canvas the size of its area", async () => {
    const { container } = render(
      <PixelCanvas scene={scene} view={{ zoom: 8, x: 4, y: 4 }} />,
    );
    const canvas = container.querySelector("canvas")!;
    await waitFor(() => expect(putImageData).toHaveBeenCalled());
    expect(canvas.width).toBe(64);
    expect(canvas.height).toBe(64);
    const image = putImageData.mock.calls[
      putImageData.mock.calls.length - 1
    ][0] as {
      data: Uint8ClampedArray;
    };
    const o = (4 * 64 + 4) * 4;
    expect([...image.data.slice(o, o + 4)]).toEqual([255, 0, 0, 255]);
  });
});
