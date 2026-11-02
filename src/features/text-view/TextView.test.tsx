// @vitest-environment jsdom
import { useState } from "react";
import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { I18nProvider } from "@/shared/lib/i18n";
import { TextView } from "./TextView";
import { FakeTextApi, GOBLIN_ROWS, goblinText } from "./testing";

const goblin = { folder: ["Characters"], name: "goblin" };

afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
});

// The view with a frame that the test can change, like the editor does.
function Host({
  api,
  frame = 1,
  frameCount = 2,
  onFrame,
}: {
  api: FakeTextApi;
  frame?: number;
  frameCount?: number;
  onFrame?: (frame: number) => void;
}) {
  const [current, setCurrent] = useState(frame);
  return (
    <I18nProvider language="en">
      <TextView
        api={api}
        sprite={goblin}
        frame={current}
        frameCount={frameCount}
        revision={0}
        onFrame={(next) => {
          setCurrent(next);
          onFrame?.(next);
        }}
      />
    </I18nProvider>
  );
}

const box = () => screen.getByRole("textbox") as HTMLTextAreaElement;

describe("Show the sprite as text", () => {
  // Checks the "Open the view" scenario (screen 21): the text of the 8x8
  // goblin has the palette, the legend and 8 rows of 8 characters, and the
  // view shows the name of the file and the frame.
  it("shows the legend and the rows of the frame", async () => {
    render(<Host api={new FakeTextApi()} />);
    await waitFor(() => expect(box().value).toBe(goblinText()));
    expect(box().value).toContain("legend");
    const rows = box()
      .value.split("\n")
      .filter((line) => /^[.NPAR]{8}$/.test(line));
    expect(rows).toEqual(GOBLIN_ROWS);
    expect(screen.getByText("goblin.txt")).toBeTruthy();
    expect(screen.getByText("Frame 1")).toBeTruthy();
  });

  // Checks that the line numbers run from 1 to the last line of the text.
  it("shows a line number for every line", async () => {
    render(<Host api={new FakeTextApi()} />);
    await waitFor(() => expect(box().value).not.toBe(""));
    const numbers = screen.getByTestId("line-numbers").children;
    // The text ends with a new line, so the last line is empty but it exists.
    expect(numbers).toHaveLength(box().value.split("\n").length);
    expect(numbers[0].textContent).toBe("1");
    expect(numbers[numbers.length - 1].textContent).toBe(
      String(numbers.length),
    );
  });

  // Checks that the frames are chips: one per frame, the current one is
  // pressed, and a click asks for that frame, and its text is read.
  it("has a chip for each frame that changes the frame", async () => {
    const api = new FakeTextApi();
    const onFrame = vi.fn();
    render(<Host api={api} onFrame={onFrame} />);
    const chips = within(await screen.findByRole("group", { name: "Frames" }));
    expect(chips.getByText("1").getAttribute("aria-pressed")).toBe("true");
    fireEvent.click(chips.getByText("2"));
    expect(onFrame).toHaveBeenCalledWith(2);
    await waitFor(() => expect(screen.getByText("Frame 2")).toBeTruthy());
    await waitFor(() => expect(api.loaded).toContain(2));
    await waitFor(() => expect(box().value).toContain("frame 2"));
  });

  // Checks that what the person typed in a frame is kept when they look at
  // another frame and come back, and is not mixed with the other frame.
  it("keeps what was typed in each frame", async () => {
    render(<Host api={new FakeTextApi()} />);
    await waitFor(() => expect(box().value).toContain("frame 1"));
    fireEvent.change(box(), { target: { value: "my draft" } });
    expect(box().value).toBe("my draft");
    const chips = within(screen.getByRole("group", { name: "Frames" }));
    fireEvent.click(chips.getByText("2"));
    await waitFor(() => expect(box().value).toContain("frame 2"));
    fireEvent.click(chips.getByText("1"));
    await waitFor(() => expect(box().value).toBe("my draft"));
  });

  // Checks that a text the core cannot give is shown as an error, not as an
  // empty editor.
  it("shows an error when the text cannot be read", async () => {
    const api = new FakeTextApi();
    api.loadText = () =>
      Promise.reject({ code: "no_project", message: "no project" });
    render(<Host api={api} />);
    expect((await screen.findByRole("alert")).textContent).toContain(
      "no project",
    );
    expect(screen.queryByRole("textbox")).toBeNull();
  });
});
