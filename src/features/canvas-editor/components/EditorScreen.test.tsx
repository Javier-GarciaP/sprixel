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
import { mockLayout, pictureAt, renderEditor } from "../testing/renderEditor";

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

describe("editor layout", () => {
  // Checks the "Open a sprite" scenario: the tools, the frames column, the
  // canvas bar, the canvas and the four parts of the right panel are all
  // visible once the sprite is open.
  it("shows every area when a sprite is opened", async () => {
    renderEditor(new FakeEditorApi(16, 16, 2, palette));
    await screen.findByRole("complementary", { name: "Tools" });
    expect(screen.getByRole("complementary", { name: "Frames" })).toBeTruthy();
    expect(screen.getByText("goblin")).toBeTruthy();
    expect(screen.getByTestId("canvas-area")).toBeTruthy();
    const panel = screen.getByRole("complementary", { name: "Editor panel" });
    for (const title of [
      "Preview",
      "Palette",
      "Onion skin",
      "Agent activity",
    ]) {
      expect(within(panel).getByRole("heading", { name: title })).toBeTruthy();
    }
  });

  // Checks the "canvas is centered" part: a 16x16 sprite in a 500x300 area
  // is drawn at 1800% (288 px), so it starts at x = 106 and y = 6.
  it("centers the canvas", async () => {
    renderEditor(new FakeEditorApi(16, 16, 1, palette));
    await screen.findByTestId("canvas-area");
    await waitFor(() => {
      expect(pictureAt(106, 6)[3]).toBe(255);
    });
    expect(pictureAt(105, 6)[3]).toBe(0);
    expect(pictureAt(106, 5)[3]).toBe(0);
    expect(pictureAt(106 + 287, 6 + 287)[3]).toBe(255);
    expect(pictureAt(106 + 288, 6)[3]).toBe(0);
  });

  // Checks that the "pick a color" hint floats over the canvas area, so when
  // it goes away after the first pixel the canvas does not jump.
  it("shows the start hint without taking space from the canvas", async () => {
    renderEditor(new FakeEditorApi(16, 16, 1, palette));
    const hint = await screen.findByText(/To start: pick a color/);
    expect(hint.className).toContain("absolute");
  });

  // Checks that the frames column lists the frames and that the palette
  // panel shows the colors of the sprite.
  it("lists the frames and the palette colors", async () => {
    renderEditor(new FakeEditorApi(8, 8, 3, palette));
    await screen.findByRole("button", { name: "Frame 3" });
    expect(screen.getByRole("button", { name: "#F04545" })).toBeTruthy();
    expect(screen.getByRole("button", { name: "#2F6BFF" })).toBeTruthy();
    expect(screen.getByRole("img", { name: "Transparent" })).toBeTruthy();
  });

  // Checks the empty states: no sprite open, and a sprite that cannot be
  // read.
  it("explains when no sprite is open", async () => {
    renderEditor(new FakeEditorApi(8, 8), { sprite: null });
    expect(await screen.findByText("No sprite is open")).toBeTruthy();
  });

  it("shows an error when the sprite cannot be read", async () => {
    const api = new FakeEditorApi(8, 8);
    api.loadSprite = () =>
      Promise.reject({ code: "not_found", message: "gone" });
    renderEditor(api);
    expect((await screen.findByRole("alert")).textContent).toContain("gone");
  });

  // Checks that the screen follows the language: the Spanish names.
  it("uses the Spanish texts", async () => {
    renderEditor(new FakeEditorApi(8, 8), { language: "es" });
    expect(
      await screen.findByRole("complementary", { name: "Herramientas" }),
    ).toBeTruthy();
    expect(
      screen.getByRole("heading", { name: "Piel de cebolla" }),
    ).toBeTruthy();
  });
});

describe("canvas bar", () => {
  // Checks the bar contents: the name, the size next to it as plain text (not
  // a menu), the active tool and the undo and redo buttons (off at the start).
  it("shows the name, the size, the active tool and undo and redo", async () => {
    renderEditor(new FakeEditorApi(16, 12, 1, palette));
    expect(await screen.findByText("16 × 12 px")).toBeTruthy();
    expect(screen.getByText("goblin")).toBeTruthy();
    expect(screen.queryByRole("combobox", { name: /size/i })).toBeNull();
    expect(screen.getByTestId("active-tool").textContent).toBe("Pencil · 1 px");
    expect(
      (screen.getByRole("button", { name: "Undo" }) as HTMLButtonElement)
        .disabled,
    ).toBe(true);
    expect(
      (screen.getByRole("button", { name: "Redo" }) as HTMLButtonElement)
        .disabled,
    ).toBe(true);
  });

  // Checks that choosing a tool in the column updates the name in the bar and
  // highlights the tool.
  it("shows the tool that is chosen", async () => {
    renderEditor(new FakeEditorApi(8, 8));
    const eraser = await screen.findByRole("button", { name: "Eraser" });
    fireEvent.click(eraser);
    await waitFor(() =>
      expect(screen.getByTestId("active-tool").textContent).toBe(
        "Eraser · 1 px",
      ),
    );
    expect(eraser.getAttribute("aria-pressed")).toBe("true");
    expect(
      screen
        .getByRole("button", { name: "Pencil" })
        .getAttribute("aria-pressed"),
    ).toBe("false");
  });

  // Checks the "name and key shown when the pointer rests on it": every tool
  // has a title with its name and its key.
  it("gives every tool a title with its name and key", async () => {
    renderEditor(new FakeEditorApi(8, 8));
    await screen.findByRole("complementary", { name: "Tools" });
    const expected: Record<string, string> = {
      Pencil: "B",
      Eraser: "E",
      Bucket: "G",
      Eyedropper: "I",
      Line: "L",
      Rectangle: "R",
      Ellipse: "C",
      Select: "M",
      Move: "V",
    };
    for (const [name, key] of Object.entries(expected)) {
      expect(screen.getByRole("button", { name }).getAttribute("title")).toBe(
        `${name} (${key})`,
      );
    }
  });

  // Checks that the Text switch is there but disabled until its spec exists.
  it("has the Canvas and Text switch", async () => {
    renderEditor(new FakeEditorApi(8, 8));
    await screen.findByRole("button", { name: "Canvas" });
    expect(
      (screen.getByRole("button", { name: "Text" }) as HTMLButtonElement)
        .disabled,
    ).toBe(true);
  });
});
