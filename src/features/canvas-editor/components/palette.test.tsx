// @vitest-environment jsdom
import { cleanup, fireEvent, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { FakeEditorApi } from "../testing/fakeApi";
import {
  colorOfPixel,
  mockLayout,
  pixelPoint,
  renderEditor,
} from "../testing/renderEditor";

beforeEach(() => mockLayout(500, 300));
afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
  vi.unstubAllGlobals();
});

function chooseHex(hex: string) {
  fireEvent.change(screen.getByLabelText("Hex color"), {
    target: { value: hex },
  });
}

function clickPixel(x: number, y: number) {
  const area = screen.getByTestId("canvas-area");
  fireEvent.pointerDown(area, { ...pixelPoint(16, 16, x, y), button: 0 });
  fireEvent.pointerUp(area, pixelPoint(16, 16, x, y));
}

describe("New colors join the palette", () => {
  // Checks the "A color that is not in the palette" scenario on the screen:
  // the stroke is drawn with the new color at once, and when the core answers,
  // the palette panel shows the new color.
  it("adds a new color to the palette panel after the stroke", async () => {
    const api = new FakeEditorApi(16, 16, 1, [{ key: "R", hex: "#F04545" }]);
    renderEditor(api);
    await screen.findByText("1800%");
    chooseHex("#123456");
    clickPixel(3, 3);
    expect(colorOfPixel(16, 16, 3, 3)).toEqual([18, 52, 86, 255]);
    await waitFor(() => expect(api.calls).toHaveLength(1));
    expect(await screen.findByRole("button", { name: "#123456" })).toBeTruthy();
    expect(api.palette.map((c) => c.hex)).toEqual(["#F04545", "#123456"]);
    // The pixel still shows the same color with the real key.
    expect(colorOfPixel(16, 16, 3, 3)).toEqual([18, 52, 86, 255]);
  });

  // Checks that using the same new color again does not add it twice, and
  // that two strokes before the first answer still work.
  it("adds a color once however many strokes use it", async () => {
    const api = new FakeEditorApi(16, 16);
    renderEditor(api);
    await screen.findByText("1800%");
    chooseHex("#00AA00");
    clickPixel(1, 1);
    clickPixel(2, 2);
    await waitFor(() => expect(api.calls).toHaveLength(2));
    await screen.findByRole("button", { name: "#00AA00" });
    expect(api.palette).toHaveLength(1);
    expect(screen.getAllByRole("button", { name: "#00AA00" })).toHaveLength(1);
    expect(colorOfPixel(16, 16, 2, 2)).toEqual([0, 170, 0, 255]);
  });

  // Checks the "The palette is full" scenario: with 93 colors a new color
  // shows "The palette is full" and the pixels are not drawn with it (the
  // frame is read again from the core, which refused the stroke).
  it("shows a message and keeps the pixels when the palette is full", async () => {
    const full = Array.from({ length: 93 }, (_, i) => ({
      key: String.fromCharCode(33 + i + (i >= 13 ? 1 : 0)),
      hex: `#${(i + 1).toString(16).padStart(6, "0").toUpperCase()}`,
    }));
    const api = new FakeEditorApi(16, 16, 1, full);
    renderEditor(api);
    await screen.findByText("1800%");
    chooseHex("#ABCDEF");
    clickPixel(4, 4);
    expect((await screen.findByRole("status")).textContent).toBe(
      "The palette is full",
    );
    expect(api.rows(1).join("")).toBe(".".repeat(256));
    await waitFor(() =>
      expect(colorOfPixel(16, 16, 4, 4)).not.toEqual([171, 205, 239, 255]),
    );
  });

  // Checks that a color that is already in a full palette still draws.
  it("still draws with a color that is in a full palette", async () => {
    const full = Array.from({ length: 93 }, (_, i) => ({
      key: String.fromCharCode(33 + i + (i >= 13 ? 1 : 0)),
      hex: `#${(i + 1).toString(16).padStart(6, "0").toUpperCase()}`,
    }));
    const api = new FakeEditorApi(16, 16, 1, full);
    renderEditor(api);
    await screen.findByText("1800%");
    chooseHex(full[5].hex);
    clickPixel(4, 4);
    await waitFor(() => expect(api.calls).toHaveLength(1));
    expect(api.rows(1)[4][4]).toBe(full[5].key);
    expect(screen.queryByRole("status")).toBeNull();
  });
});
