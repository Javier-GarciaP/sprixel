// @vitest-environment jsdom
import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { I18nProvider } from "@/shared/lib/i18n";
import { buildColorTable } from "../render/colors";
import type { Scene } from "../render/renderer";
import { pixelAt } from "../view/viewMath";
import { useCanvasView } from "../view/useCanvasView";
import { CanvasArea } from "./CanvasArea";
import { CanvasStatusBar } from "./CanvasStatusBar";
import { useState } from "react";

beforeEach(() => {
  // jsdom has no layout: the canvas area is 500x300.
  vi.spyOn(HTMLElement.prototype, "clientWidth", "get").mockReturnValue(500);
  vi.spyOn(HTMLElement.prototype, "clientHeight", "get").mockReturnValue(300);
  vi.spyOn(HTMLCanvasElement.prototype, "getContext").mockReturnValue(null);
});

afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
});

const scene: Scene = {
  width: 120,
  height: 30,
  pixels: new Uint8Array(120 * 30).fill(46),
  colors: buildColorTable([]),
  onion: null,
  grid: false,
  selection: null,
};

// The view logic and its two parts, wired like the editor does.
function Harness() {
  const canvas = useCanvasView(scene.width, scene.height);
  const [position, setPosition] = useState<{ x: number; y: number } | null>(
    null,
  );
  return (
    <I18nProvider language="en">
      <div data-testid="view" data-view={JSON.stringify(canvas.view)} />
      <CanvasArea
        scene={scene}
        view={canvas.view}
        onSize={(width, height) => canvas.setArea({ width, height })}
        onZoom={(direction, at) =>
          direction === 1 ? canvas.zoomIn(at) : canvas.zoomOut(at)
        }
        onMove={canvas.moveTo}
        onHover={setPosition}
      />
      <CanvasStatusBar
        position={position}
        zoom={canvas.view.zoom}
        onZoomIn={() => canvas.zoomIn()}
        onZoomOut={() => canvas.zoomOut()}
        onFit={canvas.fit}
      />
    </I18nProvider>
  );
}

const view = () =>
  JSON.parse(screen.getByTestId("view").getAttribute("data-view")!) as {
    zoom: number;
    x: number;
    y: number;
  };

describe("zoom", () => {
  // Checks the "Fit" scenario in the real component: a 120x30 sprite in a
  // 500x300 area starts at 400%, the largest whole zoom that fits.
  it("starts fitted with the largest whole zoom", async () => {
    render(<Harness />);
    expect(await screen.findByText("400%")).toBeTruthy();
  });

  // Checks the zoom buttons and "Fit": + goes to 500%, - goes back, and Fit
  // returns to 400% after a manual change.
  it("zooms with the buttons and returns with Fit", async () => {
    render(<Harness />);
    await screen.findByText("400%");
    fireEvent.click(screen.getByRole("button", { name: "Zoom in" }));
    await screen.findByText("500%");
    fireEvent.click(screen.getByRole("button", { name: "Zoom in" }));
    await screen.findByText("600%");
    fireEvent.click(screen.getByRole("button", { name: "Fit" }));
    await screen.findByText("400%");
  });

  // Checks the "Zoom at the pointer" scenario: zooming with Control and the
  // wheel keeps the sprite pixel under the pointer.
  it("keeps the pixel under the pointer with Control and the wheel", async () => {
    render(<Harness />);
    await screen.findByText("400%");
    const pointer = { x: 133, y: 77 };
    const before = pixelAt(view(), pointer);
    fireEvent.wheel(screen.getByTestId("canvas-area"), {
      ctrlKey: true,
      deltaY: -100,
      clientX: pointer.x,
      clientY: pointer.y,
    });
    await screen.findByText("500%");
    expect(pixelAt(view(), pointer)).toEqual(before);
  });

  // Checks that the wheel alone does not zoom.
  it("ignores the wheel without Control", async () => {
    render(<Harness />);
    await screen.findByText("400%");
    fireEvent.wheel(screen.getByTestId("canvas-area"), { deltaY: -100 });
    expect(screen.getByText("400%")).toBeTruthy();
  });

  // Checks that the buttons stop at the limits (3200% here, after many steps).
  it("stops at 3200%", async () => {
    render(<Harness />);
    await screen.findByText("400%");
    for (let i = 0; i < 40; i++) {
      fireEvent.click(screen.getByRole("button", { name: "Zoom in" }), {});
      if (
        (screen.getByRole("button", { name: "Zoom in" }) as HTMLButtonElement)
          .disabled
      )
        break;
    }
    await screen.findByText("3200%");
    expect(
      (screen.getByRole("button", { name: "Zoom in" }) as HTMLButtonElement)
        .disabled,
    ).toBe(true);
  });
});

