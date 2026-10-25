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
import {
  drag,
  manualClock,
  mockLayout,
  putImageData,
  renderEditor,
} from "../testing/renderEditor";

beforeEach(() => mockLayout(500, 300));
afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
  vi.unstubAllGlobals();
});

type Mode = "loop" | "once" | "pingpong";

// A sprite of `frames` frames with the animation "walk" over 1 to 4 at 10 fps,
// so every step lasts 100 ms.
async function open(mode: Mode, frames = 4) {
  const api = new FakeEditorApi(8, 8, frames);
  api.tags = [
    { name: "walk", from: 1, to: 4, fps: 10, mode },
    { name: "idle", from: 6, to: 7, fps: 10, mode: "loop" },
  ];
  renderEditor(api);
  const select = (await screen.findByRole("combobox", {
    name: "Animation",
  })) as HTMLSelectElement;
  await waitFor(() => expect(select.options).toHaveLength(3));
  fireEvent.change(select, { target: { value: "walk" } });
  await screen.findByRole("button", { name: "Play" });
  return api;
}

const shown = () =>
  Number(screen.getByTestId("preview").getAttribute("data-frame"));
const pass = (ms: number) => act(() => manualClock.tick(ms));
const play = () =>
  fireEvent.click(screen.getByRole("button", { name: "Play" }));
// The frames shown after each 100 ms for `steps` steps.
const sequence = (steps: number) => {
  const seen = [shown()];
  for (let i = 0; i < steps; i++) {
    pass(100);
    seen.push(shown());
  }
  return seen;
};

describe("Preview", () => {
  // Checks that it starts paused on the first frame of the animation.
  it("starts paused on the first frame", async () => {
    await open("loop");
    expect(shown()).toBe(1);
    pass(500);
    expect(shown()).toBe(1);
  });

  // Checks the "loop" mode: 1 2 3 4 and then again from 1, at 100 ms each.
  it("repeats a loop animation", async () => {
    await open("loop");
    play();
    await waitFor(() =>
      expect(screen.getByRole("button", { name: "Pause" })).toBeTruthy(),
    );
    expect(sequence(9)).toEqual([1, 2, 3, 4, 1, 2, 3, 4, 1, 2]);
  });

  // Checks the "once" mode: it plays 1 2 3 4, stays on the last frame, and the
  // button goes back to Play.
  it("plays an animation once and stops on the last frame", async () => {
    await open("once");
    play();
    expect(sequence(6)).toEqual([1, 2, 3, 4, 4, 4, 4]);
    await waitFor(() =>
      expect(screen.getByRole("button", { name: "Play" })).toBeTruthy(),
    );
  });

  // Checks the "Play" scenario: ping-pong goes forward and then backward,
  // repeatedly (1 2 3 4 3 2 1 2 3 4 ...).
  it("plays a ping-pong animation forward and then backward, repeatedly", async () => {
    await open("pingpong");
    play();
    expect(sequence(9)).toEqual([1, 2, 3, 4, 3, 2, 1, 2, 3, 4]);
  });

  // Checks that pause stops the playing and keeps the frame that is shown.
  it("pauses", async () => {
    await open("loop");
    play();
    pass(100);
    expect(shown()).toBe(2);
    fireEvent.click(await screen.findByRole("button", { name: "Pause" }));
    pass(1000);
    expect(shown()).toBe(2);
  });

  // Checks the speed control (1 to 60 fps): it starts with the speed of the
  // animation, and 20 fps makes each step last 50 ms.
  it("has a speed control from 1 to 60 fps", async () => {
    await open("loop");
    const speed = screen.getByRole("slider", {
      name: "Speed",
    }) as HTMLInputElement;
    expect([speed.min, speed.max, speed.value]).toEqual(["1", "60", "10"]);
    fireEvent.change(speed, { target: { value: "20" } });
    expect(await screen.findByText("20 fps")).toBeTruthy();
    play();
    pass(50);
    expect(shown()).toBe(2);
    pass(50);
    expect(shown()).toBe(3);
  });

  // Checks the "Filter by animation" scenario for the preview: on a sprite
  // with 7 frames, "walk" plays only frames 1 to 4, and "idle" only 6 and 7.
  it("plays only the frames of the chosen animation", async () => {
    await open("loop", 7);
    play();
    expect(new Set(sequence(12))).toEqual(new Set([1, 2, 3, 4]));
    fireEvent.change(screen.getByRole("combobox", { name: "Animation" }), {
      target: { value: "idle" },
    });
    await waitFor(() => expect(shown()).toBe(6));
    fireEvent.click(await screen.findByRole("button", { name: "Play" }));
    expect(new Set(sequence(6))).toEqual(new Set([6, 7]));
  });

  // Checks that with "All" the preview plays every frame, repeating, at 12 fps
  // (83 ms for a frame that has no duration of its own).
  it("plays all the frames when no animation is chosen", async () => {
    const api = new FakeEditorApi(8, 8, 3);
    renderEditor(api);
    await waitFor(() =>
      expect(screen.getByTestId("preview").getAttribute("data-ready")).toBe(
        "true",
      ),
    );
    fireEvent.click(screen.getByRole("button", { name: "Play" }));
    const seen = [shown()];
    for (let i = 0; i < 4; i++) {
      pass(84);
      seen.push(shown());
    }
    expect(seen).toEqual([1, 2, 3, 1, 2]);
  });
});

