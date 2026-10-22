// @vitest-environment jsdom
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { SettingsScreen } from "@/features/settings";
import { defaultPreferences } from "@/shared/lib/preferences";
import { AppProviders } from "./AppProviders";
import { BottomBar } from "./shell/BottomBar";
import { ProjectMenu } from "./shell/ProjectMenu";
import { WelcomeScreen } from "./welcome/WelcomeScreen";

afterEach(cleanup);

const oldGlyphs = /[⚙☀☾🌐⌄✕+]/u;

const inApp = (ui: React.ReactNode) =>
  render(<AppProviders initial={defaultPreferences("en")}>{ui}</AppProviders>);

// Checks that a button shows an SVG icon.
const hasSvg = (button: HTMLElement) =>
  expect(button.querySelector("svg")).not.toBeNull();

describe("icons replace the text glyphs", () => {
  // Checks that the bottom bar buttons (Settings, language, both themes) draw
  // SVG icons and contain none of the old glyph characters.
  it("bottom bar", () => {
    inApp(<BottomBar settingsOpen={false} onSettings={() => {}} />);
    for (const name of [
      /Settings/,
      "Change language",
      "Light theme",
      "Dark theme",
    ]) {
      const button = screen.getByRole("button", { name });
      hasSvg(button);
      expect(button.textContent).not.toMatch(oldGlyphs);
    }
  });

  // Checks that New project has the plus icon and Open project the folder
  // icon, and that "+" is no longer typed in the label.
  it("welcome buttons", () => {
    inApp(
      <WelcomeScreen
        recents={[]}
        onNew={vi.fn()}
        onOpen={vi.fn()}
        onOpenRecent={vi.fn()}
        onRemoveRecent={vi.fn()}
        folderExists={vi.fn()}
      />,
    );
    const create = screen.getByRole("button", { name: "New project" });
    const open = screen.getByRole("button", { name: "Open project" });
    hasSvg(create);
    hasSvg(open);
    expect(create.textContent).toBe("New project");
  });

  // Checks that the project name button uses an SVG chevron, not a glyph.
  it("project menu chevron", () => {
    inApp(
      <ProjectMenu projectName="Dungeon Kit" platform="other" handlers={{}} />,
    );
    const button = screen.getByRole("button", { name: /Dungeon Kit/ });
    hasSvg(button);
    expect(button.textContent).not.toMatch(oldGlyphs);
  });

  // Checks that the Settings sidebar items have an icon (globe and keyboard).
  it("settings sidebar", () => {
    inApp(<SettingsScreen shortcutGroups={[]} />);
    for (const name of ["Appearance and language", "Keyboard shortcuts"]) {
      hasSvg(screen.getByRole("button", { name }));
    }
    fireEvent.click(screen.getByRole("button", { name: "Keyboard shortcuts" }));
  });
});
