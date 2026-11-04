// @vitest-environment jsdom
import { act, cleanup, fireEvent, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { FakeEditorApi } from "../testing/fakeApi";
import { manualClock, mockLayout, renderEditor } from "../testing/renderEditor";

beforeEach(() => mockLayout(500, 300));
afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
  vi.unstubAllGlobals();
});

const shown = () =>
  Number(screen.getByTestId("preview").getAttribute("data-frame"));

describe("Playing one group", () => {
  // Checks the "Two animations" scenario: the play button of "jump" (frames
  // 4-5) plays only those two frames, not the base or "walk". Time is moved
  // by hand: 100 ms per frame at 10 fps.
  it("plays only the frames of the group", async () => {
    const api = new FakeEditorApi(8, 8, 5);
    api.tags = [
      { name: "walk", from: 2, to: 3, fps: 10, mode: "loop" },
      { name: "jump", from: 4, to: 5, fps: 10, mode: "loop" },
    ];
    renderEditor(api);
    fireEvent.click(await screen.findByRole("button", { name: "Play jump" }));
    await screen.findByRole("button", { name: "Pause" });
    const seen = new Set<number>();
    for (let i = 0; i < 8; i++) {
      act(() => manualClock.tick(100));
      seen.add(shown());
    }
    expect(seen).toEqual(new Set([4, 5]));
  });
});
