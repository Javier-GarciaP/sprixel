import { describe, expect, it } from "vitest";
import en from "./en.json";
import es from "./es.json";
import { detectLanguage, translate } from "./translate";

// Every source file (not tests) of the app, read as text.
const sources = import.meta.glob(
  ["/src/**/*.{ts,tsx}", "!/src/**/*.test.{ts,tsx}"],
  { query: "?raw", import: "default", eager: true },
) as Record<string, string>;

describe("translate", () => {
  const dicts = {
    en: { hello: "Hello", only: "Only English", name: "Hi {name}" },
    es: { hello: "Hola", name: "Hola {name}" },
  };

  // Checks that the chosen language text is used when it exists.
  it("uses the chosen language", () => {
    expect(translate(dicts, "es", "hello")).toBe("Hola");
  });

  // Checks the spec rule: a key missing in Spanish shows the English text,
  // never an empty string or the raw key.
  it("falls back to English when the key is missing", () => {
    expect(translate(dicts, "es", "only")).toBe("Only English");
  });

  // Checks that {placeholders} are replaced with the given values.
  it("fills placeholders", () => {
    expect(translate(dicts, "es", "name", { name: "Ana" })).toBe("Hola Ana");
  });
});

describe("detectLanguage", () => {
  // Checks the first start rule: Spanish or English follow the system,
  // anything else becomes English.
  it("uses the system language when it is es or en, otherwise English", () => {
    expect(detectLanguage("es-ES")).toBe("es");
    expect(detectLanguage("es")).toBe("es");
    expect(detectLanguage("en-US")).toBe("en");
    expect(detectLanguage("fr-FR")).toBe("en");
    expect(detectLanguage(undefined)).toBe("en");
  });
});

describe("translation files", () => {
  // Checks that Spanish has no key that English lacks (English is the source).
  it("has no Spanish key that is missing in English", () => {
    const extra = Object.keys(es).filter((key) => !(key in en));
    expect(extra).toEqual([]);
  });

  // Warns (does not fail) about keys that still have no Spanish text.
  it("warns about keys missing in Spanish", () => {
    const missing = Object.keys(en).filter((key) => !(key in es));
    if (missing.length > 0) {
      console.warn(`Missing in Spanish: ${missing.join(", ")}`);
    }
    expect(missing).toBeDefined();
  });

  // Checks that every key used in the code as t("key") or labelKey: "key"
  // exists in English. A missing English key fails the checks.
  it("has every key used in the code in English", () => {
    const used = new Set<string>();
    for (const code of Object.values(sources)) {
      for (const m of code.matchAll(/\bt\(\s*["']([\w.]+)["']/g))
        used.add(m[1]);
      for (const m of code.matchAll(/labelKey:\s*["']([\w.]+)["']/g))
        used.add(m[1]);
    }
    const missing = [...used].filter((key) => !(key in en));
    expect(missing).toEqual([]);
  });

  // Checks that the missing-key check really fails when a key is absent.
  it("the scan would catch a key that is not in English", () => {
    const code = 't("no.such.key")';
    const key = /\bt\(\s*["']([\w.]+)["']/.exec(code)?.[1] ?? "";
    expect(key in en).toBe(false);
  });
});
