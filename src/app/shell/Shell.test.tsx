// @vitest-environment jsdom
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { AppProviders } from "../AppProviders";
import { defaultPreferences } from "@/shared/lib/preferences";
import { Shell } from "./Shell";

afterEach(cleanup);

function renderShell(overrides: Partial<Parameters<typeof Shell>[0]> = {}) {
  const props = {
    topBar: {
      project: <span>Dungeon Kit</span>,
      status: <span>Saved</span>,
      activeTab: "project" as const,
      onTab: vi.fn(),
    },
    settingsOpen: false,
    onSettings: vi.fn(),
    ...overrides,
  };
  render(
    <AppProviders initial={{ ...defaultPreferences("en"), theme: "light" }}>
      <Shell {...props}>
        <p>body content</p>
      </Shell>
    </AppProviders>,
  );
  return props;
}

describe("Shell layout", () => {
  // Checks the spec scenario: the top bar has the project name, the status and
  // the four tabs, the body shows the feature, and the bottom bar has Settings,
  // language and theme.
  it("shows the top bar, the body and the bottom bar", () => {
    renderShell();
    for (const tab of ["Project", "Editor", "Export", "Agents"]) {
      expect(screen.getByRole("button", { name: tab })).toBeTruthy();
    }
    expect(screen.getByText("Dungeon Kit")).toBeTruthy();
    expect(screen.getByText("Saved")).toBeTruthy();
    expect(screen.getByText("body content")).toBeTruthy();
    expect(screen.getByRole("button", { name: /Settings/ })).toBeTruthy();
    expect(
      screen.getByRole("button", { name: "Change language" }),
    ).toBeTruthy();
    expect(screen.getByRole("button", { name: "Dark theme" })).toBeTruthy();
  });

  // Checks that the active tab is marked and a click reports the tab.
  it("marks the active tab and reports clicks", () => {
    const props = renderShell();
    expect(
      screen
        .getByRole("button", { name: "Project" })
        .getAttribute("aria-current"),
    ).toBe("page");
    fireEvent.click(screen.getByRole("button", { name: "Export" }));
    expect(props.topBar.onTab).toHaveBeenCalledWith("export");
  });

  // Checks the spec scenario: the quick theme button changes the theme of the
  // whole app at once (the data-theme attribute that the tokens use).
  it("switches the theme with the quick button", () => {
    renderShell();
    expect(document.documentElement.dataset.theme).toBe("light");
    fireEvent.click(screen.getByRole("button", { name: "Dark theme" }));
    expect(document.documentElement.dataset.theme).toBe("dark");
  });

  // Checks that Settings shows as pressed when it is open.
  it("shows Settings as active when open", () => {
    renderShell({ settingsOpen: true });
    expect(
      screen
        .getByRole("button", { name: /Settings/ })
        .getAttribute("aria-pressed"),
    ).toBe("true");
  });
});
