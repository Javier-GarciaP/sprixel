// @vitest-environment jsdom
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { I18nProvider } from "@/shared/lib/i18n";
import {
  PreferencesProvider,
  defaultPreferences,
  type Preferences,
} from "@/shared/lib/preferences";
import { SettingsScreen } from "./SettingsScreen";

afterEach(cleanup);

function show(onChange = vi.fn(), initial?: Partial<Preferences>) {
  render(
    <PreferencesProvider
      initial={{ ...defaultPreferences("en"), ...initial }}
      onChange={onChange}
    >
      <I18nProvider language="en">
        <SettingsScreen
          shortcutGroups={[
            {
              title: "Tools",
              rows: [{ id: "tool.pencil", label: "Pencil", keys: ["B"] }],
            },
          ]}
        />
      </I18nProvider>
    </PreferencesProvider>,
  );
  return onChange;
}

describe("SettingsScreen", () => {
  // Checks that the screen has the two pages of the spec and starts on
  // "Appearance and language".
  it("has the pages Appearance and language and Keyboard shortcuts", () => {
    show();
    expect(
      screen.getByRole("button", { name: "Appearance and language" }),
    ).toBeTruthy();
    expect(
      screen.getByRole("button", { name: "Keyboard shortcuts" }),
    ).toBeTruthy();
    expect(screen.getByRole("heading", { level: 1 }).textContent).toBe(
      "Appearance and language",
    );
  });

  // Checks that the shortcuts page shows the groups it is given.
  it("shows the shortcuts page", () => {
    show();
    fireEvent.click(screen.getByRole("button", { name: "Keyboard shortcuts" }));
    expect(screen.getByText("Pencil")).toBeTruthy();
  });

  // Checks the spec scenario: choosing Dark saves theme = dark.
  it("changes the theme", () => {
    const onChange = show();
    fireEvent.click(screen.getByRole("radio", { name: /Dark/ }));
    expect(onChange).toHaveBeenLastCalledWith(
      expect.objectContaining({ theme: "dark" }),
    );
  });

  // Checks that the theme cards show which choice is selected (System is the
  // first-start default).
  it("marks the selected theme", () => {
    show();
    expect(
      screen
        .getByRole("radio", { name: /Follow the system/ })
        .getAttribute("aria-checked"),
    ).toBe("true");
    expect(
      screen.getByRole("radio", { name: /Dark/ }).getAttribute("aria-checked"),
    ).toBe("false");
  });

  // Checks the spec scenario: choosing English saves language = en.
  it("changes the language", () => {
    const onChange = show(vi.fn(), { language: "es" });
    fireEvent.click(screen.getByRole("radio", { name: "English" }));
    expect(onChange).toHaveBeenLastCalledWith(
      expect.objectContaining({ language: "en" }),
    );
  });

  // Checks the spec scenario: the pixel grid option is on by default and
  // turning it off saves pixelGrid = false.
  it("turns the pixel grid off", () => {
    const onChange = show();
    const grid = screen.getByRole("switch", {
      name: "Show a pixel grid when zooming in",
    });
    expect(grid.getAttribute("aria-checked")).toBe("true");
    fireEvent.click(grid);
    expect(onChange).toHaveBeenLastCalledWith(
      expect.objectContaining({ pixelGrid: false }),
    );
  });
});
