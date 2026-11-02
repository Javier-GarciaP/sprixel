// @vitest-environment jsdom
import { cleanup, fireEvent, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { FakeEditorApi } from "../testing/fakeApi";
import { mockLayout, renderEditor } from "../testing/renderEditor";

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
});
