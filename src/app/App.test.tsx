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
});
