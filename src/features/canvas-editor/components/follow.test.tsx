// @vitest-environment jsdom
import {
  cleanup,
  fireEvent,
  screen,
  waitFor,
  act,
} from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { AgentOps, SpriteRef } from "../api/types";
import { FakeEditorApi } from "../testing/fakeApi";
import {
  goblin,
  mockLayout,
  pixelPoint,
  renderEditor,
} from "../testing/renderEditor";

beforeEach(() => mockLayout(500, 300));
afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
  vi.unstubAllGlobals();
});

const slime: SpriteRef = { folder: [], name: "slime" };

const ops = (sprite: SpriteRef, frame: number, id = 1): AgentOps => ({
  id,
  agent: "Claude Code",
  sprite,
  frame,
  tool: "apply_grid",
  pixels: [[0, 0, "R"]],
});

async function open(
  sprite: SpriteRef | null,
  onFollow?: (sprite: SpriteRef, frame: number) => void,
) {
  const api = new FakeEditorApi(16, 16, 3, [{ key: "R", hex: "#FF0000" }]);
  renderEditor(api, { sprite, onFollow });
  // Two listeners (replay and follow) wait for the events once it is open.
  await waitFor(() => expect(api.listeningToOps).toBe(true));
  if (sprite) await screen.findByRole("button", { name: "Frame 3" });
  return api;
}

// The person is in the middle of a stroke: the pointer is down.
function startStroke() {
  const area = screen.getByTestId("canvas-area");
  fireEvent.pointerDown(area, { ...pixelPoint(16, 16, 4, 4), button: 0 });
}

const currentFrame = () =>
  screen
    .getAllByRole("button", { name: /^Frame \d$/ })
    .find((b) => b.getAttribute("aria-current") === "true")
    ?.getAttribute("aria-label");

describe("Follow the agent", () => {
  // Checks that when an agent edits another sprite and the person is free,
  // the editor asks the app to show that sprite and frame, and the core is
  // told the replay is done (the sprite shown next reads the final state).
  it("follows an agent to another sprite when the person is free", async () => {
    const onFollow = vi.fn();
    const api = await open(goblin, onFollow);

    act(() => api.emitOps(ops(slime, 2)));

    await waitFor(() => expect(onFollow).toHaveBeenCalledWith(slime, 2));
    expect(api.playbackDones).toEqual([1]);
  });

  // Checks the "Person is busy" scenario: with a stroke in progress the agent
  // edits another sprite and the editor stays where it is (the frame is still
  // freed in the core).
  it("stays on the sprite of the person while they draw", async () => {
    const onFollow = vi.fn();
    const api = await open(goblin, onFollow);
    startStroke();

    act(() => api.emitOps(ops(slime, 1)));

    await waitFor(() => expect(api.playbackDones).toEqual([1]));
    expect(onFollow).not.toHaveBeenCalled();
  });

  // Checks that with no sprite open (empty editor) an agent still takes the
  // person to the sprite it edits.
  it("follows an agent when no sprite is open", async () => {
    const onFollow = vi.fn();
    const api = await open(null, onFollow);

    act(() => api.emitOps(ops(slime, 1)));

    await waitFor(() => expect(onFollow).toHaveBeenCalledWith(slime, 1));
  });

  // Checks that the sprite that is open is not "followed" again: a write to it
  // is for the replay, not for the app.
  it("does not ask the app to show the sprite that is already open", async () => {
    const onFollow = vi.fn();
    const api = await open(goblin, onFollow);

    act(() => api.emitOps(ops(goblin, 1)));

    expect(onFollow).not.toHaveBeenCalled();
  });

  // Checks that an agent that edits another frame of the open sprite takes
  // the person to that frame when they are free.
  it("follows an agent to another frame of the same sprite", async () => {
    const api = await open(goblin);
    expect(currentFrame()).toBe("Frame 1");

    act(() => api.emitOps(ops(goblin, 3)));

    await waitFor(() => expect(currentFrame()).toBe("Frame 3"));
    expect(api.playbackDones).toEqual([1]);
  });

  // Checks that the frame does not change under the hand of the person.
  it("stays on the frame of the person while they draw", async () => {
    const api = await open(goblin);
    startStroke();

    act(() => api.emitOps(ops(goblin, 3)));

    await waitFor(() => expect(api.playbackDones).toEqual([1]));
    expect(currentFrame()).toBe("Frame 1");
  });
});
