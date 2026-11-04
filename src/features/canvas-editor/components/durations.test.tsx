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

const field = (n: number) =>
  screen.getByLabelText(`Duration of frame ${n} (ms)`) as HTMLInputElement;

async function open() {
  const api = new FakeEditorApi(8, 8, 3);
  api.tags = [{ name: "walk", from: 1, to: 2, fps: 8, mode: "loop" }];
  renderEditor(api);
  await screen.findByLabelText("Duration of frame 3 (ms)");
  return api;
}

describe("Frame durations", () => {
  // Checks that a frame without a time of its own shows the time its
  // animation gives it (8 fps is 125 ms), and a frame outside any animation
  // shows the default 100. The boxes are empty: no frame has its own time.
  it("shows the animation speed when the frame has no duration", async () => {
    await open();
    await waitFor(() => expect(field(1).placeholder).toBe("125"));
    expect(field(2).placeholder).toBe("125");
    expect(field(3).placeholder).toBe("100");
    expect(field(1).value).toBe("");
  });

  // Checks the "Duration" scenario: 200 on frame 2 is sent to the core, frame
  // 2 shows 200 and the other frames still show the animation speed.
  it("saves the duration of one frame", async () => {
    const api = await open();
    fireEvent.change(field(2), { target: { value: "200" } });
    fireEvent.blur(field(2));
    await waitFor(() =>
      expect(api.durationCalls).toEqual([{ frame: 2, ms: 200 }]),
    );
    await waitFor(() => expect(field(2).value).toBe("200"));
    expect(field(1).value).toBe("");
    expect(field(1).placeholder).toBe("125");
  });

  // Checks that Enter saves too, and that a value that did not change is not
  // sent again.
  it("saves with Enter and ignores an unchanged box", async () => {
    const api = await open();
    fireEvent.change(field(3), { target: { value: "300" } });
    fireEvent.keyDown(field(3), { key: "Enter" });
    await waitFor(() => expect(api.durationCalls).toHaveLength(1));
    await waitFor(() => expect(field(3).value).toBe("300"));
    fireEvent.blur(field(3));
    expect(api.durationCalls).toHaveLength(1);
  });

  // Checks that emptying the box gives the default back (100 means "none of
  // its own" in the core), so the animation speed shows again.
  it("goes back to the animation speed when the box is emptied", async () => {
    const api = new FakeEditorApi(8, 8, 3);
    api.tags = [{ name: "walk", from: 1, to: 2, fps: 8, mode: "loop" }];
    api.durations = [250, 100, 100];
    renderEditor(api);
    // The box shows the own time that the core gave (not something typed).
    await waitFor(() => expect(field(1).value).toBe("250"));
    fireEvent.change(field(1), { target: { value: "" } });
    fireEvent.blur(field(1));
    await waitFor(() =>
      expect(api.durationCalls).toEqual([{ frame: 1, ms: 100 }]),
    );
    await waitFor(() => expect(field(1).value).toBe(""));
    expect(field(1).placeholder).toBe("125");
  });

  // Checks that a value outside 10 to 10000 is not sent and the box goes back.
  it("refuses a duration outside 10 to 10000", async () => {
    const api = await open();
    fireEvent.change(field(2), { target: { value: "5" } });
    fireEvent.blur(field(2));
    await waitFor(() => expect(field(2).value).toBe(""));
    expect(api.durationCalls).toHaveLength(0);
  });
});
