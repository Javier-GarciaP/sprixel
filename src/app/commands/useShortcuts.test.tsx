// @vitest-environment jsdom
import { cleanup, fireEvent, render } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { useShortcuts } from "./useShortcuts";

afterEach(cleanup);

function Probe({ handlers }: { handlers: Record<string, () => void> }) {
  useShortcuts(handlers, "other");
  return <input aria-label="field" />;
}

describe("useShortcuts", () => {
  // Checks the spec scenario: pressing Ctrl+Z runs the Undo handler.
  it("runs the handler of the pressed shortcut", () => {
    const undo = vi.fn();
    render(<Probe handlers={{ "edit.undo": undo }} />);
    fireEvent.keyDown(window, { key: "z", ctrlKey: true });
    expect(undo).toHaveBeenCalledTimes(1);
  });

  // Checks that a shortcut with no handler does nothing (no error).
  it("ignores shortcuts without a handler", () => {
    render(<Probe handlers={{}} />);
    expect(() => fireEvent.keyDown(window, { key: "b" })).not.toThrow();
  });

  // Checks that typing a letter in a text field does not pick a tool, but a
  // Ctrl shortcut still works there.
  it("does not steal plain letters while typing", () => {
    const pencil = vi.fn();
    const open = vi.fn();
    const { getByLabelText } = render(
      <Probe handlers={{ "tool.pencil": pencil, "project.open": open }} />,
    );
    const field = getByLabelText("field");
    fireEvent.keyDown(field, { key: "b" });
    fireEvent.keyDown(field, { key: "o", ctrlKey: true });
    expect(pencil).not.toHaveBeenCalled();
    expect(open).toHaveBeenCalledTimes(1);
  });
});
