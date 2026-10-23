// @vitest-environment jsdom
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import App from "./App";

afterEach(cleanup);

describe("App", () => {
  // Checks that the app shows its name and the current version from package.json.
  it("shows the app name and the current version", async () => {
    render(<App />);

    expect(await screen.findByText("Sprixel")).toBeTruthy();
    expect(screen.getByText(`v${__APP_VERSION__}`)).toBeTruthy();
  });

  // Checks the spec scenario: with no project the Welcome screen is shown and
  // the Editor, Export and Agents tabs are disabled.
  it("shows the Welcome screen and disabled tabs when no project is open", async () => {
    render(<App />);

    expect(await screen.findByRole("heading", { level: 1 })).toBeTruthy();
    for (const name of [/Editor/, /Export|Exportar/, /Agent/]) {
      const tab = screen.getByRole("button", { name });
      expect((tab as HTMLButtonElement).disabled).toBe(true);
    }
  });

  // Checks the spec scenario: pressing Settings in the bottom bar opens the
  // Settings screen, shows the button as active and leaves no tab selected.
  it("opens Settings from the bottom bar", async () => {
    render(<App />);
    const button = await screen.findByRole("button", {
      name: /Ajustes|Settings/,
    });
    expect(button.getAttribute("aria-pressed")).toBe("false");

    fireEvent.click(button);

    expect(button.getAttribute("aria-pressed")).toBe("true");
    expect(screen.getAllByRole("radiogroup").length).toBeGreaterThan(0);
    const tabs = document.querySelectorAll("header nav button");
    expect(tabs).toHaveLength(4);
    for (const tab of tabs) expect(tab.getAttribute("aria-current")).toBeNull();
  });
});