describe("panning", () => {
  // Checks that holding Space and dragging with the left button moves the
  // view by the distance of the pointer. The key listener is attached in an
  // effect, so the key is sent inside waitFor until the cursor changes.
  it("moves the view with Space and drag", async () => {
    render(<Harness />);
    await screen.findByText("400%");
    const area = screen.getByTestId("canvas-area");
    await waitFor(() => {
      fireEvent.keyDown(window, { key: " " });
      expect(area.style.cursor).toBe("grab");
    });
    const start = view();
    fireEvent.pointerDown(area, {
      button: 0,
      clientX: 10,
      clientY: 10,
      pointerId: 1,
    });
    fireEvent.pointerMove(area, { clientX: 30, clientY: 25, pointerId: 1 });
    await waitFor(() =>
      expect(view()).toEqual({ ...start, x: start.x + 20, y: start.y + 15 }),
    );
    fireEvent.pointerUp(area, { pointerId: 1 });
    fireEvent.pointerMove(area, { clientX: 90, clientY: 90, pointerId: 1 });
    expect(view().x).toBe(start.x + 20);
  });

  // Checks that without Space the left button does not move the view.
  it("does not move the view with the left button alone", async () => {
    render(<Harness />);
    await screen.findByText("400%");
    const area = screen.getByTestId("canvas-area");
    const start = view();
    fireEvent.pointerDown(area, {
      button: 0,
      clientX: 10,
      clientY: 10,
      pointerId: 1,
    });
    fireEvent.pointerMove(area, { clientX: 50, clientY: 50, pointerId: 1 });
    expect(view()).toEqual(start);
  });

  // Checks that dragging with the middle button moves the view.
  it("moves the view with the middle button", async () => {
    render(<Harness />);
    await screen.findByText("400%");
    const area = screen.getByTestId("canvas-area");
    const start = view();
    fireEvent.pointerDown(area, {
      button: 1,
      clientX: 0,
      clientY: 0,
      pointerId: 1,
    });
    fireEvent.pointerMove(area, { clientX: -12, clientY: 6, pointerId: 1 });
    await waitFor(() =>
      expect(view()).toEqual({ ...start, x: start.x - 12, y: start.y + 6 }),
    );
  });

  // Checks that releasing Space goes back to the normal cursor.
  it("goes back to the normal cursor when Space is released", async () => {
    render(<Harness />);
    await screen.findByText("400%");
    const area = screen.getByTestId("canvas-area");
    await waitFor(() => {
      fireEvent.keyDown(window, { key: " " });
      expect(area.style.cursor).toBe("grab");
    });
    fireEvent.keyUp(window, { key: " " });
    await waitFor(() => expect(area.style.cursor).toBe("crosshair"));
  });
});

describe("coordinates", () => {
  // Checks the "Coordinates" scenario: over the pixel at column 3 and row 4
  // the bar shows "x: 3 y: 4". The sprite is at 400% and starts at (10, 90).
  it("shows the pixel under the pointer", async () => {
    render(<Harness />);
    await screen.findByText("400%");
    const area = screen.getByTestId("canvas-area");
    const { x, y } = view();
    fireEvent.pointerMove(area, {
      clientX: x + 3 * 4 + 1,
      clientY: y + 4 * 4 + 2,
      pointerId: 1,
    });
    expect((await screen.findByLabelText("Pointer position")).textContent).toBe(
      "x: 3 y: 4",
    );
  });

  // Checks that the bar shows dashes when the pointer is outside the sprite
  // or leaves the canvas.
  it("shows dashes outside the sprite", async () => {
    render(<Harness />);
    await screen.findByText("400%");
    const area = screen.getByTestId("canvas-area");
    const { x, y } = view();
    fireEvent.pointerMove(area, {
      clientX: x + 5,
      clientY: y + 5,
      pointerId: 1,
    });
    await screen.findByText("x: 1 y: 1");
    fireEvent.pointerMove(area, {
      clientX: x - 5,
      clientY: y + 5,
      pointerId: 1,
    });
    await screen.findByText("x: – y: –");
    fireEvent.pointerMove(area, {
      clientX: x + 5,
      clientY: y + 5,
      pointerId: 1,
    });
    await screen.findByText("x: 1 y: 1");
    fireEvent.pointerLeave(area);
    await screen.findByText("x: – y: –");
  });
});
