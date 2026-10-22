// @vitest-environment jsdom
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { AppProviders } from "../AppProviders";
import { defaultPreferences } from "@/shared/lib/preferences";
import { WelcomeScreen } from "./WelcomeScreen";

afterEach(cleanup);

const recents = [
  { path: "/p/forest-run", name: "Forest Run" },
  { path: "/p/space-jam", name: "Space Jam" },
  { path: "/p/mini", name: "Mini Platformer" },
];

function show(overrides: Partial<Parameters<typeof WelcomeScreen>[0]> = {}) {
  const props = {
    recents,
    onNew: vi.fn(),
    onOpen: vi.fn(),
    onOpenRecent: vi.fn(),
    onRemoveRecent: vi.fn(),
    folderExists: vi.fn().mockResolvedValue(true),
    ...overrides,
  };
  render(
    <AppProviders initial={defaultPreferences("en")}>
      <WelcomeScreen {...props} />
    </AppProviders>,
  );
  return props;
}

describe("WelcomeScreen", () => {
  // Checks that the two main actions are offered and call their handlers.
  it("offers New project and Open project", () => {
    const props = show();
    fireEvent.click(screen.getByRole("button", { name: /New project/ }));
    fireEvent.click(screen.getByRole("button", { name: "Open project" }));
    expect(props.onNew).toHaveBeenCalledTimes(1);
    expect(props.onOpen).toHaveBeenCalledTimes(1);
  });

  // Checks the empty state: a short message instead of the list.
  it("shows a short message when there are no recent projects", () => {
    show({ recents: [] });
    expect(
      screen.getByText("You have not opened any project yet."),
    ).toBeTruthy();
  });

  // Checks the spec scenario: recent projects are listed in the given order
  // (most recent first) and clicking one opens it.
  it("lists the recent projects and opens the one clicked", async () => {
    const props = show();
    const names = screen
      .getAllByRole("listitem")
      .map((li) => li.querySelector("span")?.textContent);
    expect(names).toEqual(["Forest Run", "Space Jam", "Mini Platformer"]);

    fireEvent.click(screen.getByText("Space Jam"));
    await vi.waitFor(() =>
      expect(props.onOpenRecent).toHaveBeenCalledWith("/p/space-jam"),
    );
  });

  // Checks the spec scenario: a recent project whose folder is gone is not
  // opened; a message says the folder was not found and offers to remove it.
  it("shows a message and can remove a recent project that was moved", async () => {
    const props = show({ folderExists: vi.fn().mockResolvedValue(false) });
    fireEvent.click(screen.getByText("Space Jam"));

    const alert = await screen.findByRole("alert");
    expect(alert.textContent).toContain("Folder not found: /p/space-jam");
    expect(props.onOpenRecent).not.toHaveBeenCalled();

    fireEvent.click(
      screen.getByRole("button", { name: "Remove from the list" }),
    );
    expect(props.onRemoveRecent).toHaveBeenCalledWith("/p/space-jam");
    expect(screen.queryByRole("alert")).toBeNull();
  });
});
