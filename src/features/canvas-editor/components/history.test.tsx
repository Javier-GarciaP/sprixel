// @vitest-environment jsdom
import { cleanup, fireEvent, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { FakeEditorApi } from "../testing/fakeApi";
import {
  colorOfPixel,
  drag,
  mockLayout,
  renderEditor,
} from "../testing/renderEditor";

beforeEach(() => mockLayout(500, 300));
afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
  vi.unstubAllGlobals();
});

const undoButton = () =>
  screen.getByRole("button", { name: "Undo" }) as HTMLButtonElement;
const redoButton = () =>
  screen.getByRole("button", { name: "Redo" }) as HTMLButtonElement;

// A sprite of 32x4 (1500%): a row of 30 pixels is one stroke.
async function open(api = new FakeEditorApi(32, 4)) {
  renderEditor(api);
  await screen.findByText("1500%");
  return api;
}

describe("Undo and redo", () => {
  // Checks the "Undo a stroke" scenario: a stroke of 30 pixels is one call to
  // the core, and one click on Undo gives all 30 pixels back at once; Redo
  // brings them back.
  it("undoes a 30 pixel stroke in one step", async () => {
    const api = await open();
    drag(
      [32, 4],
      [
        [0, 1],
        [29, 1],
      ],
    );
    await waitFor(() => expect(api.calls).toHaveLength(1));
    expect(api.calls[0].changes).toHaveLength(30);
    await waitFor(() => expect(undoButton().disabled).toBe(false));
    expect(redoButton().disabled).toBe(true);

    fireEvent.click(undoButton());
    await waitFor(() => expect(api.rows(1)[1]).toBe(".".repeat(32)));
    await waitFor(() =>
      expect(colorOfPixel(32, 4, 10, 1)).not.toEqual([0, 0, 0, 255]),
    );
    await waitFor(() => expect(undoButton().disabled).toBe(true));
    expect(redoButton().disabled).toBe(false);

    fireEvent.click(redoButton());
    await waitFor(() =>
      expect(api.rows(1)[1].slice(0, 30)).toMatch(/^[^.]{30}$/),
    );
    await waitFor(() =>
      expect(colorOfPixel(32, 4, 10, 1)).toEqual([0, 0, 0, 255]),
    );
  });

  // Checks the scenario "one Undo removes both the stroke and the new color"
  // on the screen: the palette panel loses the color the stroke added.
  it("removes the new color from the palette with the stroke", async () => {
    const api = await open();
    fireEvent.change(screen.getByLabelText("Hex color"), {
      target: { value: "#336699" },
    });
    drag([32, 4], [[2, 2]]);
    await screen.findByRole("button", { name: "#336699" });
    fireEvent.click(undoButton());
    await waitFor(() =>
      expect(screen.queryByRole("button", { name: "#336699" })).toBeNull(),
    );
    expect(api.palette).toEqual([]);
    fireEvent.click(redoButton());
    expect(await screen.findByRole("button", { name: "#336699" })).toBeTruthy();
  });

  // Checks that the buttons are off when there is nothing to undo or redo.
  it("has both buttons off at the start", async () => {
    await open();
    expect(undoButton().disabled).toBe(true);
    expect(redoButton().disabled).toBe(true);
  });

  // Checks the keys: Ctrl+Z undoes and Ctrl+Shift+Z redoes. The key listener
  // is attached in an effect, so the key is sent inside waitFor.
  it("undoes with Ctrl+Z and redoes with Ctrl+Shift+Z", async () => {
    const api = await open();
    drag(
      [32, 4],
      [
        [0, 0],
        [5, 0],
      ],
    );
    await waitFor(() => expect(api.calls).toHaveLength(1));
    await waitFor(() => {
      fireEvent.keyDown(window, { key: "z", ctrlKey: true });
      expect(api.rows(1)[0]).toBe(".".repeat(32));
    });
    await waitFor(() => {
      fireEvent.keyDown(window, { key: "Z", ctrlKey: true, shiftKey: true });
      expect(api.rows(1)[0].slice(0, 6)).toMatch(/^[^.]{6}$/);
    });
  });

  // Checks that undo goes back one stroke at a time.
  it("undoes strokes one by one", async () => {
    const api = await open();
    drag(
      [32, 4],
      [
        [0, 0],
        [2, 0],
      ],
    );
    await waitFor(() => expect(api.calls).toHaveLength(1));
    drag(
      [32, 4],
      [
        [0, 2],
        [2, 2],
      ],
    );
    await waitFor(() => expect(api.calls).toHaveLength(2));
    await waitFor(() => expect(undoButton().disabled).toBe(false));
    fireEvent.click(undoButton());
    await waitFor(() => expect(api.rows(1)[2].slice(0, 3)).toBe("..."));
    expect(api.rows(1)[0].slice(0, 3)).toMatch(/^[^.]{3}$/);
  });
});
