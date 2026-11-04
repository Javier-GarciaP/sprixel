export type Language = "en" | "es";

export const languages: Language[] = ["en", "es"];

type Dictionary = Record<string, string>;
type Dictionaries = Record<Language, Dictionary>;

// English is the base language. A key missing in the chosen language shows
// the English text; a key missing in English fails the automated checks.
export function translate(
  dicts: Dictionaries,
  language: Language,
  key: string,
  params?: Record<string, string | number>,
): string {
  const text = dicts[language][key] ?? dicts.en[key] ?? key;
  if (!params) return text;
  return text.replace(/\{(\w+)\}/g, (whole, name: string) =>
    name in params ? String(params[name]) : whole,
  );
}

// First start: follow the system language when it is Spanish or English.
export function detectLanguage(systemLanguage: string | undefined): Language {
  return systemLanguage?.toLowerCase().startsWith("es") ? "es" : "en";
}
