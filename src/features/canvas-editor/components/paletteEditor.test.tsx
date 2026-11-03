// @vitest-environment jsdom
import {
  act,
  cleanup,
  fireEvent,
  screen,
  waitFor,
} from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { FakeEditorApi } from "../testing/fakeApi";
import { goblin, mockLayout, renderEditor } from "../testing/renderEditor";

beforeEach(() => mockLayout(500, 300));
afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
  vi.unstubAllGlobals();
});

const palette = [
  { key: "R", hex: "#F04545" },
  { key: "B", hex: "#2F6BFF" },
];

async function openEditor() {
  const api = new FakeEditorApi(16, 16, 1, palette);
  renderEditor(api);
  await screen.findByText("1800%");
  return api;
}

describe("Palette editor in the editor", () => {
  // Checks that the Edit button of the palette panel opens the palette editor
  // with the palette of the sprite and its colors.
  it("opens the palette editor from the Edit button", async () => {
    await openEditor();
    fireEvent.click(screen.getByRole("button", { name: "Edit" }));
    const dialog = await screen.findByRole("dialog");
    expect(dialog.getAttribute("aria-label")).toBe("Edit palette · main");
    expect(await screen.findByRole("button", { name: /^R R$/ })).toBeTruthy();
  });

  // Checks that a sprite with its own palette shows THAT palette: its colors
  // are in the panel and the Edit button opens the editor with its name.
  it("shows and edits the own palette of the sprite", async () => {
    const api = new FakeEditorApi(16, 16, 1, [{ key: "S", hex: "#FFCC99" }]);
    api.paletteName = "orc";
    renderEditor(api);
    await screen.findByText("1800%");
    expect(await screen.findByRole("button", { name: "#FFCC99" })).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "Edit" }));
    const dialog = await screen.findByRole("dialog");
    expect(dialog.getAttribute("aria-label")).toBe("Edit palette · orc");
  });

  // Checks that Done applies the edit and the editor reads the sprite again,
  // so the new colors show at once; and that Cancel does neither.
  it("reads the sprite again after Done, and not after Cancel", async () => {
    const api = await openEditor();
    const load = vi.spyOn(api, "loadSprite");
    fireEvent.click(screen.getByRole("button", { name: "Edit" }));
    await screen.findByRole("dialog");
    fireEvent.click(screen.getByRole("button", { name: "Cancel" }));
    await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
    expect(api.paletteEdits).toEqual([]);
    expect(load).not.toHaveBeenCalled();

    fireEvent.click(screen.getByRole("button", { name: "Edit" }));
    await screen.findByRole("dialog");
    fireEvent.click(await screen.findByRole("button", { name: /^R R$/ }));
    fireEvent.change(screen.getByLabelText("Color (hex)"), {
      target: { value: "#D03030" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Done" }));
    await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
    expect(api.paletteEdits).toHaveLength(1);
    expect(api.paletteEdits[0].colors[0].hex).toBe("#D03030");
    await waitFor(() => expect(load).toHaveBeenCalled());
  });

  // Checks task 2.3 on the screen: an agent that writes to the open sprite
  // blocks key changes in the palette editor (a simulated agent).
  it("blocks key changes while an agent writes to the sprite", async () => {
    const api = await openEditor();
    act(() => api.emit({ agent: "Claude Code", sprite: goblin, frame: 1 }));
    await screen.findByTestId("agent-border");
    fireEvent.click(screen.getByRole("button", { name: "Edit" }));
    await screen.findByRole("dialog");
    fireEvent.click(await screen.findByRole("button", { name: /^R R$/ }));
    fireEvent.change(screen.getByLabelText("Key (1 letter)"), {
      target: { value: "H" },
    });
    expect((await screen.findByRole("alert")).textContent).toContain(
      "Wait for it to finish",
    );
  });
});
