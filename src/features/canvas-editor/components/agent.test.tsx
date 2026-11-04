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
  colorOfPixel,
  drag,
  goblin,
  mockLayout,
  renderEditor,
} from "../testing/renderEditor";

beforeEach(() => mockLayout(500, 300));
afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
  vi.unstubAllGlobals();
});

const palette = [{ key: "R", hex: "#FF0000" }];

async function open(api = new FakeEditorApi(16, 16, 2, palette)) {
  renderEditor(api);
  await screen.findByText("1800%");
  // The editor listens to the agent events once it is open.
  await waitFor(() => expect(listening(api)).toBe(true));
  return api;
}

// The fake keeps its listeners private; emitting with none is harmless, so
// this checks that one was added by looking at a probe event.
function listening(api: FakeEditorApi) {
  return (api as unknown as { listeners: Set<unknown> }).listeners.size > 0;
}

const write = (frame: number, agent = "Claude Code") => ({
  agent,
  sprite: goblin,
  frame,
});

describe("Agent marker", () => {
  // Checks the "Agent draws frame 2" scenario: when an agent starts to write
  // to frame 2, the label "Claude Code is drawing frame 2" and Stop appear,
  // the canvas gets the amber border, and both go when the work ends.
  it("shows the label, Stop and the border while an agent draws", async () => {
    const api = await open();
    expect(screen.queryByTestId("agent-border")).toBeNull();

    act(() => api.emit(write(2)));

    expect((await screen.findByRole("status")).textContent).toContain(
      "Claude Code is drawing frame 2",
    );
    expect(screen.getByRole("button", { name: "Stop" })).toBeTruthy();
    expect(screen.getByTestId("agent-border")).toBeTruthy();

    act(() => api.emit(null));

    await waitFor(() =>
      expect(screen.queryByRole("button", { name: "Stop" })).toBeNull(),
    );
    expect(screen.queryByTestId("agent-border")).toBeNull();
  });

  // Checks that the frame being written is marked as busy in the frames
  // column, and only that one.
  it("marks the busy frame", async () => {
    const api = await open();
    act(() => api.emit(write(2)));
    await waitFor(() =>
      expect(
        screen
          .getByRole("button", { name: "Frame 2" })
          .getAttribute("aria-busy"),
      ).toBe("true"),
    );
    expect(
      screen.getByRole("button", { name: "Frame 1" }).getAttribute("aria-busy"),
    ).toBeNull();
  });

  // Checks that an agent that is already writing when the editor opens is
  // shown too (the state is read once at the start).
  it("shows an agent that was already drawing when the editor opened", async () => {
    const api = new FakeEditorApi(16, 16, 2, palette);
    api.activity = write(1, "OpenCode");
    renderEditor(api);
    expect((await screen.findByRole("status")).textContent).toContain(
      "OpenCode is drawing frame 1",
    );
  });

  // Checks that the marker is only for the sprite that is open: an agent that
  // draws another sprite shows nothing here.
  it("ignores an agent that draws another sprite", async () => {
    const api = await open();
    act(() =>
      api.emit({
        agent: "Claude Code",
        sprite: { folder: [], name: "slime" },
        frame: 1,
      }),
    );
    await waitFor(() => expect(api.activity).not.toBeNull());
    expect(screen.queryByTestId("agent-border")).toBeNull();
  });

  // Checks the Stop button: it tells the core to stop the agent, and the
  // marker goes away when the core says the work ended.
  it("stops the agent with the Stop button", async () => {
    const api = await open();
    act(() => api.emit(write(1)));
    fireEvent.click(await screen.findByRole("button", { name: "Stop" }));
    await waitFor(() => expect(api.stopped).toBe(1));
    await waitFor(() =>
      expect(screen.queryByTestId("agent-border")).toBeNull(),
    );
  });

  // Checks the Agent activity box of the right panel: it says what the agent
  // is doing while it works.
  it("says in the right panel who is drawing", async () => {
    const api = await open();
    act(() => api.emit(write(2)));
    await waitFor(() =>
      expect(
        screen.getAllByText("Claude Code is drawing frame 2").length,
      ).toBeGreaterThan(1),
    );
  });
});

describe("Busy frame", () => {
  // Checks "drawing MUST be blocked while the frame is busy, with the stroke
  // applied when the agent finishes": the stroke shows on the canvas at once,
  // nothing is sent while the agent works, and the stroke is sent when the
  // agent ends.
  it("holds a stroke until the agent finishes", async () => {
    const api = await open();
    act(() => api.emit(write(1)));
    await screen.findByTestId("agent-border");

    drag(
      [16, 16],
      [
        [2, 2],
        [4, 2],
      ],
    );

    expect(colorOfPixel(16, 16, 3, 2)).toEqual([0, 0, 0, 255]);
    expect(api.calls).toEqual([]);
    expect(api.rows(1)[2]).toBe(".".repeat(16));

    act(() => api.emit(null));

    await waitFor(() => expect(api.calls).toHaveLength(1));
    expect(api.calls[0].frame).toBe(1);
    expect(api.calls[0].changes).toHaveLength(3);
    await waitFor(() => expect(api.rows(1)[2].slice(0, 5)).toBe("..AAA"));
  });

  // Checks that a stroke on a frame the agent is not writing is sent at once.
  it("does not hold strokes on other frames", async () => {
    const api = await open();
    act(() => api.emit(write(2)));
    await screen.findByTestId("agent-border");
    drag([16, 16], [[2, 2]]);
    await waitFor(() => expect(api.calls).toHaveLength(1));
  });

  // Checks the race: the core says the frame is busy before the screen knew.
  // The stroke is kept and sent when the agent ends.
  it("keeps a stroke the core refused as busy", async () => {
    const api = await open();
    api.activity = write(1);
    drag([16, 16], [[5, 5]]);
    await waitFor(() =>
      expect(colorOfPixel(16, 16, 5, 5)).toEqual([0, 0, 0, 255]),
    );
    expect(api.calls).toEqual([]);
    expect(screen.queryByRole("alert")).toBeNull();
    act(() => api.emit(null));
    await waitFor(() => expect(api.calls).toHaveLength(1));
  });

  // Checks that when the agent ends the frame is read again, so the pixels
  // the agent drew show on the canvas.
  it("shows what the agent drew when it ends", async () => {
    const api = await open();
    act(() => api.emit(write(1)));
    await screen.findByTestId("agent-border");
    const rows = Array.from({ length: 16 }, () => ".".repeat(16));
    rows[7] = ".".repeat(7) + "R" + ".".repeat(8);
    api.setFrame(1, rows);
    act(() => api.emit(null));
    await waitFor(() =>
      expect(colorOfPixel(16, 16, 7, 7)).toEqual([255, 0, 0, 255]),
    );
  });
});
