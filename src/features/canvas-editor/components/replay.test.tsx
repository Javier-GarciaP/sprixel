// @vitest-environment jsdom
import { act, cleanup, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { AgentOps } from "../api/types";
import { FakeEditorApi } from "../testing/fakeApi";
import { centeredView, fitZoom } from "../view/viewMath";
import {
  colorOfPixel,
  drag,
  goblin,
  manualClock,
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
const RED = [255, 0, 0, 255];

async function open(api: FakeEditorApi) {
  renderEditor(api);
  await screen.findByTestId("canvas-area");
  // The editor listens to the agent events once it is open.
  await waitFor(() => expect(api.listeningToOps).toBe(true));
  return api;
}

const ops = (
  id: number,
  pixels: [number, number, string][],
  extra: Partial<AgentOps> = {},
): AgentOps => ({
  id,
  agent: "Claude Code",
  sprite: goblin,
  frame: 1,
  tool: "apply_grid",
  pixels,
  ...extra,
});

// What the core does for a write: it stores the pixels, marks the frame busy
// and sends the ops. The screen only learns it from the events.
function agentWrites(api: FakeEditorApi, write: AgentOps) {
  const chars = [...api.frames[write.frame - 1]];
  const width = api.width;
  for (const [x, y, key] of write.pixels) chars[y * width + x] = key;
  api.frames[write.frame - 1] = chars.join("");
  act(() => {
    api.emit({ agent: write.agent, sprite: write.sprite, frame: write.frame });
    api.emitOps(write);
  });
}

const tick = (ms: number) => act(() => manualClock.tick(ms));

describe("Replay of agent writes", () => {
  // Checks the "Grid" scenario: the pixels of a grid appear in order while time
  // passes (not all at once), the editor tells the core when the replay is
  // done, and at the end the canvas has the same pixels as the stored frame.
  it("shows the pixels in order and ends equal to the stored frame", async () => {
    const api = await open(new FakeEditorApi(4, 4, 1, palette));
    const grid: [number, number, string][] = [];
    for (let y = 0; y < 3; y++)
      for (let x = 0; x < 4; x++) grid.push([x, y, "R"]);

    agentWrites(api, ops(1, grid));

    // Nothing is drawn yet: the first pixel needs a moment.
    expect(colorOfPixel(4, 4, 0, 0)).not.toEqual(RED);
    // 12 pixels last 360 ms: at 100 ms the first 3 are there, the rest not.
    tick(100);
    expect(colorOfPixel(4, 4, 2, 0)).toEqual(RED);
    expect(colorOfPixel(4, 4, 3, 0)).not.toEqual(RED);
    expect(colorOfPixel(4, 4, 0, 2)).not.toEqual(RED);
    expect(api.playbackDones).toEqual([]);

    tick(300);
    await waitFor(() => expect(api.playbackDones).toEqual([1]));
    // The frame is free, the sprite is read again, and the canvas matches it.
    await waitFor(() => expect(api.activity).toBeNull());
    for (let y = 0; y < 4; y++) {
      for (let x = 0; x < 4; x++) {
        const stored = api.rows(1)[y][x] === "R";
        await waitFor(() =>
          expect(colorOfPixel(4, 4, x, y)[0] === 255).toBe(stored),
        );
      }
    }
  });

  // Checks the "Big write" scenario: 5000 pixels do not take more than 8
  // seconds of the clock, and the replay is not over just before that.
  it("ends within 8 seconds for 5000 pixels", async () => {
    const api = await open(new FakeEditorApi(100, 100, 1, palette));
    const pixels: [number, number, string][] = [];
    for (let i = 0; i < 5000; i++)
      pixels.push([i % 100, Math.floor(i / 100), "R"]);

    agentWrites(api, ops(7, pixels));
    tick(7900);
    expect(api.playbackDones).toEqual([]);
    tick(100);

    await waitFor(() => expect(api.playbackDones).toEqual([7]));
  });

  // Checks that writes in a row play one after the other, in order: the second
  // does not start until the first ends.
  it("plays queued writes one after the other", async () => {
    const api = await open(new FakeEditorApi(4, 4, 1, palette));
    agentWrites(api, ops(1, [[0, 0, "R"]]));
    agentWrites(api, ops(2, [[1, 0, "R"]]));

    tick(10);
    expect(colorOfPixel(4, 4, 1, 0)).not.toEqual(RED);
    tick(260);
    await waitFor(() => expect(api.playbackDones).toEqual([1]));
    tick(260);
    await waitFor(() => expect(api.playbackDones).toEqual([1, 2]));
  });

  // Checks the queue rule: with more than 3 writes waiting, the editor shows
  // the final state at once, with no time passing, and tells the core about
  // every write in order.
  it("jumps to the final state when more than 3 writes wait", async () => {
    const api = await open(new FakeEditorApi(4, 4, 1, palette));
    for (let id = 1; id <= 5; id++) {
      agentWrites(api, ops(id, [[id - 1, 0, "R"]]));
    }

    await waitFor(() => expect(api.playbackDones).toEqual([1, 2, 3, 4, 5]));
  });
});

describe("Agent cursor", () => {
  // Checks that the cursor carries the agent name and sits on the pixel being
  // drawn, using the zoom and offset of the view (the sprite is fitted and
  // centered in the 500x300 area, so a 4x4 sprite has zoom 75).
  it("shows the agent name on the pixel being drawn", async () => {
    const api = await open(new FakeEditorApi(4, 4, 1, palette));
    agentWrites(
      api,
      ops(1, [
        [0, 0, "R"],
        [1, 1, "R"],
        [3, 2, "R"],
        [3, 3, "R"],
      ]),
    );

    // 4 pixels last 250 ms: at 200 ms three are drawn, the last one is (3, 2).
    tick(200);

    const cursor = await screen.findByTestId("agent-cursor");
    expect(cursor.textContent).toBe("Claude Code");
    const view = centeredView(4, 4, fitZoom(4, 4, 500, 300), 500, 300);
    expect(cursor.style.left).toBe(`${view.x + 3 * view.zoom}px`);
    expect(cursor.style.top).toBe(`${view.y + 2 * view.zoom}px`);
    expect(cursor.style.width).toBe(`${view.zoom}px`);
  });

  // Checks that the cursor goes away when the replay ends.
  it("goes away at the end", async () => {
    const api = await open(new FakeEditorApi(4, 4, 1, palette));
    agentWrites(
      api,
      ops(1, [
        [0, 0, "R"],
        [1, 0, "R"],
      ]),
    );
    tick(200);
    expect(await screen.findByTestId("agent-cursor")).toBeTruthy();

    tick(100);

    await waitFor(() =>
      expect(screen.queryByTestId("agent-cursor")).toBeNull(),
    );
  });
});

describe("Busy during the replay", () => {
  const line = (count: number): [number, number, string][] =>
    Array.from({ length: count }, (_, x) => [x, 0, "R"]);

  // Checks the "Person draws during replay" scenario: a stroke on the frame
  // that is being replayed is held (it shows on the canvas but nothing is sent),
  // and it is sent after the replay ends and the core frees the frame.
  it("holds a stroke of the person until the replay ends", async () => {
    const api = await open(new FakeEditorApi(16, 16, 1, palette));
    agentWrites(api, ops(1, line(10)));
    tick(100);

    drag([16, 16], [[8, 8]]);

    expect(api.calls).toEqual([]);
    expect(colorOfPixel(16, 16, 8, 8)).toEqual([0, 0, 0, 255]);
    tick(300);
    await waitFor(() => expect(api.playbackDones).toEqual([1]));
    await waitFor(() => expect(api.calls).toHaveLength(1));
    expect(api.calls[0].changes[0]).toMatchObject({ x: 8, y: 8 });
  });

  // Checks the core timeout (10 s): if the core frees the frame while the
  // editor is still replaying (the replays took too long), the canvas shows
  // the final state at once and the held stroke is sent.
  it("shows the final state and sends held strokes when the core frees the frame", async () => {
    const api = await open(new FakeEditorApi(16, 16, 1, palette));
    agentWrites(api, ops(1, line(10)));
    tick(100);
    drag([16, 16], [[8, 8]]);
    expect(colorOfPixel(16, 16, 9, 0)).not.toEqual(RED);

    act(() => api.emit(null));

    await waitFor(() => expect(colorOfPixel(16, 16, 9, 0)).toEqual(RED));
    await waitFor(() => expect(api.calls).toHaveLength(1));
  });
});
