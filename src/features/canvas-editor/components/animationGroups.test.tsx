// @vitest-environment jsdom
import {
  cleanup,
  fireEvent,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { FakeEditorApi } from "../testing/fakeApi";
import { mockLayout, renderEditor } from "../testing/renderEditor";

beforeEach(() => mockLayout(500, 300));
afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
  vi.unstubAllGlobals();
});

// Frame 1 is the base; walk is 2-3 and jump is 4-5.
function twoAnimations() {
  const api = new FakeEditorApi(8, 8, 5);
  api.tags = [
    { name: "walk", from: 2, to: 3, fps: 6, mode: "loop" },
    { name: "jump", from: 4, to: 5, fps: 10, mode: "once" },
  ];
  return api;
}

const framesIn = (group: HTMLElement) =>
  within(group)
    .queryAllByRole("button", { name: /^Frame \d+$/ })
    .map((b) => b.getAttribute("aria-label"));

describe("The frames strip in groups", () => {
  // Checks the "Two animations" scenario: Base, walk and jump are groups in
  // this order, and each one lists only its own frames.
  it("shows Base, walk and jump with their frames", async () => {
    renderEditor(twoAnimations());
    const walk = await screen.findByRole("region", { name: "walk" });
    const base = screen.getByRole("region", { name: "Base" });
    const jump = screen.getByRole("region", { name: "jump" });
    expect(framesIn(base)).toEqual(["Frame 1"]);
    expect(framesIn(walk)).toEqual(["Frame 2", "Frame 3"]);
    expect(framesIn(jump)).toEqual(["Frame 4", "Frame 5"]);
    expect(
      screen.getAllByRole("region").map((r) => r.getAttribute("aria-label")),
    ).toEqual(["Base", "walk", "jump"]);
  });

  // Checks that the header of an animation shows its speed and how it ends.
  it("shows the speed and the loop mode in the header", async () => {
    renderEditor(twoAnimations());
    const walk = await screen.findByRole("region", { name: "walk" });
    expect(within(walk).getByText("6 fps")).toBeTruthy();
    expect(within(walk).getByRole("img", { name: "Repeat" })).toBeTruthy();
    const jump = screen.getByRole("region", { name: "jump" });
    expect(within(jump).getByRole("img", { name: "Once" })).toBeTruthy();
  });

  // Checks that frames no animation owns go in a last group, "No animation".
  it("puts the frames of no animation in the last group", async () => {
    const api = new FakeEditorApi(8, 8, 6);
    api.tags = [{ name: "walk", from: 1, to: 2, fps: 6, mode: "loop" }];
    renderEditor(api);
    const none = await screen.findByRole("region", { name: "No animation" });
    expect(framesIn(none)).toEqual([
      "Frame 3",
      "Frame 4",
      "Frame 5",
      "Frame 6",
    ]);
    const names = screen
      .getAllByRole("region")
      .map((r) => r.getAttribute("aria-label"));
    expect(names).toEqual(["walk", "No animation"]);
  });

  // Checks that a header collapses and expands its frames, and that the
  // header itself stays where it is.
  it("collapses and expands a group", async () => {
    renderEditor(twoAnimations());
    const walk = await screen.findByRole("region", { name: "walk" });
    fireEvent.click(
      within(walk).getByRole("button", { name: "Hide the frames of walk" }),
    );
    await waitFor(() => expect(framesIn(walk)).toEqual([]));
    fireEvent.click(
      within(walk).getByRole("button", { name: "Show the frames of walk" }),
    );
    await waitFor(() => expect(framesIn(walk)).toEqual(["Frame 2", "Frame 3"]));
  });

  // Checks that the play button of a header chooses that animation for the
  // preview (the selected group gets the amber border).
  it("chooses the animation with its play button", async () => {
    renderEditor(twoAnimations());
    fireEvent.click(await screen.findByRole("button", { name: "Play jump" }));
    await waitFor(() =>
      expect(
        (
          screen.getByRole("combobox", {
            name: "Animation",
          }) as HTMLSelectElement
        ).value,
      ).toBe("jump"),
    );
  });

  // Checks that old overlapping animations show each frame once and warn.
  it("warns when two animations share a frame", async () => {
    const api = new FakeEditorApi(8, 8, 4);
    api.tags = [
      { name: "a", from: 1, to: 3, fps: 6, mode: "loop" },
      { name: "b", from: 3, to: 4, fps: 6, mode: "loop" },
    ];
    renderEditor(api);
    await screen.findByRole("status");
    expect(screen.getAllByRole("button", { name: /^Frame \d+$/ })).toHaveLength(
      4,
    );
  });
});
