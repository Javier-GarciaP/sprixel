// @vitest-environment jsdom
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { AppProviders } from "../AppProviders";
import { defaultPreferences } from "@/shared/lib/preferences";
import { SaveStatus, type SaveState } from "./SaveStatus";
import { windowTitle } from "./windowTitle";

afterEach(cleanup);

function show(state: SaveState, onRetry = vi.fn()) {
  render(
    <AppProviders initial={defaultPreferences("en")}>
      <SaveStatus state={state} onRetry={onRetry} />
    </AppProviders>,
  );
  return onRetry;
}

describe("SaveStatus", () => {
  // Checks the Saved state.
  it("shows Saved", () => {
    show({ status: "saved" });
    expect(screen.getByText("Saved")).toBeTruthy();
  });

  // Checks the Saving state.
  it("shows Saving...", () => {
    show({ status: "saving" });
    expect(screen.getByText("Saving...")).toBeTruthy();
  });

  // Checks the spec scenario: on error the top bar says "Error saving" and a
  // message gives the reason and a Retry action that calls onRetry.
  it("shows Error saving, the reason and Retry", () => {
    const onRetry = show({ status: "error", reason: "disk full" });
    expect(screen.getByText("Error saving")).toBeTruthy();
    expect(screen.getByRole("status").textContent).toContain("disk full");
    const retries = screen.getAllByRole("button", { name: "Retry" });
    expect(retries).toHaveLength(2);
    fireEvent.click(retries[0]);
    expect(onRetry).toHaveBeenCalledTimes(1);
  });
});

describe("windowTitle", () => {
  // Checks the spec scenario: the title is "<project> - Sprixel".
  it("joins the project name and the app name", () => {
    expect(windowTitle("Dungeon Kit")).toBe("Dungeon Kit - Sprixel");
  });

  // Checks that with no project the title is just the app name.
  it("is only the app name without a project", () => {
    expect(windowTitle(null)).toBe("Sprixel");
  });
});
