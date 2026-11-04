// @vitest-environment jsdom
import { cleanup, fireEvent, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { FakeEditorApi } from "../testing/fakeApi";
import { mockLayout, pixelPoint, renderEditor } from "../testing/renderEditor";

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

async function open(api = new FakeEditorApi(16, 16, 1, palette)) {
  renderEditor(api);
  await screen.findByText("1800%");
  return api;
}

// The Control is "main" or "secondary".
const swatch = (which: "main" | "secondary") =>
  screen.getByTestId(`${which}-color`);

// Presses a key once the key listener is attached (it is attached in an
// effect), repeating until `done` is true.
async function press(key: string, done: () => void) {
  await waitFor(() => {
    fireEvent.keyDown(window, { key });
    done();
  });
}

function click(x: number, y: number, button = 0) {
  const area = screen.getByTestId("canvas-area");
  fireEvent.pointerDown(area, { ...pixelPoint(16, 16, x, y), button });
  fireEvent.pointerUp(area, pixelPoint(16, 16, x, y));
}

describe("Tool keys", () => {
  // Checks the "Switch tool by key" scenario: pressing E makes the Eraser the
  // active tool.
  it("changes the tool with its key", async () => {
    await open();
    await press("e", () =>
      expect(screen.getByTestId("active-tool").textContent).toContain("Eraser"),
    );
    await press("g", () =>
      expect(screen.getByTestId("active-tool").textContent).toContain("Bucket"),
    );
  });

  // Checks that tool keys are ignored while a text field has the focus (the
  // hex field), so typing a color does not change tools.
  it("does not change the tool while typing in a field", async () => {
    await open();
    const field = screen.getByLabelText("Hex color");
    field.focus();
    fireEvent.keyDown(field, { key: "e" });
    expect(screen.getByTestId("active-tool").textContent).toContain("Pencil");
  });
});

describe("Main and secondary colors", () => {
  // Checks the "Right click" scenario: the left button draws with the main
  // color and the right button with the secondary color.
  it("draws with the secondary color on the right button", async () => {
    const api = await open();
    click(1, 1, 0);
    click(2, 2, 2);
    await waitFor(() => expect(api.calls).toHaveLength(2));
    expect(api.calls[0].changes[0].color).toBe("#000000");
    expect(api.calls[1].changes[0].color).toBe("#FFFFFF");
  });

  // Checks that a click on a palette color sets the main color and a right
  // click sets the secondary color.
  it("chooses colors from the palette with click and right click", async () => {
    const api = await open();
    fireEvent.click(screen.getByRole("button", { name: "#F04545" }));
    fireEvent.contextMenu(screen.getByRole("button", { name: "#2F6BFF" }));
    await waitFor(() => {
      expect(swatch("main").style.background).toBe("rgb(240, 69, 69)");
      expect(swatch("secondary").style.background).toBe("rgb(47, 107, 255)");
    });
    click(1, 1, 0);
    click(2, 2, 2);
    await waitFor(() => expect(api.calls).toHaveLength(2));
    expect(api.calls.map((c) => c.changes[0].color)).toEqual([
      "#F04545",
      "#2F6BFF",
    ]);
  });

  // Checks that the swap button exchanges the two colors, and that the X key
  // does the same.
  it("swaps the colors with the button and with X", async () => {
    await open();
    fireEvent.click(screen.getByRole("button", { name: "#F04545" }));
    await waitFor(() =>
      expect(swatch("main").style.background).toBe("rgb(240, 69, 69)"),
    );
    fireEvent.click(screen.getByRole("button", { name: "Swap colors" }));
    await waitFor(() => {
      expect(swatch("main").style.background).toBe("rgb(255, 255, 255)");
      expect(swatch("secondary").style.background).toBe("rgb(240, 69, 69)");
    });
    await press("x", () =>
      expect(swatch("main").style.background).toBe("rgb(240, 69, 69)"),
    );
  });

  // Checks "any color": a valid hex typed in the field becomes the main color
  // (with or without #, any letter case), and an invalid one is marked and
  // changes nothing.
  it("accepts any color from the hex field", async () => {
    const api = await open();
    const field = screen.getByLabelText("Hex color") as HTMLInputElement;
    fireEvent.change(field, { target: { value: "#12" } });
    expect(field.getAttribute("aria-invalid")).toBe("true");
    expect(swatch("main").style.background).toBe("rgb(0, 0, 0)");
    fireEvent.change(field, { target: { value: "00ff7f" } });
    await waitFor(() =>
      expect(swatch("main").style.background).toBe("rgb(0, 255, 127)"),
    );
    click(3, 3);
    await waitFor(() => expect(api.calls).toHaveLength(1));
    expect(api.calls[0].changes[0].color).toBe("#00FF7F");
  });

  // Checks that the system color picker (the color boxes) sets each color.
  it("sets the colors with the color pickers", async () => {
    await open();
    fireEvent.change(screen.getByLabelText("Main color"), {
      target: { value: "#336699" },
    });
    fireEvent.change(screen.getByLabelText("Secondary color"), {
      target: { value: "#aabbcc" },
    });
    await waitFor(() => {
      expect(swatch("main").style.background).toBe("rgb(51, 102, 153)");
      expect(swatch("secondary").style.background).toBe("rgb(170, 187, 204)");
    });
  });
});

describe("Eyedropper", () => {
  // Checks the "Pick a color" scenario: clicking a red pixel makes the main
  // color that red; a click on a transparent pixel leaves it unchanged.
  it("sets the main color to the pixel under the pointer", async () => {
    const api = new FakeEditorApi(16, 16, 1, palette);
    const rows = Array.from({ length: 16 }, () => ".".repeat(16));
    rows[4] = "..R" + ".".repeat(13);
    api.setFrame(1, rows);
    await open(api);
    fireEvent.click(screen.getByRole("button", { name: "Eyedropper" }));
    click(2, 4);
    await waitFor(() =>
      expect(swatch("main").style.background).toBe("rgb(240, 69, 69)"),
    );
    click(9, 9);
    click(2, 4, 2);
    await waitFor(() =>
      expect(swatch("secondary").style.background).toBe("rgb(240, 69, 69)"),
    );
    expect(swatch("main").style.background).toBe("rgb(240, 69, 69)");
    expect(api.calls).toEqual([]);
  });
});
