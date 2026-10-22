// @vitest-environment jsdom
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { Button, Chip, Dialog, Field, Menu, Switch, Toast } from ".";

afterEach(cleanup);

describe("base components", () => {
  // Checks that a button calls its handler and that a disabled one ignores clicks.
  it("Button clicks, and a disabled Button does not", () => {
    const onClick = vi.fn();
    const { rerender } = render(<Button onClick={onClick}>Go</Button>);
    fireEvent.click(screen.getByText("Go"));
    expect(onClick).toHaveBeenCalledTimes(1);

    rerender(
      <Button onClick={onClick} disabled>
        Go
      </Button>,
    );
    fireEvent.click(screen.getByText("Go"));
    expect(onClick).toHaveBeenCalledTimes(1);
  });

  // Checks that a chip shows its text.
  it("Chip shows its content", () => {
    render(<Chip tone="success">Claude Code</Chip>);
    expect(screen.getByText("Claude Code")).toBeTruthy();
  });

  // Checks that the label is tied to the input and that an error marks the
  // input invalid and shows the message.
  it("Field links its label and shows an error", () => {
    render(<Field label="Name" error="Already exists" />);
    const input = screen.getByLabelText("Name");
    expect(input.getAttribute("aria-invalid")).toBe("true");
    expect(screen.getByText("Already exists")).toBeTruthy();
  });

  // Checks that the switch reports the opposite value when it is clicked.
  it("Switch toggles", () => {
    const onChange = vi.fn();
    render(<Switch checked={true} onChange={onChange} label="Grid" />);
    fireEvent.click(screen.getByRole("switch", { name: "Grid" }));
    expect(onChange).toHaveBeenCalledWith(false);
  });

  // Checks that a dialog closes with its close button and with Escape.
  it("Dialog closes with the button and with Escape", () => {
    const onClose = vi.fn();
    render(
      <Dialog title="New" closeLabel="Close" onClose={onClose}>
        body
      </Dialog>,
    );
    expect(screen.getByRole("dialog", { name: "New" })).toBeTruthy();
    fireEvent.click(screen.getByLabelText("Close"));
    fireEvent.keyDown(window, { key: "Escape" });
    expect(onClose).toHaveBeenCalledTimes(2);
  });

  // Checks that choosing a menu item closes the menu and runs the action,
  // and that a disabled item does nothing.
  it("Menu runs an item and closes", () => {
    const onClose = vi.fn();
    const open = vi.fn();
    const off = vi.fn();
    render(
      <Menu
        onClose={onClose}
        items={[
          { id: "open", label: "Open", hint: "Ctrl+O", onSelect: open },
          { id: "off", label: "Off", disabled: true, onSelect: off },
        ]}
      />,
    );
    fireEvent.click(screen.getByRole("menuitem", { name: /Open/ }));
    fireEvent.click(screen.getByRole("menuitem", { name: /Off/ }));
    expect(open).toHaveBeenCalledTimes(1);
    expect(off).not.toHaveBeenCalled();
    expect(onClose).toHaveBeenCalledTimes(1);
  });

  // Checks that a click outside the menu closes it.
  it("Menu closes on an outside click", () => {
    const onClose = vi.fn();
    render(<Menu onClose={onClose} items={[]} />);
    fireEvent.mouseDown(document.body);
    expect(onClose).toHaveBeenCalledTimes(1);
  });

  // Checks that a toast shows its message and runs its action.
  it("Toast shows a message and an action", () => {
    const onAction = vi.fn();
    render(<Toast message="Saved" actionLabel="Undo" onAction={onAction} />);
    expect(screen.getByRole("status").textContent).toContain("Saved");
    fireEvent.click(screen.getByText("Undo"));
    expect(onAction).toHaveBeenCalledTimes(1);
  });
});
