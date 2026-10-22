import { createContext, useContext, type ReactNode } from "react";
import en from "./en.json";
import es from "./es.json";
import { translate, type Language } from "./translate";

export { detectLanguage, languages, type Language } from "./translate";

// Only the keys of the English file are valid: a typo fails `npm run build`.
export type TranslationKey = keyof typeof en;
type Params = Record<string, string | number>;

const dicts = { en, es } as Record<Language, Record<string, string>>;

export function t(
  language: Language,
  key: TranslationKey,
  params?: Params,
): string {
  return translate(dicts, language, key, params);
}

const LanguageContext = createContext<Language>("en");

export function I18nProvider({
  language,
  children,
}: {
  language: Language;
  children: ReactNode;
}) {
  return (
    <LanguageContext.Provider value={language}>
      {children}
    </LanguageContext.Provider>
  );
}

// Returns a function bound to the current language: `const t = useT()`.
export function useT() {
  const language = useContext(LanguageContext);
  return (key: TranslationKey, params?: Params) => t(language, key, params);
}
