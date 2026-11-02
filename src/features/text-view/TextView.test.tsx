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
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { I18nProvider } from "@/shared/lib/i18n";
import { TextView } from "./TextView";
import { FakeTextApi, GOBLIN_ROWS, goblinText } from "./testing";

const goblin = { folder: ["Characters"], name: "goblin" };

// jsdom has no canvas: the picture is checked from its `data-rows`.
beforeEach(() => {
  vi.spyOn(HTMLCanvasElement.prototype, "getContext").mockReturnValue(null);
});
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
  onApplied,
}: {
  api: FakeTextApi;
  frame?: number;
  frameCount?: number;
  onFrame?: (frame: number) => void;
  onApplied?: () => void;
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
        onApplied={() => onApplied?.()}
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

const preview = () => screen.getByRole("img", { name: "Preview of the text" });
const status = () => screen.getByRole("status");

// A row of the goblin changed: `rows[index]` becomes `row`.
function typeRow(index: number, row: string) {
  const rows = [...GOBLIN_ROWS];
  rows[index] = row;
  fireEvent.change(box(), { target: { value: goblinText(rows) } });
  return rows;
}

describe("Live validation and preview", () => {
  // Checks the first part of screen 21: a text that is valid shows "Valid"
  // with the size and the number of keys used, and the live preview with the
  // rows of the text, and Apply is not marked as an error.
  it("shows Valid with the size and the keys, and the preview", async () => {
    render(<Host api={new FakeTextApi()} />);
    await waitFor(() =>
      expect(status().textContent).toBe(
        "Valid: 8 × 8 · 4 keys used · 0 errors",
      ),
    );
    expect(status().getAttribute("data-tone")).toBe("ok");
    expect(preview().getAttribute("data-rows")).toBe(GOBLIN_ROWS.join("|"));
    expect(screen.getByText("Live view")).toBeTruthy();
  });

  // Checks the "Type a valid change" scenario: one character becomes another
  // valid key, the preview updates and the status stays Valid.
  it("updates the preview when a valid key is typed", async () => {
    render(<Host api={new FakeTextApi()} />);
    await waitFor(() => expect(status().getAttribute("data-tone")).toBe("ok"));
    const rows = typeRow(0, "....NRRR");
    await waitFor(() =>
      expect(preview().getAttribute("data-rows")).toBe(rows.join("|")),
    );
    await waitFor(() => expect(status().getAttribute("data-tone")).toBe("ok"));
    expect(screen.getByText("Live view")).toBeTruthy();
  });

  // Checks that typing does not call the core for every key: several quick
  // changes make one check of the last text.
  it("waits for a pause before it checks what was typed", async () => {
    const api = new FakeTextApi();
    render(<Host api={api} />);
    await waitFor(() => expect(status().getAttribute("data-tone")).toBe("ok"));
    const before = api.checked.length;
    typeRow(0, "....NRRR");
    typeRow(0, "....NNRR");
    const rows = typeRow(0, "....NNNR");
    await waitFor(() =>
      expect(preview().getAttribute("data-rows")).toBe(rows.join("|")),
    );
    expect(api.checked.length - before).toBe(1);
  });
});

describe("Errors in the text", () => {
  // Checks the "A short row" scenario (screen 22): the line with the problem
  // is marked and says "7 columns", the status shows the error with its line,
  // and the preview keeps the last valid state with its own title.
  it("marks the line, shows the error and keeps the last valid preview", async () => {
    const api = new FakeTextApi();
    render(<Host api={api} />);
    await waitFor(() => expect(status().getAttribute("data-tone")).toBe("ok"));
    typeRow(6, ".NAAAAN");
    // Row 7 of the frame is line 17 of this text (10 lines come before).
    const marked = await waitFor(() => {
      const line = document.querySelector("[data-error-line]");
      expect(line).not.toBeNull();
      return line as HTMLElement;
    });
    expect(marked.getAttribute("data-error-line")).toBe("17");
    expect(marked.textContent).toContain("7 columns, expected 8");
    expect(status().textContent).toBe(
      "line 17 (row 7 of frame 1): 7 columns, expected 8",
    );
    expect(status().getAttribute("data-tone")).toBe("error");
    expect(screen.getByText("Last valid state")).toBeTruthy();
    // The picture is still the goblin before the mistake.
    expect(preview().getAttribute("data-rows")).toBe(GOBLIN_ROWS.join("|"));
  });

  // Checks that when there are several errors, the status shows the first one
  // and how many more there are, and every line is marked.
  it("counts the other errors", async () => {
    const api = new FakeTextApi();
    render(<Host api={api} />);
    await waitFor(() => expect(status().getAttribute("data-tone")).toBe("ok"));
    typeRow(1, "...RRRR");
    const rows = [...GOBLIN_ROWS];
    rows[1] = "...RRRR";
    rows[4] = "..NPXPPN";
    fireEvent.change(box(), { target: { value: goblinText(rows) } });
    await waitFor(() => expect(status().textContent).toContain("(and 1 more)"));
    expect(document.querySelectorAll("[data-error-line]")).toHaveLength(2);
  });

  // Checks that when the text is fixed the marks go, the status is Valid and
  // the picture is the new one again.
  it("goes back to Valid when the text is fixed", async () => {
    const api = new FakeTextApi();
    render(<Host api={api} />);
    await waitFor(() => expect(status().getAttribute("data-tone")).toBe("ok"));
    typeRow(6, ".NAAAAN");
    await waitFor(() =>
      expect(status().getAttribute("data-tone")).toBe("error"),
    );
    const rows = typeRow(6, ".NAAAAR.");
    await waitFor(() => expect(status().getAttribute("data-tone")).toBe("ok"));
    expect(document.querySelector("[data-error-line]")).toBeNull();
    expect(preview().getAttribute("data-rows")).toBe(rows.join("|"));
  });

  // Checks that a failure of the check itself (the core is not there) is
  // shown as an error, not as a Valid text.
  it("shows an error when the check fails", async () => {
    const api = new FakeTextApi();
    api.check = () => Promise.reject({ code: "no_core", message: "no core" });
    render(<Host api={api} />);
    expect((await screen.findByRole("alert")).textContent).toContain("no core");
    expect(screen.queryByText(/^Valid/)).toBeNull();
  });
});

const button = (name: string) =>
  screen.getByRole("button", { name }) as HTMLButtonElement;

describe("Apply, copy and paste", () => {
  // Checks that Copy puts the whole text on the clipboard, exactly as it is
  // (the format the AI tools read), and says so.
  it("copies the whole grid", async () => {
    const api = new FakeTextApi();
    render(<Host api={api} />);
    await waitFor(() => expect(status().getAttribute("data-tone")).toBe("ok"));
    fireEvent.click(button("Copy"));
    await screen.findByText("Grid copied");
    expect(api.clipboard).toBe(goblinText());
  });

  // Checks the "Paste a grid from an AI" scenario: a valid grid from the
  // clipboard replaces the text, is checked, and Apply sends exactly that
  // text to the core and tells the editor to read the sprite again.
  it("pastes a valid grid and applies it", async () => {
    const api = new FakeTextApi();
    const onApplied = vi.fn();
    render(<Host api={api} onApplied={onApplied} />);
    await waitFor(() => expect(status().getAttribute("data-tone")).toBe("ok"));
    const rows = GOBLIN_ROWS.map((row) => row.replace(/R/g, "A"));
    api.clipboard = goblinText(rows);
    fireEvent.click(button("Paste"));
    await waitFor(() => expect(box().value).toBe(goblinText(rows)));
    await waitFor(() =>
      expect(preview().getAttribute("data-rows")).toBe(rows.join("|")),
    );
    await waitFor(() => expect(button("Apply").disabled).toBe(false));
    fireEvent.click(button("Apply"));
    await waitFor(() => expect(onApplied).toHaveBeenCalledTimes(1));
    expect(api.applied).toEqual([goblinText(rows)]);
  });

  // Checks the end of an apply: the text of the draft is dropped and the text
  // is read again from the core, so the view shows what the core has.
  it("reads the text again after Apply", async () => {
    const api = new FakeTextApi();
    render(<Host api={api} />);
    await waitFor(() => expect(status().getAttribute("data-tone")).toBe("ok"));
    typeRow(0, "....NRRR");
    await waitFor(() => expect(button("Apply").disabled).toBe(false));
    const reads = api.loaded.length;
    api.texts[1] = goblinText(GOBLIN_ROWS, 1) + "# saved\n";
    fireEvent.click(button("Apply"));
    await waitFor(() => expect(box().value).toContain("# saved"));
    expect(api.loaded.length).toBeGreaterThan(reads);
  });

  // Checks that Apply is off while there is an error, and while the text
  // waits to be checked, so a grid that was not checked is never sent.
  it("disables Apply when the text has an error", async () => {
    const api = new FakeTextApi();
    render(<Host api={api} />);
    await waitFor(() => expect(button("Apply").disabled).toBe(false));
    typeRow(6, ".NAAAAN");
    expect(button("Apply").disabled).toBe(true);
    await waitFor(() =>
      expect(status().getAttribute("data-tone")).toBe("error"),
    );
    expect(button("Apply").disabled).toBe(true);
    fireEvent.click(button("Apply"));
    expect(api.applied).toEqual([]);
  });

  // Checks that a grid the core refuses (an agent is drawing on the frame)
  // shows why, keeps the text the person wrote and does not tell the editor.
  it("shows why the core did not apply the grid", async () => {
    const api = new FakeTextApi();
    api.failApply = {
      code: "frame_busy",
      message: "an agent is drawing on this frame",
    };
    const onApplied = vi.fn();
    render(<Host api={api} onApplied={onApplied} />);
    await waitFor(() => expect(button("Apply").disabled).toBe(false));
    typeRow(0, "....NRRR");
    await waitFor(() => expect(button("Apply").disabled).toBe(false));
    fireEvent.click(button("Apply"));
    await screen.findByText(
      "The grid was not applied: an agent is drawing on this frame",
    );
    expect(onApplied).not.toHaveBeenCalled();
    expect(box().value).toContain("....NRRR");
  });

  // Checks that a clipboard that cannot be read shows an error and keeps the
  // text as it was.
  it("keeps the text when the clipboard cannot be read", async () => {
    const api = new FakeTextApi();
    api.clipboardBlocked = true;
    render(<Host api={api} />);
    await waitFor(() => expect(box().value).toBe(goblinText()));
    fireEvent.click(button("Paste"));
    await screen.findByText(/The clipboard could not be read/);
    expect(box().value).toBe(goblinText());
  });
});
