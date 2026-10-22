import { renderToString } from "react-dom/server";
import { describe, expect, it } from "vitest";
import App from "./App";

describe("App", () => {
  it("shows the app name and the current version", () => {
    const html = renderToString(<App />);

    expect(html).toContain("Sprixel");
    expect(html).toContain(__APP_VERSION__);
  });
});
