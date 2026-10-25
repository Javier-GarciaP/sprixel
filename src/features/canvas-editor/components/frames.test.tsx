// @vitest-environment jsdom
import { cleanup, fireEvent, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { FakeEditorApi } from "../testing/fakeApi";
import {
  colorOfPixel,
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

const current = () =>
  screen
    .getAllByRole("button", { name: /^Frame \d$/ })
    .find((b) => b.getAttribute("aria-current") === "true")
    ?.getAttribute("aria-label");

describe("Changing frame", () => {
  // Checks that a click on a frame in the column makes it the current frame.
  it("changes frame with a click", async () => {
    renderEditor(new FakeEditorApi(8, 8, 3));
    fireEvent.click(await screen.findByRole("button", { name: "Frame 3" }));
    await waitFor(() => expect(current()).toBe("Frame 3"));
  });

  // Checks the keys `,` and `.`: they go to the previous and the next frame
  // and stop at the first and the last one. The key listener is attached in
  // an effect, so the key is sent inside waitFor.
  it("moves with the comma and period keys and stops at the ends", async () => {
    renderEditor(new FakeEditorApi(8, 8, 2));
    await screen.findByRole("button", { name: "Frame 2" });
    await waitFor(() => {
      fireEvent.keyDown(window, { key: "." });
      expect(current()).toBe("Frame 2");
    });
    // Already the last frame: another key press stays there.
    fireEvent.keyDown(window, { key: "." });
    await waitFor(() => expect(current()).toBe("Frame 2"));
    await waitFor(() => {
      fireEvent.keyDown(window, { key: "," });
      expect(current()).toBe("Frame 1");
    });
    // Already the first frame: another key press stays there.
    fireEvent.keyDown(window, { key: "," });
    await waitFor(() => expect(current()).toBe("Frame 1"));
  });
});

describe("Opening another sprite", () => {
  // Checks that a new sprite starts on its first frame: after the editor was
  // on frame 3 of one sprite, a stroke on a sprite with one frame is sent for
  // frame 1 and the column lists only that frame.
  it("starts on frame 1 and keeps nothing of the old sprite", async () => {
    const view = renderEditor(new FakeEditorApi(16, 16, 3));
    fireEvent.click(await screen.findByRole("button", { name: "Frame 3" }));
    await waitFor(() => expect(current()).toBe("Frame 3"));

    const other = new FakeEditorApi(16, 16, 1);
    view.show(other, { folder: [], name: "slime" });

    await waitFor(() => expect(current()).toBe("Frame 1"));
    expect(screen.queryByRole("button", { name: "Frame 3" })).toBeNull();
    await screen.findByText("1800%");
    drag([16, 16], [[3, 3]]);
    await waitFor(() => expect(other.calls).toHaveLength(1));
    expect(other.calls[0].frame).toBe(1);
  });
});

describe("Frames column", () => {
  const palette = [{ key: "R", hex: "#FF0000" }];
  // Frame 3 of 4 has a red pixel at (1, 1).
  function fourFrames() {
    const api = new FakeEditorApi(8, 8, 4, palette);
    const rows = Array.from({ length: 8 }, () => ".".repeat(8));
    rows[1] = ".R" + ".".repeat(6);
    api.setFrame(3, rows);
    return api;
  }

  // Checks the "Select a frame" scenario: a click on the thumbnail of frame 3
  // makes it the current frame and the canvas shows its pixels.
  it("shows frame 3 on the canvas after a click on its thumbnail", async () => {
    renderEditor(fourFrames());
    await screen.findByText("3200%");
    await waitFor(() => expect(colorOfPixel(8, 8, 1, 1)[0]).not.toBe(255));
    fireEvent.click(screen.getByRole("button", { name: "Frame 3" }));
    await waitFor(() => expect(current()).toBe("Frame 3"));
    await waitFor(() =>
      expect(colorOfPixel(8, 8, 1, 1)).toEqual([255, 0, 0, 255]),
    );
  });

  // Checks that every frame has a thumbnail with its picture: each frame is
  // read once from the core and drawn.
  it("draws a thumbnail for each frame", async () => {
    const api = fourFrames();
    const read = vi.spyOn(api, "loadFrameData");
    renderEditor(api);
    await waitFor(() => expect(read).toHaveBeenCalledTimes(4));
    expect(read.mock.calls.map((c) => c[1]).sort()).toEqual([1, 2, 3, 4]);
    await waitFor(() =>
      expect(putImageData.mock.calls.length).toBeGreaterThan(4),
    );
  });

  // Checks the "Add after the current frame" scenario: frame 2 of 4 is
  // current, "+" adds an empty frame, so the sprite has 5 frames, the new one
  // is frame 3 and it is the current frame (and it is empty).
  it("adds an empty frame right after the current one", async () => {
    const api = fourFrames();
    renderEditor(api);
    fireEvent.click(await screen.findByRole("button", { name: "Frame 2" }));
    await waitFor(() => expect(current()).toBe("Frame 2"));

    fireEvent.click(
      screen.getByRole("button", { name: /^Add an empty frame/ }),
    );

    await screen.findByRole("button", { name: "Frame 5" });
    await waitFor(() => expect(current()).toBe("Frame 3"));
    // The old frame 3 (with the red pixel) is now frame 4.
    expect(api.rows(4)[1]).toBe(".R" + ".".repeat(6));
    expect(api.rows(3)[1]).toBe(".".repeat(8));
  });

  // Checks the copy: a copy of the current frame goes after it with the same
  // pixels and becomes the current frame.
  it("adds a copy of the current frame", async () => {
    const api = fourFrames();
    renderEditor(api);
    fireEvent.click(await screen.findByRole("button", { name: "Frame 3" }));
    await waitFor(() => expect(current()).toBe("Frame 3"));
    fireEvent.click(screen.getByRole("button", { name: /^Add a copy/ }));
    await waitFor(() => expect(current()).toBe("Frame 4"));
    expect(api.rows(4)).toEqual(api.rows(3));
    await waitFor(() =>
      expect(colorOfPixel(8, 8, 1, 1)).toEqual([255, 0, 0, 255]),
    );
  });
});
