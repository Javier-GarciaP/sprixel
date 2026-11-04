// @vitest-environment jsdom
import { cleanup, fireEvent, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { FakeEditorApi } from "../testing/fakeApi";
import { mockLayout, pictureAt, renderEditor } from "../testing/renderEditor";

beforeEach(() => mockLayout(500, 300));
afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
  vi.unstubAllGlobals();
});

const palette = [
  { key: "K", hex: "#000000" },
  { key: "R", hex: "#FF0000" },
];

async function open(api: FakeEditorApi) {
  renderEditor(api);
  await screen.findByText("1800%");
  fireEvent.click(screen.getByRole("button", { name: "Review sprite" }));
  await screen.findByRole("dialog", { name: "Review" });
}

describe("Review panel", () => {
  // Checks the "Clean sprite" scenario.
  it("says so when nothing is wrong", async () => {
    await open(new FakeEditorApi(16, 16, 1, palette));
    expect(await screen.findByText("Nothing wrong found.")).toBeTruthy();
  });

  // Checks the "Loose pixel" scenario: the pixel is listed with its frame,
  // and clicking it goes to that frame and draws a box on the canvas.
  it("lists a loose pixel and points at it when clicked", async () => {
    const api = new FakeEditorApi(16, 16, 2, palette);
    api.review_ = {
      used: ["R"],
      unused: [],
      frames: [
        { frame: 1, loose: [], gaps: [] },
        { frame: 2, loose: [[3, 4]], gaps: [] },
      ],
      changes: [],
    };
    await open(api);
    fireEvent.click(
      await screen.findByRole("button", { name: "Frame 2: 3, 4" }),
    );
    await waitFor(() =>
      expect(
        screen
          .getByRole("button", { name: "Frame 2" })
          .getAttribute("aria-current"),
      ).toBe("true"),
    );
    // Sprite 16x16 at 18x in 500x300: x starts at 106, y at 6. The box is
    // black one pixel outside the edge of pixel (3, 4).
    await waitFor(() =>
      expect(pictureAt(106 + 3 * 18 - 1, 6 + 4 * 18 + 5).slice(0, 3)).toEqual([
        0, 0, 0,
      ]),
    );
  });

  // Checks the unused colors, the outline gaps (they need an outline key and
  // the panel asks the core again with it) and the frame comparison text.
  it("lists unused colors, gaps and the comparison of frames", async () => {
    const api = new FakeEditorApi(16, 16, 2, palette);
    api.review_ = {
      used: ["K"],
      unused: ["R"],
      frames: [
        { frame: 1, loose: [], gaps: [[1, 1]] },
        { frame: 2, loose: [], gaps: [] },
      ],
      changes: [
        { from: 1, to: 2, changedPixels: 5, bbox: null, groundKept: true },
      ],
    };
    await open(api);
    expect(await screen.findByText("Unused colors (1)")).toBeTruthy();
    expect(
      screen.getByText("Frame 1 to 2: 5 pixels changed (feet kept)"),
    ).toBeTruthy();
    const group = screen.getByRole("group", {
      name: "Outline color (to find gaps)",
    });
    fireEvent.click(group.querySelector('[aria-label="Color K (#000000)"]')!);
    await screen.findByText("Outline gaps (1)");
    const reviews = api.powerCalls.filter((c) => c.name === "review");
    expect(reviews[0].args[1]).toBeNull();
    expect(reviews[reviews.length - 1].args[1]).toBe("K");
  });

  // Checks that closing the panel takes the box away and the panel goes.
  it("closes with Done", async () => {
    await open(new FakeEditorApi(16, 16, 1, palette));
    fireEvent.click(screen.getByRole("button", { name: "Done" }));
    await waitFor(() =>
      expect(screen.queryByRole("dialog", { name: "Review" })).toBeNull(),
    );
  });
});
