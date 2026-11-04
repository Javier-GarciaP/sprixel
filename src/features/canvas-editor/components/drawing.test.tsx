// @vitest-environment jsdom
import { cleanup, fireEvent, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { FakeEditorApi } from "../testing/fakeApi";
import {
  colorOfPixel,
  mockLayout,
  pictureAt,
  pixelPoint,
  renderEditor,
} from "../testing/renderEditor";

beforeEach(() => mockLayout(500, 300));
afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
  vi.unstubAllGlobals();
});

// Draws with the left (0) or right (2) button from one pixel to others.
function drag(
  size: [number, number],
  path: [number, number][],
  button = 0,
  extra: Record<string, unknown> = {},
) {
  const area = screen.getByTestId("canvas-area");
  const at = ([x, y]: [number, number]) => ({
    ...pixelPoint(size[0], size[1], x, y),
    ...extra,
  });
  fireEvent.pointerDown(area, { ...at(path[0]), button });
  for (const step of path.slice(1)) fireEvent.pointerMove(area, at(step));
  fireEvent.pointerUp(area, at(path[path.length - 1]));
}

describe("Pencil and Eraser in the editor", () => {
  // Checks the whole path of a stroke: the pixels show on the canvas while
  // drawing, and one call with only the changed pixels goes to the core.
  it("draws a stroke and sends one operation", async () => {
    const api = new FakeEditorApi(16, 16);
    renderEditor(api);
    await screen.findByText("1800%");
    drag(
      [16, 16],
      [
        [2, 3],
        [6, 3],
      ],
    );
    await waitFor(() => expect(api.calls).toHaveLength(1));
    expect(api.calls[0].changes.map((c) => [c.x, c.y])).toEqual([
      [2, 3],
      [3, 3],
      [4, 3],
      [5, 3],
      [6, 3],
    ]);
    expect(api.calls[0].changes.every((c) => c.color === "#000000")).toBe(true);
    expect(colorOfPixel(16, 16, 4, 3)).toEqual([0, 0, 0, 255]);
    expect(colorOfPixel(16, 16, 7, 3)).not.toEqual([0, 0, 0, 255]);
  });

  // Checks the "Fast stroke" scenario through the real events: one pointer
  // move across 100 pixels paints all of them.
  it("paints every pixel of a 100 pixel move", async () => {
    const api = new FakeEditorApi(128, 4);
    renderEditor(api);
    await screen.findByText("300%");
    drag(
      [128, 4],
      [
        [0, 1],
        [99, 1],
      ],
    );
    await waitFor(() => expect(api.calls).toHaveLength(1));
    expect(api.calls[0].changes).toHaveLength(100);
    expect(api.rows(1)[1].slice(0, 100)).toMatch(/^[^.]{100}$/);
  });

  // Checks the "Pointer leaves the canvas" scenario: the stroke goes outside
  // the sprite and only the pixels inside are sent.
  it("sends only the pixels inside the sprite", async () => {
    const api = new FakeEditorApi(16, 16);
    renderEditor(api);
    await screen.findByText("1800%");
    drag(
      [16, 16],
      [
        [14, 5],
        [40, 5],
        [-20, 5],
      ],
    );
    await waitFor(() => expect(api.calls).toHaveLength(1));
    const xs = api.calls[0].changes.map((c) => c.x).sort((a, b) => a - b);
    expect(xs).toEqual(Array.from({ length: 16 }, (_, i) => i));
  });

  // Checks the Eraser: after choosing it, a stroke over drawn pixels sends
  // transparent pixels (color null) and the canvas shows them cleared.
  it("erases with the Eraser tool", async () => {
    const api = new FakeEditorApi(16, 16);
    renderEditor(api);
    await screen.findByText("1800%");
    drag(
      [16, 16],
      [
        [1, 1],
        [4, 1],
      ],
    );
    await waitFor(() => expect(api.calls).toHaveLength(1));
    fireEvent.click(screen.getByRole("button", { name: "Eraser" }));
    await waitFor(() =>
      expect(screen.getByTestId("active-tool").textContent).toContain("Eraser"),
    );
    drag(
      [16, 16],
      [
        [2, 1],
        [3, 1],
      ],
    );
    await waitFor(() => expect(api.calls).toHaveLength(2));
    expect(api.calls[1].changes).toEqual([
      { x: 2, y: 1, color: null },
      { x: 3, y: 1, color: null },
    ]);
    expect(api.rows(1)[1].slice(0, 5)).toBe(".A..A");
  });

  // Checks that a click without moving still draws one pixel, and that the
  // context menu does not open on a right click (the right button draws).
  it("draws one pixel with a click", async () => {
    const api = new FakeEditorApi(16, 16);
    renderEditor(api);
    await screen.findByText("1800%");
    drag([16, 16], [[5, 5]]);
    await waitFor(() => expect(api.calls).toHaveLength(1));
    expect(api.calls[0].changes).toHaveLength(1);
    const menu = fireEvent.contextMenu(screen.getByTestId("canvas-area"));
    expect(menu).toBe(false);
  });
});

