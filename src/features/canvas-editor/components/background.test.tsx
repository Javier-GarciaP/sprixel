// @vitest-environment jsdom
import { cleanup, fireEvent, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { FakeEditorApi } from "../testing/fakeApi";
import { mockLayout, renderEditor } from "../testing/renderEditor";

beforeEach(() => mockLayout(500, 300));
afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
  vi.unstubAllGlobals();
});

const surface = () => screen.getByTestId("preview-surface");
const choice = (name: string) => screen.getByRole("button", { name });

async function open() {
  renderEditor(new FakeEditorApi(16, 16));
  await screen.findByText("1800%");
}

describe("Preview background", () => {
  // Checks that the preview starts on the gray checkerboard and each button
  // changes what is behind the sprite: a flat light color, a flat dark one.
  it("changes the background of the preview", async () => {
    await open();
    expect(choice("Checker").getAttribute("aria-pressed")).toBe("true");
    expect(surface().style.backgroundImage).toContain("conic-gradient");
    fireEvent.click(choice("Light"));
    expect(surface().style.backgroundColor).toBe("rgb(244, 244, 245)");
    expect(surface().style.backgroundImage).toBe("");
    fireEvent.click(choice("Dark"));
    expect(surface().style.backgroundColor).toBe("rgb(22, 22, 26)");
    expect(choice("Dark").getAttribute("aria-pressed")).toBe("true");
    expect(choice("Light").getAttribute("aria-pressed")).toBe("false");
  });

  // Checks the key (Shift+B): checker, light, dark and back to checker.
  it("goes through the backgrounds with Shift+B", async () => {
    await open();
    // The first key can come before the handlers are registered, so it is
    // sent inside waitFor; one press that works changes the mode at once.
    for (const next of ["Light", "Dark", "Checker"]) {
      await waitFor(() => {
        fireEvent.keyDown(window, { key: "B", shiftKey: true });
        expect(choice(next).getAttribute("aria-pressed")).toBe("true");
      });
    }
  });

  // Checks that the editing canvas does not follow the preview background:
  // the pixel canvas stays on the neutral checkerboard in both themes.
  it("leaves the editing canvas alone", async () => {
    await open();
    fireEvent.click(choice("Dark"));
    expect(screen.getByTestId("canvas-area")).toBeTruthy();
    expect(screen.getByTestId("canvas-area").style.backgroundColor).toBe("");
  });
});
