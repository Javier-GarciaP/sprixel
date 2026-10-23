// @vitest-environment jsdom
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { AppProviders } from "../AppProviders";
import { defaultPreferences } from "@/shared/lib/preferences";
import { ProjectMenu } from "./ProjectMenu";

afterEach(cleanup);

const recents = [
  { path: "/p/forest-run", name: "Forest Run" },
  { path: "/p/space-jam", name: "Space Jam" },
];

function show(
  handlers: Record<string, () => void> = {},
  list = recents,
  onOpenRecent = vi.fn(),
) {
  render(
    <AppProviders initial={defaultPreferences("en")}>
      <ProjectMenu
        projectName="Dungeon Kit"
        platform="other"
        handlers={handlers}
        recents={list}
        onOpenRecent={onOpenRecent}
      />
    </AppProviders>,
  );
  return onOpenRecent;
}

describe("ProjectMenu", () => {
  // Checks the spec scenario: clicking the project name opens the menu and
  // the name button looks pressed (aria-expanded).
  it("opens from the project name", () => {
    show();
    const name = screen.getByRole("button", { name: /Dungeon Kit/ });
    expect(screen.queryByRole("menu")).toBeNull();
    fireEvent.click(name);
    expect(screen.getByRole("menu")).toBeTruthy();
    expect(name.getAttribute("aria-expanded")).toBe("true");
  });

  // Checks the spec scenario: the menu has the listed commands with their
  // shortcuts and no Save command.
  it("lists the commands with shortcuts and has no Save", () => {
    show();
    fireEvent.click(screen.getByRole("button", { name: /Dungeon Kit/ }));
    const names = screen.getAllByRole("menuitem").map((i) => i.textContent);
    expect(names).toEqual([
      "New projectCtrl+N",
      "Open...Ctrl+O",
      "Open recent",
      "Rename project...",
      "Save a copy as...",
      "Show in folder",
      "Export...Ctrl+E",
      "Close project",
    ]);
    expect(screen.queryByRole("menuitem", { name: /^Save$/ })).toBeNull();
    expect(screen.getByText("The project saves by itself.")).toBeTruthy();
  });

  // Checks that a command with a handler runs it and closes the menu, and a
  // command without a handler is disabled.
  it("runs a handler and disables commands without one", () => {
    const close = vi.fn();
    show({ "project.close": close });
    fireEvent.click(screen.getByRole("button", { name: /Dungeon Kit/ }));
    expect(
      (screen.getByRole("menuitem", { name: /Export/ }) as HTMLButtonElement)
        .disabled,
    ).toBe(true);
    fireEvent.click(screen.getByRole("menuitem", { name: "Close project" }));
    expect(close).toHaveBeenCalledTimes(1);
    expect(screen.queryByRole("menu")).toBeNull();
  });

  // Checks that "Open recent" lists the recent projects in a submenu and that
  // choosing one opens it by its path and closes the menu.
  it("opens a recent project from the Open recent submenu", () => {
    const onOpenRecent = show();
    fireEvent.click(screen.getByRole("button", { name: /Dungeon Kit/ }));
    fireEvent.click(screen.getByRole("menuitem", { name: /Open recent/ }));

    const items = screen.getAllByRole("menuitem").map((i) => i.textContent);
    expect(items).toContain("Forest Run/p/forest-run");

    fireEvent.click(screen.getByRole("menuitem", { name: /Space Jam/ }));
    expect(onOpenRecent).toHaveBeenCalledWith("/p/space-jam");
    expect(screen.queryByRole("menu")).toBeNull();
  });

  // Checks that "Open recent" is disabled when there are no recent projects.
  it("disables Open recent when the list is empty", () => {
    show({}, []);
    fireEvent.click(screen.getByRole("button", { name: /Dungeon Kit/ }));
    expect(
      (
        screen.getByRole("menuitem", {
          name: /Open recent/,
        }) as HTMLButtonElement
      ).disabled,
    ).toBe(true);
  });
});
