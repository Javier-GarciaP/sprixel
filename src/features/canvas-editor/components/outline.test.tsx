// @vitest-environment jsdom
import { cleanup, fireEvent, screen, waitFor } from "@testing-library/react";
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
  { key: "K", hex: "#000000" },
  { key: "R", hex: "#FF0000" },
];

async function open() {
  const api = new FakeEditorApi(16, 16, 1, palette);
  api.preview = {
    pixels: [
      [3, 3, "K"],
      [4, 3, "K"],
    ],
  };
  renderEditor(api);
  await screen.findByText("1800%");
  return api;
}

const outlineButton = () => screen.getByRole("button", { name: "Outline" });
const calls = (api: FakeEditorApi, name: string) =>
  api.powerCalls.filter((c) => c.name === name);
const panel = () => screen.findByRole("dialog", { name: "Outline" });

describe("Outline panel", () => {
  // Checks the "Preview" scenario: opening the panel asks the core for a
  // preview with the default options (the darkest color, 8 neighbors), says
  // how many pixels will change and writes nothing.
  it("shows a preview and writes nothing", async () => {
    const api = await open();
    fireEvent.click(outlineButton());
    await panel();
    expect(await screen.findByText("Pixels to change: 2")).toBeTruthy();
    expect(calls(api, "outlinePreview")[0].args[2]).toEqual({
      key: "K",
      connectivity: 8,
      outsideOnly: false,
      inner: false,
    });
    expect(calls(api, "outlineApply")).toHaveLength(0);
    expect(api.revision).toBe(0);
  });

  // Checks that the canvas paints the preview: the pixel (3,3) turns dark.
  it("paints the preview pixels on the canvas", async () => {
    const api = await open();
    api.preview = { pixels: [[0, 0, "K"]] };
    fireEvent.click(outlineButton());
    await screen.findByText("Pixels to change: 1");
    await waitFor(() => {
      const [r, g, b] = pictureAt(250 - 8 * 18 + 9, 150 - 8 * 18 + 9);
      expect(Math.max(r, g, b)).toBeLessThan(90);
    });
  });

  // Checks that changing an option asks for a new preview with that option.
  it("asks for a new preview when an option changes", async () => {
    const api = await open();
    fireEvent.click(outlineButton());
    await panel();
    await screen.findByText("Pixels to change: 2");
    fireEvent.click(screen.getByRole("button", { name: "4 (sides)" }));
    fireEvent.click(
      screen.getByRole("checkbox", { name: "Skip holes (outside only)" }),
    );
    fireEvent.click(screen.getByRole("button", { name: "Color R (#FF0000)" }));
    await waitFor(() => {
      const list = calls(api, "outlinePreview");
      expect(list[list.length - 1].args[2]).toEqual({
        key: "R",
        connectivity: 4,
        outsideOnly: true,
        inner: false,
      });
    });
  });

  // Checks "Apply and undo": Apply sends the same options once, the panel
  // closes and the history can be undone.
  it("applies with the same options and closes", async () => {
    const api = await open();
    fireEvent.click(outlineButton());
    await panel();
    await screen.findByText("Pixels to change: 2");
    fireEvent.click(screen.getByRole("button", { name: "Apply" }));
    await waitFor(() => expect(calls(api, "outlineApply")).toHaveLength(1));
    expect(calls(api, "outlineApply")[0].args[2]).toMatchObject({ key: "K" });
    await waitFor(() =>
      expect(screen.queryByRole("dialog", { name: "Outline" })).toBeNull(),
    );
    await waitFor(() =>
      expect(
        (screen.getByRole("button", { name: "Undo" }) as HTMLButtonElement)
          .disabled,
      ).toBe(false),
    );
  });

  // Checks "Cancel": nothing is written and the preview goes away.
  it("changes nothing on Cancel and on Escape", async () => {
    const api = await open();
    fireEvent.click(outlineButton());
    await panel();
    await screen.findByText("Pixels to change: 2");
    fireEvent.click(screen.getByRole("button", { name: "Cancel" }));
    await waitFor(() =>
      expect(screen.queryByRole("dialog", { name: "Outline" })).toBeNull(),
    );
    expect(calls(api, "outlineApply")).toHaveLength(0);
    fireEvent.click(outlineButton());
    await panel();
    fireEvent.keyDown(window, { key: "Escape" });
    await waitFor(() =>
      expect(screen.queryByRole("dialog", { name: "Outline" })).toBeNull(),
    );
    expect(calls(api, "outlineApply")).toHaveLength(0);
    expect(api.revision).toBe(0);
  });

  // Checks that Apply is off when nothing would change, and a failure of the
  // core is shown in the panel.
  it("turns Apply off with nothing to change and shows failures", async () => {
    const api = await open();
    api.preview = { pixels: [] };
    fireEvent.click(outlineButton());
    await screen.findByText("Nothing would change.");
    expect(
      (screen.getByRole("button", { name: "Apply" }) as HTMLButtonElement)
        .disabled,
    ).toBe(true);
    api.preview = { pixels: [[1, 1, "K"]] };
    fireEvent.click(screen.getByRole("button", { name: "4 (sides)" }));
    await screen.findByText("Pixels to change: 1");
    api.powerFailure = { code: "frame_busy" };
    fireEvent.click(screen.getByRole("button", { name: "Apply" }));
    expect((await screen.findByRole("alert")).textContent).toMatch(
      /agent is drawing/,
    );
  });

  // Checks the key to open the panel (Shift+O).
  it("opens with Shift+O", async () => {
    await open();
    await waitFor(() => {
      fireEvent.keyDown(window, { key: "O", shiftKey: true });
      expect(screen.getByRole("dialog", { name: "Outline" })).toBeTruthy();
    });
  });
});