describe("Bucket in the editor", () => {
  // Checks the fill through the real events: a click inside an outline sends
  // one operation with only the inside pixels, and the canvas shows them.
  it("fills inside a closed outline with one operation", async () => {
    const api = new FakeEditorApi(8, 8, 1, [{ key: "K", hex: "#000000" }]);
    api.setFrame(1, [
      "........",
      "..KKKK..",
      "..K..K..",
      "..K..K..",
      "..KKKK..",
      "........",
      "........",
      "........",
    ]);
    renderEditor(api);
    await screen.findByText("3200%");
    fireEvent.click(screen.getByRole("button", { name: "Bucket" }));
    fireEvent.change(screen.getByLabelText("Hex color"), {
      target: { value: "#FF0000" },
    });
    await waitFor(() =>
      expect(screen.getByTestId("main-color").style.background).toBe(
        "rgb(255, 0, 0)",
      ),
    );
    const area = screen.getByTestId("canvas-area");
    const at = pixelPoint(8, 8, 3, 2);
    fireEvent.pointerDown(area, { ...at, button: 0 });
    fireEvent.pointerUp(area, at);
    await waitFor(() => expect(api.calls).toHaveLength(1));
    expect(api.calls[0].changes.map((c) => [c.x, c.y]).sort()).toEqual(
      [
        [3, 2],
        [4, 2],
        [3, 3],
        [4, 3],
      ].sort(),
    );
    expect(colorOfPixel(8, 8, 3, 2)).toEqual([255, 0, 0, 255]);
    expect(colorOfPixel(8, 8, 0, 0)[0]).not.toBe(255);
  });
});

