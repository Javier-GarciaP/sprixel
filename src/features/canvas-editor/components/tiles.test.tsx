// @vitest-environment jsdom
import { cleanup, fireEvent, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { FakeEditorApi } from "../testing/fakeApi";
import {
  drag,
  mockLayout,
  putImageData,
  renderEditor,
} from "../testing/renderEditor";

beforeEach(() => mockLayout(500, 300));
afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
  vi.unstubAllGlobals();
});

const toggle = () =>
  screen.getByRole("switch", { name: "Tile preview (3 × 3)" });
const calls = (api: FakeEditorApi) =>
  api.powerCalls.filter((c) => c.name === "tileCheck");

async function open(api = new FakeEditorApi(16, 16)) {
  renderEditor(api);
  await screen.findByText("1800%");
  return api;
}

// The last picture sent to a canvas that is 3 times the sprite (48 wide).
function tilePicture() {
  const images = putImageData.mock.calls
    .map((c) => c[0] as { data: Uint8ClampedArray; width: number })
    .filter((image) => image.width === 48);
  return images[images.length - 1];
}

describe("Tile preview", () => {
  // Checks that the preview is off at first, and the switch shows a 3 x 3
  // picture of the frame and asks the core for the seam check.
  it("shows the tile three times on each side", async () => {
    const api = await open();
    expect(
      screen.queryByRole("img", { name: "Tile preview of frame 1" }),
    ).toBeNull();
    expect(calls(api)).toHaveLength(0);
    fireEvent.click(toggle());
    await screen.findByRole("img", { name: "Tile preview of frame 1" });
    await waitFor(() => expect(calls(api)).toHaveLength(1));
    expect(tilePicture().width).toBe(48);
  });

  // Checks both answers of the check: the edges match, or they do not and the
  // number of pairs is said (the warning of the spec).
  it("says if the tile is seamless and warns about seams", async () => {
    const api = new FakeEditorApi(16, 16);
    api.tile = { seamless: true, mismatches: [] };
    await open(api);
    fireEvent.click(toggle());
    expect(await screen.findByText("Seamless: the edges match.")).toBeTruthy();
    fireEvent.click(toggle());
    api.tile = {
      seamless: false,
      mismatches: [
        { edge: "left_right", a: [0, 1], b: [15, 1] },
        { edge: "top_bottom", a: [2, 0], b: [2, 15] },
        { edge: "top_bottom", a: [3, 0], b: [3, 15] },
      ],
    };
    fireEvent.click(toggle());
    expect(
      await screen.findByText("Seams: 3 pairs of edge pixels do not match."),
    ).toBeTruthy();
  });

  // Checks that a stroke makes the tile and the check again, and that the
  // pixel is repeated in the other tiles (x 0 and x 16 and x 32).
  it("is made again after a stroke", async () => {
    const api = await open();
    fireEvent.click(toggle());
    await waitFor(() => expect(calls(api)).toHaveLength(1));
    drag([16, 16], [[0, 0]]);
    await waitFor(() => expect(calls(api).length).toBeGreaterThan(1));
    await waitFor(() => {
      const { data } = tilePicture();
      for (const x of [0, 16, 32]) expect(data[x * 4 + 3]).toBe(255);
      expect(data[1 * 4 + 3]).toBe(0);
    });
  });

  // Checks the T key.
  it("opens and closes with T", async () => {
    await open();
    await waitFor(() => {
      fireEvent.keyDown(window, { key: "t" });
      expect(toggle().getAttribute("aria-checked")).toBe("true");
    });
    fireEvent.keyDown(window, { key: "t" });
    await waitFor(() =>
      expect(toggle().getAttribute("aria-checked")).toBe("false"),
    );
  });
});
