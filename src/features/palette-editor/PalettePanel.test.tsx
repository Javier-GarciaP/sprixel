// @vitest-environment jsdom
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { I18nProvider } from "@/shared/lib/i18n";
import { PalettePanel } from "./PalettePanel";

afterEach(cleanup);

const colors = [
  { key: "R", hex: "#F04545" },
  { key: "B", hex: "#2F6BFF" },
];

function show(props: Partial<Parameters<typeof PalettePanel>[0]> = {}) {
  const handlers = {
    onMain: vi.fn(),
    onSecondary: vi.fn(),
    onEdit: vi.fn(),
  };
  render(
    <I18nProvider language="en">
      <PalettePanel
        colors={colors}
        main="#000000"
        secondary="#FFFFFF"
        {...handlers}
        {...props}
      />
    </I18nProvider>,
  );
  return handlers;
}

describe("Palette panel", () => {
  // Checks that a click chooses the main color and a right click the
  // secondary color.
  it("chooses the main color on click and the secondary on right click", () => {
    const { onMain, onSecondary } = show();
    fireEvent.click(screen.getByRole("button", { name: "#F04545" }));
    expect(onMain).toHaveBeenCalledWith("#F04545");
    fireEvent.contextMenu(screen.getByRole("button", { name: "#2F6BFF" }));
    expect(onSecondary).toHaveBeenCalledWith("#2F6BFF");
  });

  // Checks that the transparent entry is always there, even with no colors.
  it("shows a transparent entry", () => {
    show({ colors: [] });
    expect(screen.getByRole("img", { name: "Transparent" })).toBeTruthy();
  });

  // Checks the "Show keys" scenario: the letters are hidden at first and
  // every color (and the transparent entry, with ".") shows its letter when
  // the switch is turned on.
  it("shows the key of every color with the Show keys switch", () => {
    show();
    expect(screen.queryByText("R")).toBeNull();
    fireEvent.click(screen.getByRole("switch", { name: "Show keys" }));
    expect(screen.getByText("R")).toBeTruthy();
    expect(screen.getByText("B")).toBeTruthy();
    expect(screen.getByText(".")).toBeTruthy();
    fireEvent.click(screen.getByRole("switch", { name: "Show keys" }));
    expect(screen.queryByText("R")).toBeNull();
  });

  // Checks that the Edit button asks to open the palette editor.
  it("has an Edit button", () => {
    const { onEdit } = show();
    fireEvent.click(screen.getByRole("button", { name: "Edit" }));
    expect(onEdit).toHaveBeenCalledTimes(1);
  });

  // Checks that the main color is marked, found by its value.
  it("marks the main color", () => {
    show({ main: "#2f6bff" });
    expect(
      screen
        .getByRole("button", { name: "#2F6BFF" })
        .getAttribute("aria-pressed"),
    ).toBe("true");
    expect(
      screen
        .getByRole("button", { name: "#F04545" })
        .getAttribute("aria-pressed"),
    ).toBe("false");
  });
});
