// @vitest-environment jsdom
import { cleanup, fireEvent, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { FakeTextApi } from "@/features/text-view";
import { FakeEditorApi } from "../testing/fakeApi";
import {
  colorOfPixel,
  mockLayout,
  renderEditor,
} from "../testing/renderEditor";

beforeEach(() => mockLayout(500, 300));
afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
  vi.unstubAllGlobals();
});

describe("Canvas and Text switch", () => {
  // Checks the "Switch to text" scenario: while editing frame 2, the switch
  // to Text shows frame 2 of the same sprite (its text is read from the core).
  it("shows the same frame in the Text view", async () => {
    const text = new FakeTextApi();
    renderEditor(new FakeEditorApi(8, 8, 3), { textApi: text });
    fireEvent.click(await screen.findByRole("button", { name: "Frame 2" }));
    fireEvent.click(screen.getByRole("button", { name: "Text" }));
    await screen.findByRole("textbox", { name: "Text grid" });
    expect(screen.getByText("goblin.txt")).toBeTruthy();
    expect(screen.getByText("Frame 2")).toBeTruthy();
    await waitFor(() => expect(text.loaded).toContain(2));
  });

  // Checks that the switch is in the same place in both views (the bar above)
  // and shows which view is on, and that Canvas brings the canvas back with
  // the frame the Text view left.
  it("goes back to the canvas on the frame of the text", async () => {
    renderEditor(new FakeEditorApi(8, 8, 3), { textApi: new FakeTextApi() });
    const canvasButton = await screen.findByRole("button", { name: "Canvas" });
    const textButton = screen.getByRole("button", { name: "Text" });
    expect(canvasButton.getAttribute("aria-pressed")).toBe("true");
    fireEvent.click(textButton);
    await screen.findByRole("textbox", { name: "Text grid" });
    expect(textButton.getAttribute("aria-pressed")).toBe("true");
    // The canvas bar and the frames column are still there.
    expect(screen.getByTestId("active-tool")).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "Frame 3" }));
    fireEvent.click(canvasButton);
    await waitFor(() =>
      expect(screen.queryByRole("textbox", { name: "Text grid" })).toBeNull(),
    );
    expect(screen.getByTestId("canvas-area")).toBeTruthy();
    expect(
      screen
        .getAllByRole("button", { name: /^Frame \d$/ })
        .find((b) => b.getAttribute("aria-current") === "true")
        ?.getAttribute("aria-label"),
    ).toBe("Frame 3");
  });

  // Checks the "Paste a grid from an AI" scenario on the whole editor: a grid
  // is pasted and applied, the canvas reads the sprite again, and after going
  // back to Canvas it shows the new pixels.
  it("shows the applied grid on the canvas", async () => {
    const editor = new FakeEditorApi(2, 2, 1, [{ key: "R", hex: "#F04545" }]);
    const text = new FakeTextApi();
    text.size = { width: 2, height: 2 };
    text.texts[1] = "frame 1\n..\n..\n";
    // What the core does when the grid is applied: the pixels change.
    text.apply = async (_sprite, grid) => {
      text.applied.push(grid);
      editor.setFrame(1, ["R.", ".R"]);
      editor.revision++;
    };
    renderEditor(editor, { textApi: text });
    fireEvent.click(await screen.findByRole("button", { name: "Text" }));
    await screen.findByRole("textbox", { name: "Text grid" });
    text.clipboard = "frame 1\nR.\n.R\n";
    // The top bar has a Paste button too: take the one of the Text view.
    fireEvent.click(
      screen
        .getAllByRole("button", { name: "Paste" })
        .find((b) => !b.closest('[role="toolbar"]'))!,
    );
    const apply = screen.getByRole("button", { name: "Apply" });
    await waitFor(() =>
      expect((apply as HTMLButtonElement).disabled).toBe(false),
    );
    fireEvent.click(apply);
    await waitFor(() => expect(text.applied).toHaveLength(1));
    fireEvent.click(screen.getByRole("button", { name: "Canvas" }));
    await waitFor(() =>
      expect(colorOfPixel(2, 2, 0, 0).slice(0, 3)).toEqual([240, 69, 69]),
    );
    expect(colorOfPixel(2, 2, 1, 0)[0]).not.toBe(240);
  });
});
