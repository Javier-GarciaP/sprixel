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

const choose = (name: string) =>
  fireEvent.click(screen.getByRole("button", { name: `Select ${name}` }));
const regionNames = () =>
  screen.getAllByRole("region").map((r) => r.getAttribute("aria-label"));
const preview = () =>
  screen.getByRole("combobox", { name: "Animation" }) as HTMLSelectElement;

describe("The frames strip in groups", () => {
  // Checks the "Two animations" scenario: Base, walk and jump are groups in
  // this order. At first only Base (the group of the current frame) is open.
  it("shows Base, walk and jump, and only Base is open", async () => {
    renderEditor(twoAnimations());
    const walk = await screen.findByRole("region", { name: "walk" });
    const base = screen.getByRole("region", { name: "Base" });
    const jump = screen.getByRole("region", { name: "jump" });
    expect(regionNames()).toEqual(["Base", "walk", "jump"]);
    expect(framesIn(base)).toEqual(["Frame 1"]);
    expect(framesIn(walk)).toEqual([]);
    expect(framesIn(jump)).toEqual([]);
  });

  // Checks that a click on a header chooses that animation and opens it, and
  // that the group that was open closes (an accordion). The preview list
  // follows the choice.
  it("opens only the group that is clicked", async () => {
    renderEditor(twoAnimations());
    await screen.findByRole("region", { name: "walk" });
    choose("walk");
    const walk = screen.getByRole("region", { name: "walk" });
    await waitFor(() => expect(framesIn(walk)).toEqual(["Frame 2", "Frame 3"]));
    expect(framesIn(screen.getByRole("region", { name: "Base" }))).toEqual([]);
    expect(preview().value).toBe("walk");
    choose("jump");
    const jump = screen.getByRole("region", { name: "jump" });
    await waitFor(() => expect(framesIn(jump)).toEqual(["Frame 4", "Frame 5"]));
    expect(framesIn(walk)).toEqual([]);
    expect(
      within(jump)
        .getByRole("button", { name: "Select jump" })
        .getAttribute("aria-expanded"),
    ).toBe("true");
  });

  // Checks that Base is a group too: choosing it opens its frames and goes
  // back to "All" (no animation).
  it("opens Base and goes back to All", async () => {
    renderEditor(twoAnimations());
    await screen.findByRole("region", { name: "walk" });
    choose("jump");
    await waitFor(() => expect(preview().value).toBe("jump"));
    choose("Base");
    await waitFor(() => expect(preview().value).toBe(""));
    expect(framesIn(screen.getByRole("region", { name: "Base" }))).toEqual([
      "Frame 1",
    ]);
    expect(framesIn(screen.getByRole("region", { name: "jump" }))).toEqual([]);
  });

  // Checks that the header row never changes size: the same height classes
  // whether the group is chosen or not (no jump when selecting).
  it("keeps the header height when a group is chosen", async () => {
    renderEditor(twoAnimations());
    await screen.findByRole("region", { name: "walk" });
    const row = () =>
      screen.getByRole("button", { name: "Select walk" }).parentElement;
    const before = row()?.className;
    choose("walk");
    await waitFor(() =>
      expect(
        screen
          .getByRole("button", { name: "Select walk" })
          .getAttribute("aria-expanded"),
      ).toBe("true"),
    );
    expect(before).toContain("h-8");
    expect(row()?.className).toBe(before);
  });

  // Checks that the speed and the end mode show under the name only for the
  // chosen group; the others have them in the tooltip (title).
  it("shows the speed and the mode only for the chosen group", async () => {
    renderEditor(twoAnimations());
    const walk = await screen.findByRole("region", { name: "walk" });
    expect(within(walk).queryByText("6 fps")).toBeNull();
    choose("walk");
    await waitFor(() => expect(within(walk).getByText("6 fps")).toBeTruthy());
    expect(within(walk).getByRole("img", { name: "Repeat" })).toBeTruthy();
    const jump = screen.getByRole("region", { name: "jump" });
    expect(within(jump).queryByRole("img", { name: "Once" })).toBeNull();
  });

  // Checks that frames no animation owns go in a last group, "No animation".
  it("puts the frames of no animation in the last group", async () => {
    const api = new FakeEditorApi(8, 8, 6);
    api.tags = [{ name: "walk", from: 1, to: 2, fps: 6, mode: "loop" }];
    renderEditor(api);
    const none = await screen.findByRole("region", { name: "No animation" });
    expect(regionNames()).toEqual(["walk", "No animation"]);
    choose("No animation");
    await waitFor(() =>
      expect(framesIn(none)).toEqual([
        "Frame 3",
        "Frame 4",
        "Frame 5",
        "Frame 6",
      ]),
    );
  });

  // Checks that the play button of a header chooses that animation for the
  // preview and opens the group.
  it("chooses the animation with its play button", async () => {
    renderEditor(twoAnimations());
    fireEvent.click(await screen.findByRole("button", { name: "Play jump" }));
    await waitFor(() => expect(preview().value).toBe("jump"));
    await waitFor(() =>
      expect(
        framesIn(screen.getByRole("region", { name: "jump" })),
      ).toHaveLength(2),
    );
  });

  // Checks the keyboard: Down and Up move the focus between the headers, and
  // Home and End go to the first and the last one.
  it("moves between the headers with the arrow keys", async () => {
    renderEditor(twoAnimations());
    await screen.findByRole("region", { name: "walk" });
    const header = (name: string) =>
      screen.getByRole("button", { name: `Select ${name}` });
    header("Base").focus();
    fireEvent.keyDown(header("Base"), { key: "ArrowDown" });
    expect(document.activeElement).toBe(header("walk"));
    fireEvent.keyDown(header("walk"), { key: "End" });
    expect(document.activeElement).toBe(header("jump"));
    fireEvent.keyDown(header("jump"), { key: "ArrowUp" });
    expect(document.activeElement).toBe(header("walk"));
    fireEvent.keyDown(header("walk"), { key: "Home" });
    expect(document.activeElement).toBe(header("Base"));
  });

  // Checks that the actions at the bottom are there: a full-width button for
  // a new animation and two icon buttons (add and copy a frame).
  it("has the actions at the bottom", async () => {
    renderEditor(twoAnimations());
    expect(
      await screen.findByRole("button", { name: "New animation…" }),
    ).toBeTruthy();
    expect(
      screen.getByRole("button", { name: /^Add an empty frame/ }),
    ).toBeTruthy();
    expect(screen.getByRole("button", { name: /^Add a copy/ })).toBeTruthy();
  });

  // Checks that a long sprite (120 frames) keeps working: only the chosen
  // animation lists its frames, the other groups stay one row.
  it("lists only the frames of the open group in a long sprite", async () => {
    const api = new FakeEditorApi(4, 4, 120);
    api.tags = [
      { name: "walk", from: 2, to: 100, fps: 10, mode: "loop" },
      { name: "idle", from: 101, to: 120, fps: 10, mode: "loop" },
    ];
    renderEditor(api);
    await screen.findByRole("region", { name: "idle" });
    expect(screen.getAllByRole("button", { name: /^Frame \d+$/ })).toHaveLength(
      1,
    );
    choose("walk");
    await waitFor(() =>
      expect(
        screen.getAllByRole("button", { name: /^Frame \d+$/ }),
      ).toHaveLength(99),
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
      3,
    );
    choose("b");
    await waitFor(() =>
      expect(
        screen.getAllByRole("button", { name: /^Frame \d+$/ }),
      ).toHaveLength(1),
    );
  });
});
