// @vitest-environment jsdom
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import { AppProviders } from "../AppProviders";
import { defaultPreferences } from "@/shared/lib/preferences";
import { LanguageButton } from "./LanguageButton";

afterEach(cleanup);

describe("language", () => {
  // Checks the first start rule: a Spanish system starts in Spanish.
  it("starts in the system language", () => {
    expect(defaultPreferences("es-MX").language).toBe("es");
    expect(defaultPreferences("de-DE").language).toBe("en");
  });

  // Checks the spec scenario: the quick button changes the visible text at
  // once, in the same render (no restart, no reload).
  it("switches language with the quick button without a restart", () => {
    render(
      <AppProviders initial={defaultPreferences("es-ES")}>
        <LanguageButton />
      </AppProviders>,
    );
    const button = screen.getByRole("button", { name: "Cambiar idioma" });
    expect(button.textContent).toContain("Español");

    fireEvent.click(button);

    const changed = screen.getByRole("button", { name: "Change language" });
    expect(changed.textContent).toContain("English");
  });

  // Checks that the app tells the caller about each change, so it can be saved.
  it("reports every change to onChange", () => {
    const seen: string[] = [];
    render(
      <AppProviders
        initial={defaultPreferences("en")}
        onChange={(p) => seen.push(p.language)}
      >
        <LanguageButton />
      </AppProviders>,
    );
    fireEvent.click(screen.getByRole("button"));
    expect(seen).toEqual(["es"]);
  });
});
