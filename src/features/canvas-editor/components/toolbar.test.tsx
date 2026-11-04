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
import { drag, mockLayout, renderEditor } from "../testing/renderEditor";

beforeEach(() => mockLayout(500, 300));
afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
  vi.unstubAllGlobals();
});

// The keys are listened to from an effect: press M (harmless) until the
// Select tool is active, so all the handlers are there.
async function ready(api = new FakeEditorApi(32, 4)) {
  renderEditor(api);
  await screen.findByText("1500%");
  await waitFor(() => {
    fireEvent.keyDown(window, { key: "m" });
    expect(screen.getByTestId("active-tool").textContent).toContain("Select");
  });
  return api;
}

const bar = () => screen.getByRole("toolbar", { name: "Editor controls" });
const button = (name: string) => screen.getByRole("button", { name });
const names = () =>
  within(bar())
    .getAllByRole("button")
    .map((b) => b.getAttribute("aria-label") ?? b.textContent);

describe("Top bar of the editor", () => {
  // Checks the fix for the jumping screen: selecting an area must not add or
  // remove any control. The same buttons are there, the ones that need a
  // selection are only disabled before, and enabled after.
  it("keeps the same controls with and without a selection", async () => {
    await ready();
    const before = names();
    expect(button("Flip horizontally").hasAttribute("disabled")).toBe(true);
    expect(button("Rotate 180°").hasAttribute("disabled")).toBe(true);
    drag(
      [32, 4],
      [
        [2, 0],
        [5, 2],
      ],
    );
    await waitFor(() =>
      expect(button("Flip horizontally").hasAttribute("disabled")).toBe(false),
    );
    expect(names()).toEqual(before);
    expect(screen.queryByRole("toolbar", { name: "Selection actions" })).toBe(
      null,
    );
  });

  // Checks that the symmetry control shows which mode is on, and that a
  // click changes it.
  it("shows the symmetry mode", async () => {
    await ready();
    expect(button("Symmetry: off").getAttribute("aria-pressed")).toBe("true");
    fireEvent.click(button("Symmetry: both"));
    expect(button("Symmetry: both").getAttribute("aria-pressed")).toBe("true");
    expect(button("Symmetry: off").getAttribute("aria-pressed")).toBe("false");
  });

  // Checks the tooltip: it shows with the pointer and with the keyboard
  // focus, and has the name and the shortcut from the shortcut system.
  it("shows the name and the shortcut on hover and on focus", async () => {
    await ready();
    fireEvent.mouseEnter(button("Undo").parentElement!);
    const tip = await screen.findByRole("tooltip");
    expect(tip.textContent).toContain("Undo");
    expect(tip.textContent).toContain("Ctrl+Z");
    fireEvent.mouseLeave(button("Undo").parentElement!);
    await waitFor(() => expect(screen.queryByRole("tooltip")).toBeNull());
    fireEvent.focus(button("Review sprite"));
    expect((await screen.findByRole("tooltip")).textContent).toContain(
      "Shift+A",
    );
    fireEvent.blur(button("Review sprite"));
    await waitFor(() => expect(screen.queryByRole("tooltip")).toBeNull());
  });

  // Checks that a disabled control says why when the pointer is over it.
  it("tells why a control is disabled", async () => {
    await ready();
    fireEvent.mouseEnter(button("Flip horizontally").parentElement!);
    expect((await screen.findByRole("tooltip")).textContent).toContain(
      "Select an area first",
    );
  });
});
