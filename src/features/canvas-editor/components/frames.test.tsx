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

const current = () =>
  screen
    .getAllByRole("button", { name: /^Frame \d$/ })
    .find((b) => b.getAttribute("aria-current") === "true")
    ?.getAttribute("aria-label");

describe("Changing frame", () => {
  // Checks that a click on a frame in the column makes it the current frame.
  it("changes frame with a click", async () => {
    renderEditor(new FakeEditorApi(8, 8, 3));
    fireEvent.click(await screen.findByRole("button", { name: "Frame 3" }));
    await waitFor(() => expect(current()).toBe("Frame 3"));
  });

  // Checks the keys `,` and `.`: they go to the previous and the next frame
  // and stop at the first and the last one. The key listener is attached in
  // an effect, so the key is sent inside waitFor.
  it("moves with the comma and period keys and stops at the ends", async () => {
    renderEditor(new FakeEditorApi(8, 8, 2));
    await screen.findByRole("button", { name: "Frame 2" });
    await waitFor(() => {
      fireEvent.keyDown(window, { key: "." });
      expect(current()).toBe("Frame 2");
    });
    // Already the last frame: another key press stays there.
    fireEvent.keyDown(window, { key: "." });
    await waitFor(() => expect(current()).toBe("Frame 2"));
    await waitFor(() => {
      fireEvent.keyDown(window, { key: "," });
      expect(current()).toBe("Frame 1");
    });
    // Already the first frame: another key press stays there.
    fireEvent.keyDown(window, { key: "," });
    await waitFor(() => expect(current()).toBe("Frame 1"));
  });
});
