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

const mode = (name: string) => screen.getByRole("button", { name });

// The keys are listened to from an effect: press M (harmless) until the
// Select tool is active, so all the handlers are there. Then back to the
// Pencil.
async function open(api = new FakeEditorApi(16, 16)) {
  renderEditor(api);
  await screen.findByText("1800%");
  await waitFor(() => {
    fireEvent.keyDown(window, { key: "m" });
    expect(screen.getByTestId("active-tool").textContent).toContain("Select");
  });
  fireEvent.keyDown(window, { key: "b" });
  await waitFor(() =>
    expect(screen.getByTestId("active-tool").textContent).toContain("Pencil"),
  );
  return api;
}

describe("Symmetry mode in the editor", () => {
  // Checks the toggle: it starts off, each button turns its mode on and the
  // others off, and the S key goes through off, horizontal, vertical, both.
  it("has the four modes and the S key goes through them", async () => {
    await open();
    expect(mode("Off").getAttribute("aria-pressed")).toBe("true");
    fireEvent.click(mode("Vertical"));
    expect(mode("Vertical").getAttribute("aria-pressed")).toBe("true");
    expect(mode("Off").getAttribute("aria-pressed")).toBe("false");
    fireEvent.click(mode("Off"));
    for (const next of ["Horizontal", "Vertical", "Both", "Off"]) {
      fireEvent.keyDown(window, { key: "s" });
      await waitFor(() =>
        expect(mode(next).getAttribute("aria-pressed")).toBe("true"),
      );
    }
  });

  // Checks the spec scenario: in horizontal mode a pixel at x 1 of a 16 pixel
  // wide sprite also draws x 14, and both are sent in ONE call (one history
  // entry), so one Undo takes both away.
  it("sends a mirrored stroke as one call", async () => {
    const api = await open();
    fireEvent.click(mode("Horizontal"));
    drag([16, 16], [[1, 5]]);
    await waitFor(() => expect(api.calls).toHaveLength(1));
    expect(api.calls[0].changes.map(({ x, y }) => [x, y]).sort()).toEqual([
      [1, 5],
      [14, 5],
    ]);
    fireEvent.click(screen.getByRole("button", { name: "Undo" }));
    await waitFor(() => expect(api.rows(1)[5]).toBe(".".repeat(16)));
  });

  // Checks that with the mode off only the pixel is drawn.
  it("draws one pixel with symmetry off", async () => {
    const api = await open();
    drag([16, 16], [[1, 5]]);
    await waitFor(() => expect(api.calls).toHaveLength(1));
    expect(api.calls[0].changes).toHaveLength(1);
  });
});
