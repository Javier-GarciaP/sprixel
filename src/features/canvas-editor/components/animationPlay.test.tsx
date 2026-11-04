// @vitest-environment jsdom
import {
  act,
  cleanup,
  fireEvent,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
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
const frameButtons = () =>
  screen.queryAllByRole("button", { name: /^Frame \d+$/ });

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

describe("A long sprite", () => {
  const long = (frames: number) => {
    const api = new FakeEditorApi(4, 4, frames);
    api.tags = [{ name: "walk", from: 2, to: frames, fps: 10, mode: "loop" }];
    return api;
  };

  // Checks that above 40 frames the groups start closed (no thumbnails at all)
  // and the headers still show the count; opening one lists its frames.
  it("starts with the groups closed above 40 frames", async () => {
    renderEditor(long(41));
    const walk = await screen.findByRole("region", { name: "walk" });
    expect(within(walk).getByText("40 fr.")).toBeTruthy();
    expect(frameButtons()).toHaveLength(0);
    fireEvent.click(
      within(walk).getByRole("button", { name: "Show the frames of walk" }),
    );
    await waitFor(() => expect(frameButtons()).toHaveLength(40));
  });

  // Checks the limit itself: with exactly 40 frames the groups start open.
  it("starts with the groups open at 40 frames", async () => {
    renderEditor(long(40));
    await screen.findByRole("region", { name: "walk" });
    await waitFor(() => expect(frameButtons()).toHaveLength(40));
  });
});
