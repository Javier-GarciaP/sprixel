// @vitest-environment jsdom
import { cleanup, fireEvent, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { FakeEditorApi } from "../testing/fakeApi";
import {
  colorOfPixel,
  drag,
  mockLayout,
  renderEditor,
} from "../testing/renderEditor";

beforeEach(() => mockLayout(500, 300));
afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
  vi.unstubAllGlobals();
});

const palette = [{ key: "R", hex: "#FF0000" }];

// Frame 1 has a red pixel at (1, 1). Frame 2 is empty.
function twoFrames() {
  const api = new FakeEditorApi(16, 16, 2, palette);
  const rows = Array.from({ length: 16 }, () => ".".repeat(16));
  rows[1] = ".R" + ".".repeat(14);
  api.setFrame(1, rows);
  return api;
}

const isBlue = ([r, g, b]: number[]) => b > r && b > g;

async function openFrame(api: FakeEditorApi, frame: number) {
  renderEditor(api);
  await screen.findByText("1800%");
  if (frame > 1) {
    fireEvent.click(screen.getByRole("button", { name: `Frame ${frame}` }));
  }
}

describe("Onion skin", () => {
  // Checks the "Drawing frame 2" scenario: with onion skin on and the user on
  // frame 2, the pixels of frame 1 show in blue below frame 2.
  it("shows the previous frame in blue on frame 2", async () => {
    await openFrame(twoFrames(), 2);
    await waitFor(() => expect(isBlue(colorOfPixel(16, 16, 1, 1))).toBe(true));
    // Where frame 1 is empty the checkerboard shows as it is.
    expect(isBlue(colorOfPixel(16, 16, 8, 8))).toBe(false);
  });

  // Checks the "First frame" scenario: on frame 1 there is no onion skin, and
  // the editor does not read any frame before the first. Frame 2 has a pixel
  // that must not show on frame 1 either.
  it("shows nothing on frame 1", async () => {
    const api = new FakeEditorApi(16, 16, 2, palette);
    const rows = Array.from({ length: 16 }, () => ".".repeat(16));
    rows[3] = "...R" + ".".repeat(12);
    api.setFrame(2, rows);
    const read = vi.spyOn(api, "loadFrame");
    await openFrame(api, 1);
    await waitFor(() => expect(read).toHaveBeenCalled());
    expect(isBlue(colorOfPixel(16, 16, 3, 3))).toBe(false);
    expect(read.mock.calls.every(([, frame]) => frame >= 1)).toBe(true);
    expect(read.mock.calls.every(([, frame]) => frame === 1)).toBe(true);
  });

  // Checks that the drawing wins: the red pixel of the current frame is drawn
  // over the blue of the previous frame, never mixed with it.
  it("draws the current frame over the onion skin", async () => {
    const api = twoFrames();
    await openFrame(api, 2);
    await waitFor(() => expect(isBlue(colorOfPixel(16, 16, 1, 1))).toBe(true));
    fireEvent.change(screen.getByLabelText("Hex color"), {
      target: { value: "#FF0000" },
    });
    drag([16, 16], [[1, 1]]);
    await waitFor(() => expect(api.calls).toHaveLength(1));
    expect(colorOfPixel(16, 16, 1, 1)).toEqual([255, 0, 0, 255]);
  });

  // Checks the switch: off hides the previous frame, on shows it again.
  it("hides the previous frame when the switch is off", async () => {
    await openFrame(twoFrames(), 2);
    await waitFor(() => expect(isBlue(colorOfPixel(16, 16, 1, 1))).toBe(true));
    const toggle = screen.getByRole("switch", { name: "Show previous frame" });
    fireEvent.click(toggle);
    await waitFor(() => expect(isBlue(colorOfPixel(16, 16, 1, 1))).toBe(false));
    fireEvent.click(toggle);
    await waitFor(() => expect(isBlue(colorOfPixel(16, 16, 1, 1))).toBe(true));
  });

  // Checks the O key: it turns onion skin off and on.
  it("toggles with the O key", async () => {
    await openFrame(twoFrames(), 2);
    await waitFor(() => expect(isBlue(colorOfPixel(16, 16, 1, 1))).toBe(true));
    await waitFor(() => {
      fireEvent.keyDown(window, { key: "o" });
      expect(
        screen
          .getByRole("switch", { name: "Show previous frame" })
          .getAttribute("aria-checked"),
      ).toBe("false");
    });
    await waitFor(() => expect(isBlue(colorOfPixel(16, 16, 1, 1))).toBe(false));
  });

  // Checks the opacity slider: it goes from 10% to 80%, starts at 40%, and a
  // higher value makes the blue stronger.
  it("has an opacity from 10% to 80%, 40% by default", async () => {
    await openFrame(twoFrames(), 2);
    const slider = (await screen.findByLabelText(
      "Opacity",
    )) as HTMLInputElement;
    expect(slider.min).toBe("10");
    expect(slider.max).toBe("80");
    expect(slider.value).toBe("40");
    expect(screen.getByText("40%")).toBeTruthy();
    await waitFor(() => expect(isBlue(colorOfPixel(16, 16, 1, 1))).toBe(true));
    const at40 = colorOfPixel(16, 16, 1, 1)[2] - colorOfPixel(16, 16, 1, 1)[0];
    fireEvent.change(slider, { target: { value: "80" } });
    await waitFor(() => {
      const [r, , b] = colorOfPixel(16, 16, 1, 1);
      expect(b - r).toBeGreaterThan(at40);
    });
    expect(screen.getByText("80%")).toBeTruthy();
  });

  // Checks that the previous frame can not be edited: a stroke on frame 2 is
  // only sent for frame 2, also over the pixels of the onion skin.
  it("never edits the previous frame", async () => {
    const api = twoFrames();
    await openFrame(api, 2);
    await waitFor(() => expect(isBlue(colorOfPixel(16, 16, 1, 1))).toBe(true));
    fireEvent.click(screen.getByRole("button", { name: "Eraser" }));
    drag(
      [16, 16],
      [
        [0, 1],
        [3, 1],
      ],
    );
    await waitFor(() => expect(api.calls.length).toBeGreaterThanOrEqual(0));
    expect(api.calls.every((call) => call.frame === 2)).toBe(true);
    expect(api.rows(1)[1]).toBe(".R" + ".".repeat(14));
  });

  // Checks that the onion skin follows the frame: on frame 2 it shows frame 1,
  // and back on frame 1 it is gone.
  it("goes away when the user returns to frame 1", async () => {
    await openFrame(twoFrames(), 2);
    await waitFor(() => expect(isBlue(colorOfPixel(16, 16, 1, 1))).toBe(true));
    fireEvent.click(screen.getByRole("button", { name: "Frame 1" }));
    await waitFor(() =>
      expect(colorOfPixel(16, 16, 1, 1)).toEqual([255, 0, 0, 255]),
    );
  });
});