describe("Shapes in the editor", () => {
  const shapeCall = async (
    tool: string,
    path: [number, number][],
    extra: Record<string, unknown> = {},
    beforeDrag?: () => void,
  ) => {
    const api = new FakeEditorApi(16, 16);
    renderEditor(api);
    await screen.findByText("1800%");
    fireEvent.click(screen.getByRole("button", { name: tool }));
    beforeDrag?.();
    drag([16, 16], path, 0, extra);
    await waitFor(() => expect(api.calls).toHaveLength(1));
    return api.calls[0].changes;
  };
  const extent = (changes: { x: number; y: number }[]) => [
    Math.max(...changes.map((c) => c.x)) - Math.min(...changes.map((c) => c.x)),
    Math.max(...changes.map((c) => c.y)) - Math.min(...changes.map((c) => c.y)),
  ];

  // Checks that a Line, a Rectangle and an Ellipse each send one operation
  // with the shape when the pointer is released.
  it("draws a line, a rectangle and an ellipse", async () => {
    expect(
      extent(
        await shapeCall("Line", [
          [1, 1],
          [8, 1],
        ]),
      ),
    ).toEqual([7, 0]);
    cleanup();
    expect(
      extent(
        await shapeCall("Rectangle", [
          [1, 1],
          [6, 4],
        ]),
      ),
    ).toEqual([5, 3]);
    cleanup();
    expect(
      extent(
        await shapeCall("Ellipse", [
          [1, 1],
          [6, 4],
        ]),
      ),
    ).toEqual([5, 3]);
  });

  // Checks the "Square with Shift" scenario through the real events: the
  // pointer carries the Shift key and the result is a square.
  it("makes a square when Shift is held", async () => {
    const changes = await shapeCall(
      "Rectangle",
      [
        [2, 2],
        [9, 5],
      ],
      {
        shiftKey: true,
      },
    );
    expect(extent(changes)).toEqual([7, 7]);
  });

  // Checks the fill option: the Filled switch appears for the Rectangle and
  // not for the Pencil, and a filled 4x3 rectangle sends 12 pixels (an outline
  // would send 10).
  it("fills the shape when Filled is on", async () => {
    const changes = await shapeCall(
      "Rectangle",
      [
        [1, 1],
        [4, 3],
      ],
      {},
      () => {
        fireEvent.click(screen.getByRole("button", { name: "Filled" }));
      },
    );
    expect(changes).toHaveLength(12);
    cleanup();
    const outline = await shapeCall("Rectangle", [
      [1, 1],
      [4, 3],
    ]);
    expect(outline).toHaveLength(10);
  });

  it("shows the shape style only for Rectangle and Ellipse", async () => {
    renderEditor(new FakeEditorApi(16, 16));
    await screen.findByText("1800%");
    expect(screen.queryByRole("group", { name: "Shape style" })).toBeNull();
    fireEvent.click(screen.getByRole("button", { name: "Ellipse" }));
    expect(
      await screen.findByRole("group", { name: "Shape style" }),
    ).toBeTruthy();
  });
});

describe("Select and Move in the editor", () => {
  async function drawn() {
    const api = new FakeEditorApi(16, 16);
    renderEditor(api);
    await screen.findByText("1800%");
    drag(
      [16, 16],
      [
        [2, 2],
        [3, 2],
      ],
    );
    await waitFor(() => expect(api.calls).toHaveLength(1));
    return api;
  }
  const useTool = async (name: string) => {
    fireEvent.click(screen.getByRole("button", { name }));
    await waitFor(() =>
      expect(screen.getByTestId("active-tool").textContent).toContain(name),
    );
  };

  // Checks that Select shows an outline on the edge of the chosen area (a
  // black or white dash, not the checkerboard gray), and that a click clears
  // it.
  it("outlines the selected area", async () => {
    await drawn();
    await useTool("Select");
    drag(
      [16, 16],
      [
        [5, 5],
        [8, 7],
      ],
    );
    // The top left screen pixel of the area: the zoom is 1800%, so the
    // center of a pixel is 9 screen pixels from its corner.
    const corner = (x: number, y: number) => {
      const point = pixelPoint(16, 16, x, y);
      return pictureAt(point.clientX - 9, point.clientY - 9);
    };
    const outline = corner(5, 5);
    expect(outline[0]).toBe(outline[1]);
    expect([0, 255]).toContain(outline[0]);
    drag([16, 16], [[12, 12]]);
    await waitFor(() => {
      const color = corner(5, 5);
      expect(color[0] === 0 || color[0] === 255).toBe(false);
    });
  });

  // Checks the "Move a selection" scenario through the real events: select
  // the two pixels, drag them 3 pixels to the right, and one operation clears
  // the old place and paints the new one.
  it("moves a selection 3 pixels to the right in one operation", async () => {
    const api = await drawn();
    await useTool("Select");
    drag(
      [16, 16],
      [
        [2, 2],
        [3, 2],
      ],
    );
    await useTool("Move");
    drag(
      [16, 16],
      [
        [2, 2],
        [5, 2],
      ],
    );
    await waitFor(() => expect(api.calls).toHaveLength(2));
    expect(api.rows(1)[2].slice(0, 8)).toBe(".....AA.");
    expect(
      api.calls[1].changes.map((c) => `${c.x},${c.y},${c.color}`).sort(),
    ).toEqual(["2,2,null", "3,2,null", "5,2,#000000", "6,2,#000000"]);
  });
});
