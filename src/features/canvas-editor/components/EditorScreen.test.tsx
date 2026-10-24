// @vitest-environment jsdom
import { cleanup, screen, waitFor, within } from "@testing-library/react";
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

  // Checks that the frames column lists the frames and that the palette
  // panel shows the colors of the sprite.
  it("lists the frames and the palette colors", async () => {
    renderEditor(new FakeEditorApi(8, 8, 3, palette));
    await screen.findByRole("button", { name: "Frame 3" });
    expect(screen.getByRole("img", { name: "#F04545" })).toBeTruthy();
    expect(screen.getByRole("img", { name: "#2F6BFF" })).toBeTruthy();
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
