// @vitest-environment jsdom
import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import App from "./App";

afterEach(cleanup);

describe("App", () => {
  // Checks that the app shows its name and the current version from package.json.
  it("shows the app name and the current version", () => {
    render(<App />);

    expect(screen.getByText("Sprixel")).toBeTruthy();
    expect(screen.getByText(`v${__APP_VERSION__}`)).toBeTruthy();
  });

  // Checks the spec scenario: with no project the Welcome screen is shown and
  // the Editor, Export and Agents tabs are disabled.
  it("shows the Welcome screen and disabled tabs when no project is open", () => {
    render(<App />);

    expect(screen.getByRole("heading", { level: 1 })).toBeTruthy();
    for (const name of [/Editor/, /Export|Exportar/, /Agent/]) {
      const tab = screen.getByRole("button", { name });
      expect((tab as HTMLButtonElement).disabled).toBe(true);
    }
  });
});
