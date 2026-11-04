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

// Only the chosen group lists its frames, so the group is chosen first.
const framesIn = async (name: string) => {
  fireEvent.click(
    await screen.findByRole("button", { name: `Select ${name}` }),
  );
  return within(screen.getByRole("region", { name }))
    .queryAllByRole("button", { name: /^Frame \d+$/ })
    .map((b) => b.getAttribute("aria-label"));
};

async function openMenu(animation: string, item: string) {
  fireEvent.click(
    await screen.findByRole("button", { name: `Menu of ${animation}` }),
  );
  fireEvent.click(await screen.findByRole("menuitem", { name: item }));
}

async function openNewDialog() {
  fireEvent.click(
    await screen.findByRole("button", { name: "New animation…" }),
  );
  return screen.findByRole("dialog");
}

describe("New animation", () => {
  // Checks that the dialog sends name, frame count, speed and mode in ONE call
  // (one undo step), and that the new group appears with the copied frames.
  it("creates an animation from the base with one call", async () => {
    const api = twoAnimations();
    renderEditor(api);
    const dialog = await openNewDialog();
    fireEvent.change(within(dialog).getByLabelText("Name"), {
      target: { value: "run" },
    });
    fireEvent.change(within(dialog).getByLabelText("Frames"), {
      target: { value: "3" },
    });
    fireEvent.change(within(dialog).getByLabelText("Speed (fps)"), {
      target: { value: "8" },
    });
    fireEvent.click(within(dialog).getByRole("radio", { name: "Ping-pong" }));
    fireEvent.click(
      within(dialog).getByRole("button", { name: "Create animation" }),
    );
    await screen.findByRole("region", { name: "run" });
    expect(await framesIn("run")).toEqual(["Frame 6", "Frame 7", "Frame 8"]);
    expect(api.animationCalls).toEqual(["create run 3 8 pingpong"]);
    expect(screen.queryByRole("dialog")).toBeNull();
  });

  // Checks the clear error for a repeated name (ignoring case): it is shown
  // under the field, the button is off and nothing is sent.
  it("shows an error for a repeated name and does not create", async () => {
    const api = twoAnimations();
    renderEditor(api);
    await screen.findByRole("region", { name: "walk" });
    const dialog = await openNewDialog();
    fireEvent.change(within(dialog).getByLabelText("Name"), {
      target: { value: "WALK" },
    });
    expect(
      await within(dialog).findByText(
        "This sprite already has an animation with this name",
      ),
    ).toBeTruthy();
    expect(
      (
        within(dialog).getByRole("button", {
          name: "Create animation",
        }) as HTMLButtonElement
      ).disabled,
    ).toBe(true);
    expect(api.animationCalls).toEqual([]);
  });

  // Checks that when the core refuses the name (it knows better), the dialog
  // stays open and shows the same clear message.
  it("shows the core's name_exists error in the dialog", async () => {
    const api = twoAnimations();
    vi.spyOn(api, "animationCreate").mockRejectedValue({
      code: "name_exists",
      message: "x",
    });
    renderEditor(api);
    const dialog = await openNewDialog();
    fireEvent.change(within(dialog).getByLabelText("Name"), {
      target: { value: "run" },
    });
    fireEvent.click(
      within(dialog).getByRole("button", { name: "Create animation" }),
    );
    expect(await within(dialog).findByRole("alert")).toHaveProperty(
      "textContent",
      "This sprite already has an animation with this name",
    );
  });
});

describe("The menu of an animation", () => {
  // Checks that rename sends the old and the new name in one call, and that a
  // name used by another animation is refused in the dialog.
  it("renames an animation", async () => {
    const api = twoAnimations();
    renderEditor(api);
    await openMenu("walk", "Rename…");
    const dialog = await screen.findByRole("dialog");
    fireEvent.change(within(dialog).getByLabelText("Name"), {
      target: { value: "Jump" },
    });
    expect(
      await within(dialog).findByText(
        "This sprite already has an animation with this name",
      ),
    ).toBeTruthy();
    fireEvent.change(within(dialog).getByLabelText("Name"), {
      target: { value: "stroll" },
    });
    fireEvent.click(within(dialog).getByRole("button", { name: "Save" }));
    await screen.findByRole("region", { name: "stroll" });
    expect(api.animationCalls).toEqual(["rename walk stroll"]);
    expect(screen.queryByRole("region", { name: "walk" })).toBeNull();
  });

  // Checks that duplicate makes "walk copy" with the same frames, in one call.
  it("duplicates an animation", async () => {
    const api = twoAnimations();
    renderEditor(api);
    await openMenu("walk", "Duplicate");
    await screen.findByRole("region", { name: "walk copy" });
    expect(await framesIn("walk copy")).toEqual(["Frame 6", "Frame 7"]);
    expect(api.animationCalls).toEqual(["duplicate walk walk copy"]);
  });

  // Checks "delete, keep the frames": the group goes, the frames stay (they
  // join the Base because they come before the next animation).
  it("deletes only the animation", async () => {
    const api = twoAnimations();
    renderEditor(api);
    await openMenu("walk", "Delete animation (keep frames)");
    await waitFor(() =>
      expect(screen.queryByRole("region", { name: "walk" })).toBeNull(),
    );
    expect(await framesIn("Base")).toEqual(["Frame 1", "Frame 2", "Frame 3"]);
    expect(api.animationCalls).toEqual(["delete walk false"]);
  });

  // Checks "delete with frames": the frames go and the next animation follows
  // its frames (jump moves from 4-5 to 2-3).
  it("deletes the animation and its frames", async () => {
    const api = twoAnimations();
    renderEditor(api);
    await openMenu("walk", "Delete animation and its frames");
    await waitFor(() =>
      expect(screen.queryByRole("region", { name: "walk" })).toBeNull(),
    );
    expect(await framesIn("jump")).toEqual(["Frame 2", "Frame 3"]);
    expect(api.animationCalls).toEqual(["delete walk true"]);
  });

  // Checks that every control of a header has a name for the tooltip and for
  // screen readers (title and aria-label). The title of the name button also
  // tells the speed and how the animation ends.
  it("names every header control", async () => {
    renderEditor(twoAnimations());
    const walk = await screen.findByRole("region", { name: "walk" });
    for (const label of ["Play walk", "Menu of walk"]) {
      expect(
        within(walk).getByRole("button", { name: label }).getAttribute("title"),
      ).toBe(label);
    }
    expect(
      within(walk)
        .getByRole("button", { name: "Select walk" })
        .getAttribute("title"),
    ).toBe("walk · 6 fps · Repeat");
  });
});
