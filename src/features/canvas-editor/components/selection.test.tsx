// @vitest-environment jsdom
import { cleanup, fireEvent, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { FakeEditorApi } from "../testing/fakeApi";
import { drag, mockLayout, renderEditor } from "../testing/renderEditor";

beforeEach(() => mockLayout(500, 300));
afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
  vi.unstubAllGlobals();
});

// The keys are listened to from an effect, so the first key can come too
// early. Pressing M (harmless, and the same every time) until the Select tool
// is active proves the handlers of all the commands are registered.
async function ready(api: FakeEditorApi) {
  renderEditor(api);
  await screen.findByText("1500%");
  await waitFor(() => {
    fireEvent.keyDown(window, { key: "m" });
    expect(screen.getByTestId("active-tool").textContent).toContain("Select");
  });
}

// A sprite of 32x4 with the Select tool and a 4x3 rectangle selected at 2,0.
async function withSelection(api = new FakeEditorApi(32, 4, 2)) {
  await ready(api);
  drag(
    [32, 4],
    [
      [2, 0],
      [5, 2],
    ],
  );
  await screen.findByRole("toolbar", { name: "Selection actions" });
  return api;
}

const last = (api: FakeEditorApi) => api.powerCalls[api.powerCalls.length - 1];

describe("Selection bar", () => {
  // Checks that the bar is only there while something is selected, and says
  // the size of the selection.
  it("shows the bar and the size only with a selection", async () => {
    await ready(new FakeEditorApi(32, 4));
    expect(
      screen.queryByRole("toolbar", { name: "Selection actions" }),
    ).toBeNull();
    drag(
      [32, 4],
      [
        [2, 0],
        [5, 2],
      ],
    );
    expect(await screen.findByText("Selection 4 × 3")).toBeTruthy();
  });

  // Checks that a button sends its action for the selected rectangle only
  // (not for the whole frame) and that the selection is still there after
  // the sprite is read again.
  it("flips only the selection and keeps it", async () => {
    const api = await withSelection();
    fireEvent.click(screen.getByRole("button", { name: "Flip horizontally" }));
    await waitFor(() => expect(api.powerCalls).toHaveLength(1));
    expect(last(api)).toEqual({
      name: "region",
      args: [
        { folder: ["Characters"], name: "goblin" },
        1,
        { x: 2, y: 0, width: 4, height: 3 },
        { kind: "flip_horizontal" },
      ],
    });
    await waitFor(() =>
      expect(screen.getByRole("button", { name: "Undo" })).toHaveProperty(
        "disabled",
        false,
      ),
    );
    expect(screen.getByText("Selection 4 × 3")).toBeTruthy();
  });

  // Checks every other button sends the right action.
  it("sends rotate and mirror actions", async () => {
    const api = await withSelection();
    const press = async (name: string, action: object) => {
      const before = api.powerCalls.length;
      fireEvent.click(screen.getByRole("button", { name }));
      await waitFor(() => expect(api.powerCalls).toHaveLength(before + 1));
      expect(last(api).args[3]).toEqual(action);
    };
    await press("Flip vertically", { kind: "flip_vertical" });
    await press("Rotate 90° right", { kind: "rotate", degrees: 90 });
    await press("Rotate 180°", { kind: "rotate", degrees: 180 });
    await press("Rotate 90° left", { kind: "rotate", degrees: 270 });
    await press("Mirror the left half onto the right", {
      kind: "mirror",
      direction: "left_to_right",
    });
    await press("Mirror the bottom half onto the top", {
      kind: "mirror",
      direction: "bottom_to_top",
    });
  });

  // Checks the clear message when the core says the selection is not square.
  it("tells the person when the selection cannot turn 90 degrees", async () => {
    const api = await withSelection();
    api.powerFailure = { code: "not_square" };
    fireEvent.click(screen.getByRole("button", { name: "Rotate 90° right" }));
    expect(await screen.findByText(/cannot turn 90°/)).toBeTruthy();
  });

  // Checks that nothing is sent while an agent draws on the frame.
  it("does not call the core while an agent is drawing", async () => {
    const api = await withSelection();
    // The editor listens from an effect: say it until the marker shows.
    await waitFor(() => {
      api.emit({
        agent: "Claude Code",
        sprite: { folder: ["Characters"], name: "goblin" },
        frame: 1,
      });
      expect(screen.getAllByRole("status").length).toBeGreaterThan(0);
    });
    fireEvent.click(screen.getByRole("button", { name: "Flip horizontally" }));
    expect(await screen.findByText(/agent is drawing/)).toBeTruthy();
    expect(api.powerCalls).toHaveLength(0);
  });

  // Checks the keyboard shortcuts of the bar.
  it("runs the actions with their keys", async () => {
    const api = await withSelection();
    fireEvent.keyDown(window, { key: "h" });
    await waitFor(() => expect(api.powerCalls).toHaveLength(1));
    expect(last(api).args[3]).toEqual({ kind: "flip_horizontal" });
    fireEvent.keyDown(window, { key: "H", shiftKey: true });
    await waitFor(() => expect(api.powerCalls).toHaveLength(2));
    expect(last(api).args[3]).toEqual({ kind: "flip_vertical" });
  });
});

describe("Move the selection with the arrow keys", () => {
  // Checks that each arrow moves the selected pixels one pixel, without wrap
  // at first.
  it("moves one pixel with each arrow", async () => {
    const api = await withSelection();
    const expectMove = async (
      key: string,
      dx: number,
      dy: number,
      n: number,
    ) => {
      fireEvent.keyDown(window, { key });
      await waitFor(() => expect(api.powerCalls).toHaveLength(n));
      expect(last(api).args[3]).toEqual({ kind: "move", dx, dy, wrap: false });
    };
    await expectMove("ArrowRight", 1, 0, 1);
    await expectMove("ArrowLeft", -1, 0, 2);
    await expectMove("ArrowDown", 0, 1, 3);
    await expectMove("ArrowUp", 0, -1, 4);
  });

  // Checks the wrap option: the check box and the W key turn it on and off,
  // and the move says so.
  it("wraps when the option is on", async () => {
    const api = await withSelection();
    fireEvent.click(screen.getByRole("checkbox", { name: "Wrap" }));
    fireEvent.keyDown(window, { key: "ArrowRight" });
    await waitFor(() => expect(api.powerCalls).toHaveLength(1));
    expect(last(api).args[3]).toEqual({
      kind: "move",
      dx: 1,
      dy: 0,
      wrap: true,
    });
    fireEvent.keyDown(window, { key: "w" });
    await waitFor(() =>
      expect(
        (screen.getByRole("checkbox", { name: "Wrap" }) as HTMLInputElement)
          .checked,
      ).toBe(false),
    );
  });

  // Checks that with nothing selected the arrow keys move the whole frame
  // (no region is sent).
  it("moves the whole frame when nothing is selected", async () => {
    const api = new FakeEditorApi(32, 4);
    await ready(api);
    fireEvent.keyDown(window, { key: "ArrowDown" });
    await waitFor(() => expect(api.powerCalls).toHaveLength(1));
    expect(last(api).args[2]).toBeNull();
  });
});