describe("Preview follows the drawing", () => {
  // The preview draws at 200 pixels wide; this is its last picture, and the
  // alpha of the pixel at the center of the sprite pixel (x, y) (25 px each).
  const previewAlpha = (x: number, y: number) => {
    const calls = putImageData.mock.calls.filter(
      (call) => (call[0] as { width: number }).width === 200,
    );
    const image = calls[calls.length - 1][0] as { data: Uint8ClampedArray };
    return image.data[((y * 25 + 12) * 200 + x * 25 + 12) * 4 + 3];
  };

  // Checks the "Edit while playing" scenario: while the preview plays, a stroke
  // on frame 1 is read again from the core and the preview shows the new pixel
  // the next time it comes to frame 1.
  it("shows a new pixel on the next loop", async () => {
    const api = new FakeEditorApi(8, 8, 2);
    renderEditor(api);
    await waitFor(() =>
      expect(screen.getByTestId("preview").getAttribute("data-ready")).toBe(
        "true",
      ),
    );
    fireEvent.click(screen.getByRole("button", { name: "Play" }));
    pass(84);
    expect(shown()).toBe(2);
    await waitFor(() => expect(previewAlpha(1, 1)).toBe(0));

    // The user is on frame 1 and draws while the preview plays.
    await screen.findByText("3200%");
    drag([8, 8], [[1, 1]]);
    await waitFor(() => expect(api.calls).toHaveLength(1));
    await waitFor(() => expect(api.rows(1)[1][1]).not.toBe("."));

    // Next loop: frame 1 comes back with the new pixel.
    await waitFor(() => {
      pass(84);
      expect(shown()).toBe(1);
      expect(previewAlpha(1, 1)).toBe(255);
    });
  });

  // Checks that the preview reads the frames again after an undo too (a change
  // that does not come from the canvas).
  it("reads the frames again after an undo", async () => {
    const api = new FakeEditorApi(8, 8, 1);
    renderEditor(api);
    await screen.findByText("3200%");
    drag([8, 8], [[1, 1]]);
    await waitFor(() => expect(api.calls).toHaveLength(1));
    await waitFor(() => expect(previewAlpha(1, 1)).toBe(255));
    fireEvent.click(screen.getByRole("button", { name: "Undo" }));
    await waitFor(() => expect(previewAlpha(1, 1)).toBe(0));
  });
});
