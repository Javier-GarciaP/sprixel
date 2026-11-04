// @vitest-environment jsdom
import { cleanup, fireEvent, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { FakeEditorApi } from "../testing/fakeApi";
import { drag, mockLayout, renderEditor } from "../testing/renderEditor";

// The selection actions are enabled once something is selected.
const selected = () =>
  waitFor(() =>
    expect(
      screen
        .getByRole("button", { name: "Flip horizontally" })
        .hasAttribute("disabled"),
    ).toBe(false),
  );

beforeEach(() => mockLayout(500, 300));
afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
  vi.unstubAllGlobals();
});

const piece = { palette: "main", width: 4, height: 3, keys: "R".repeat(12) };

// The keys are listened to from an effect: press M (harmless) until the
// Select tool is active, so all the handlers are there.
async function open(api = new FakeEditorApi(32, 4, 2)) {
  api.clip = piece;
  const view = renderEditor(api);
  await screen.findByText("1500%");
  await waitFor(() => {
    fireEvent.keyDown(window, { key: "m" });
    expect(screen.getByTestId("active-tool").textContent).toContain("Select");
  });
  return { api, view };
}

function select() {
  drag(
    [32, 4],
    [
      [2, 0],
      [5, 2],
    ],
  );
}

const ctrl = (key: string) => fireEvent.keyDown(window, { key, ctrlKey: true });
const calls = (api: FakeEditorApi, name: string) =>
  api.powerCalls.filter((c) => c.name === name);

describe("Copy and paste", () => {
  // Checks that Ctrl+C copies the selected rectangle and says so, and that
  // Ctrl+V pastes it at the top left of the selection, without erasing under
  // the transparent pixels, and selects the pasted piece.
  it("copies the selection and pastes it", async () => {
    const { api } = await open();
    select();
    await selected();
    ctrl("c");
    expect(await screen.findByText(/Copied 4 × 3/)).toBeTruthy();
    expect(calls(api, "copy")[0].args[2]).toEqual({
      x: 2,
      y: 0,
      width: 4,
      height: 3,
    });

    ctrl("v");
    await waitFor(() => expect(calls(api, "paste")).toHaveLength(1));
    const [sprite, frame, clip, at, skip] = calls(api, "paste")[0].args;
    expect([sprite, frame, clip, at, skip]).toEqual([
      { folder: ["Characters"], name: "goblin" },
      1,
      piece,
      { x: 2, y: 0 },
      true,
    ]);
    await selected();
  });

  // Checks the "Paste in another frame" scenario: copy in frame 1, go to
  // frame 2 and paste there.
  it("pastes in another frame", async () => {
    const { api } = await open();
    select();
    await selected();
    ctrl("c");
    await screen.findByText(/Copied/);
    await waitFor(() => {
      fireEvent.keyDown(window, { key: "." });
      expect(
        screen
          .getByRole("button", { name: "Frame 2" })
          .getAttribute("aria-current"),
      ).toBe("true");
    });
    ctrl("v");
    await waitFor(() => expect(calls(api, "paste")).toHaveLength(1));
    expect(calls(api, "paste")[0].args[1]).toBe(2);
  });

  // Checks that the copy stays when another sprite is opened, so it can be
  // pasted there.
  it("keeps the copy when another sprite is opened", async () => {
    const { api, view } = await open();
    select();
    await selected();
    ctrl("c");
    await screen.findByText(/Copied/);
    view.show(api, { folder: [], name: "slime" });
    await screen.findByText("1500%");
    await waitFor(() => {
      ctrl("v");
      expect(calls(api, "paste").length).toBeGreaterThan(0);
    });
    expect(calls(api, "paste")[0].args[0]).toEqual({
      folder: [],
      name: "slime",
    });
  });

  // Checks the "Different palette" scenario: a clear message and the sprite
  // is not changed (the core refuses; the screen only shows the message).
  it("shows a clear message when the palettes differ", async () => {
    const { api } = await open();
    select();
    await selected();
    ctrl("c");
    await screen.findByText(/Copied/);
    api.powerFailure = { code: "palette_mismatch" };
    ctrl("v");
    expect(await screen.findByText(/another palette/)).toBeTruthy();
  });

  // Checks that pasting with nothing copied only says so.
  it("says there is nothing to paste before a copy", async () => {
    const { api } = await open();
    ctrl("v");
    expect(await screen.findByText(/Nothing copied yet/)).toBeTruthy();
    expect(calls(api, "paste")).toHaveLength(0);
  });

  // Checks the bar buttons: Paste is off until something is copied.
  it("enables the Paste button after a copy", async () => {
    await open();
    select();
    await selected();
    const paste = screen.getByRole("button", { name: "Paste" });
    expect((paste as HTMLButtonElement).disabled).toBe(true);
    fireEvent.click(screen.getByRole("button", { name: "Copy" }));
    await waitFor(() =>
      expect((paste as HTMLButtonElement).disabled).toBe(false),
    );
  });
});
