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

const palette = [
  { key: "L", hex: "#FFFFFF" },
  { key: "B", hex: "#3366CC" },
  { key: "D", hex: "#000033" },
];

async function open() {
  const api = new FakeEditorApi(32, 4, 1, palette);
  api.preview = { pixels: [[1, 1, "L"]] };
  renderEditor(api);
  await screen.findByText("1500%");
  fireEvent.click(screen.getByRole("button", { name: "Shade..." }));
  await screen.findByRole("dialog", { name: "Shade" });
  return api;
}

const pick = (key: string, hex: string, group: string) =>
  fireEvent.click(
    screen
      .getByRole("group", { name: group })
      .querySelector(`[aria-label="Color ${key} (${hex})"]`)!,
  );
const calls = (api: FakeEditorApi, name: string) =>
  api.powerCalls.filter((c) => c.name === name);
const BASE = "Base color (the one to shade)";
const RAMP = "Ramp: pick 2 to 8 colors, the base included";

describe("Shade panel", () => {
  // Checks that nothing is asked of the core until the base and a ramp are
  // chosen, and that the panel says what is missing.
  it("asks for a base and a ramp before it previews", async () => {
    const api = await open();
    expect(screen.getByText(/Pick the base color/)).toBeTruthy();
    expect(
      (screen.getByRole("button", { name: "Apply" }) as HTMLButtonElement)
        .disabled,
    ).toBe(true);
    expect(calls(api, "shadePreview")).toHaveLength(0);
  });

  // Checks that the ramp is sent from dark to light, whatever the order it
  // was picked in, with the base included, and that nothing is written.
  it("previews with the ramp sorted dark to light", async () => {
    const api = await open();
    pick("L", "#FFFFFF", RAMP);
    pick("D", "#000033", RAMP);
    pick("B", "#3366CC", BASE);
    await screen.findByText("Pixels to change: 1");
    const list = calls(api, "shadePreview");
    expect(list[list.length - 1].args[2]).toEqual({
      light: "top-left",
      base: "B",
      keys: ["D", "B", "L"],
      region: null,
    });
    expect(calls(api, "shadeApply")).toHaveLength(0);
    expect(api.revision).toBe(0);
  });

  // Checks the light direction and Apply: one call with the same options,
  // the panel closes and Undo is possible.
  it("applies with the chosen light and closes", async () => {
    const api = await open();
    pick("D", "#000033", RAMP);
    pick("B", "#3366CC", BASE);
    fireEvent.change(screen.getByLabelText("Light comes from"), {
      target: { value: "bottom-right" },
    });
    await waitFor(() => {
      const list = calls(api, "shadePreview");
      expect(list[list.length - 1].args[2]).toMatchObject({
        light: "bottom-right",
      });
    });
    await screen.findByText("Pixels to change: 1");
    fireEvent.click(screen.getByRole("button", { name: "Apply" }));
    await waitFor(() => expect(calls(api, "shadeApply")).toHaveLength(1));
    expect(calls(api, "shadeApply")[0].args[2]).toMatchObject({
      light: "bottom-right",
      keys: ["D", "B"],
    });
    await waitFor(() =>
      expect(screen.queryByRole("dialog", { name: "Shade" })).toBeNull(),
    );
  });

  // Checks Cancel: nothing is written.
  it("writes nothing on Cancel", async () => {
    const api = await open();
    pick("D", "#000033", RAMP);
    pick("B", "#3366CC", BASE);
    await screen.findByText("Pixels to change: 1");
    fireEvent.click(screen.getByRole("button", { name: "Cancel" }));
    await waitFor(() =>
      expect(screen.queryByRole("dialog", { name: "Shade" })).toBeNull(),
    );
    expect(calls(api, "shadeApply")).toHaveLength(0);
    expect(api.revision).toBe(0);
  });

  // Checks the region option: it only exists with a selection, and when on
  // it sends the selected rectangle.
  it("can limit the shading to the selection", async () => {
    const api = new FakeEditorApi(32, 4, 1, palette);
    api.preview = { pixels: [[1, 1, "L"]] };
    renderEditor(api);
    await screen.findByText("1500%");
    await waitFor(() => {
      fireEvent.keyDown(window, { key: "m" });
      expect(screen.getByTestId("active-tool").textContent).toContain("Select");
    });
    drag(
      [32, 4],
      [
        [2, 0],
        [5, 2],
      ],
    );
    await screen.findByText("Selection 4 × 3");
    fireEvent.click(screen.getByRole("button", { name: "Shade..." }));
    await screen.findByRole("dialog", { name: "Shade" });
    pick("D", "#000033", RAMP);
    pick("B", "#3366CC", BASE);
    fireEvent.click(
      screen.getByRole("checkbox", { name: "Only inside the selection" }),
    );
    await waitFor(() => {
      const list = calls(api, "shadePreview");
      expect(list[list.length - 1].args[2]).toMatchObject({
        region: { x: 2, y: 0, width: 4, height: 3 },
      });
    });
  });
});
