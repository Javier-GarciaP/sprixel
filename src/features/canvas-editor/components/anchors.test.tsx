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

const feet = { name: "feet", x: 1, y: 1, frame: null };

// Opens a 16x16 sprite with the Anchor tool. The keys are listened to from
// an effect, so A is pressed until the tool is active.
async function open(anchors = [feet]) {
  const api = new FakeEditorApi(16, 16, 2);
  api.anchorList = anchors.map((a) => ({ ...a }));
  renderEditor(api);
  await screen.findByText("1800%");
  await waitFor(() => {
    fireEvent.keyDown(window, { key: "a" });
    expect(screen.getByRole("dialog", { name: "Anchors" })).toBeTruthy();
  });
  return api;
}

const calls = (api: FakeEditorApi, name: string) =>
  api.powerCalls.filter((c) => c.name === name);
const nameBox = () => screen.getByLabelText("Name") as HTMLInputElement;

describe("Anchors", () => {
  // Checks that the anchors of the sprite are drawn with their name over the
  // canvas, and listed in the panel (the keyboard way to pick one).
  it("shows the name of each anchor on the canvas and in the panel", async () => {
    await open();
    expect((await screen.findByTestId("anchor-label")).textContent).toBe(
      "feet",
    );
    expect(screen.getByRole("button", { name: "feet" })).toBeTruthy();
  });

  // Checks "create": a click on an empty pixel starts a new anchor at that
  // pixel; with a name, Save sends it for the whole sprite.
  it("creates an anchor where the canvas is clicked", async () => {
    const api = await open();
    drag([16, 16], [[5, 6]]);
    fireEvent.change(await waitFor(nameBox), { target: { value: "hand" } });
    fireEvent.click(screen.getByRole("button", { name: "Save" }));
    await waitFor(() => expect(calls(api, "saveAnchor")).toHaveLength(1));
    expect(calls(api, "saveAnchor")[0].args.slice(1)).toEqual([
      { name: "hand", x: 5, y: 6, frame: null },
      null,
    ]);
    await waitFor(() =>
      expect(screen.getAllByTestId("anchor-label")).toHaveLength(2),
    );
  });

  // Checks the "only this frame" option.
  it("can make an anchor for the current frame only", async () => {
    const api = await open([]);
    drag([16, 16], [[2, 2]]);
    fireEvent.change(await waitFor(nameBox), { target: { value: "hand" } });
    fireEvent.click(
      screen.getByRole("checkbox", { name: "Only on this frame" }),
    );
    fireEvent.click(screen.getByRole("button", { name: "Save" }));
    await waitFor(() => expect(calls(api, "saveAnchor")).toHaveLength(1));
    expect(calls(api, "saveAnchor")[0].args[1]).toMatchObject({ frame: 1 });
  });

  // Checks "drag": pressing on an anchor and releasing on another pixel saves
  // it there, replacing the old one (one call, so one history entry).
  it("moves an anchor by dragging it", async () => {
    const api = await open();
    drag(
      [16, 16],
      [
        [1, 1],
        [3, 1],
        [4, 2],
      ],
    );
    await waitFor(() => expect(calls(api, "saveAnchor")).toHaveLength(1));
    expect(calls(api, "saveAnchor")[0].args.slice(1)).toEqual([
      { name: "feet", x: 4, y: 2, frame: null },
      { name: "feet", frame: null },
    ]);
  });

  // Checks that a click on an anchor (no move) only selects it: nothing is
  // saved, and the form shows its name.
  it("selects an anchor with a click and saves nothing", async () => {
    const api = await open();
    drag([16, 16], [[1, 1]]);
    await waitFor(() => expect(nameBox().value).toBe("feet"));
    expect(calls(api, "saveAnchor")).toHaveLength(0);
  });

  // Checks "rename": the old name is the one that is replaced.
  it("renames an anchor", async () => {
    const api = await open();
    fireEvent.click(screen.getByRole("button", { name: "feet" }));
    fireEvent.change(await waitFor(nameBox), { target: { value: "boots" } });
    fireEvent.click(screen.getByRole("button", { name: "Save" }));
    await waitFor(() => expect(calls(api, "saveAnchor")).toHaveLength(1));
    expect(calls(api, "saveAnchor")[0].args.slice(1)).toEqual([
      { name: "boots", x: 1, y: 1, frame: null },
      { name: "feet", frame: null },
    ]);
    await waitFor(() =>
      expect(screen.getByTestId("anchor-label").textContent).toBe("boots"),
    );
  });

  // Checks "delete".
  it("deletes an anchor", async () => {
    const api = await open();
    fireEvent.click(screen.getByRole("button", { name: "feet" }));
    fireEvent.click(await screen.findByRole("button", { name: "Delete" }));
    await waitFor(() => expect(calls(api, "deleteAnchor")).toHaveLength(1));
    expect(calls(api, "deleteAnchor")[0].args[1]).toEqual({
      name: "feet",
      frame: null,
    });
    await waitFor(() =>
      expect(screen.queryAllByTestId("anchor-label")).toHaveLength(0),
    );
  });

  // Checks the keyboard way: New anchor, then X and Y in the boxes.
  it("makes an anchor with the keyboard", async () => {
    const api = await open([]);
    fireEvent.click(screen.getByRole("button", { name: "New anchor" }));
    fireEvent.change(await waitFor(nameBox), { target: { value: "head" } });
    fireEvent.change(screen.getByLabelText("X"), { target: { value: "7" } });
    fireEvent.change(screen.getByLabelText("Y"), { target: { value: "2" } });
    fireEvent.click(screen.getByRole("button", { name: "Save" }));
    await waitFor(() => expect(calls(api, "saveAnchor")).toHaveLength(1));
    expect(calls(api, "saveAnchor")[0].args[1]).toMatchObject({
      name: "head",
      x: 7,
      y: 2,
    });
  });

  // Checks that a place outside the sprite or an empty name keeps Save off,
  // and that a failure of the core is shown.
  it("keeps Save off for bad values and shows failures", async () => {
    const api = await open([]);
    fireEvent.click(screen.getByRole("button", { name: "New anchor" }));
    const save = () =>
      screen.getByRole("button", { name: "Save" }) as HTMLButtonElement;
    expect(save().disabled).toBe(true);
    fireEvent.change(await waitFor(nameBox), { target: { value: "head" } });
    fireEvent.change(screen.getByLabelText("X"), { target: { value: "99" } });
    expect(save().disabled).toBe(true);
    fireEvent.change(screen.getByLabelText("X"), { target: { value: "3" } });
    expect(save().disabled).toBe(false);
    api.powerFailure = { code: "bad_anchor", message: "outside" };
    fireEvent.click(save());
    expect((await screen.findByRole("alert")).textContent).toContain("outside");
  });
});
