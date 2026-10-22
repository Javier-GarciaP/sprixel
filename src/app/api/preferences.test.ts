import { describe, expect, it } from "vitest";
import { defaultPreferences } from "@/shared/lib/preferences";
import { mergePreferences } from "./preferences";

describe("mergePreferences", () => {
  // Checks the first start: nothing saved, so the defaults are used
  // (the system language, the System theme, the grid on).
  it("uses the defaults when nothing was saved", () => {
    const defaults = defaultPreferences("es-ES");
    expect(mergePreferences(defaults, {})).toEqual({
      language: "es",
      theme: "system",
      pixelGrid: true,
    });
  });

  // Checks that the saved choices win over the defaults, including a saved
  // `false`, and that null (the Rust "not chosen") means "use the default".
  it("applies what the user saved", () => {
    const defaults = defaultPreferences("es-ES");
    expect(
      mergePreferences(defaults, {
        language: "en",
        theme: "dark",
        pixelGrid: false,
      }),
    ).toEqual({ language: "en", theme: "dark", pixelGrid: false });
    expect(
      mergePreferences(defaults, { language: null, pixelGrid: null }).language,
    ).toBe("es");
  });
});
