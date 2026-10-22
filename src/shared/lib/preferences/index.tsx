import {
  createContext,
  useCallback,
  useContext,
  useRef,
  useState,
  type ReactNode,
} from "react";
import { detectLanguage, type Language } from "../i18n";

export type Theme = "light" | "dark" | "system";

export type Preferences = {
  language: Language;
  theme: Theme;
  pixelGrid: boolean;
};

// First start: the language follows the system, the grid is on.
export function defaultPreferences(
  systemLanguage: string | undefined,
): Preferences {
  return {
    language: detectLanguage(systemLanguage),
    theme: "system",
    pixelGrid: true,
  };
}

type Value = {
  preferences: Preferences;
  update: (change: Partial<Preferences>) => void;
};

const PreferencesContext = createContext<Value | null>(null);

export function PreferencesProvider({
  initial,
  onChange,
  children,
}: {
  initial: Preferences;
  // Called after every change, so the caller can save it.
  onChange?: (preferences: Preferences) => void;
  children: ReactNode;
}) {
  const [preferences, setPreferences] = useState(initial);
  // The latest value, so two quick changes do not overwrite each other.
  const latest = useRef(initial);

  const update = useCallback(
    (change: Partial<Preferences>) => {
      latest.current = { ...latest.current, ...change };
      setPreferences(latest.current);
      onChange?.(latest.current);
    },
    [onChange],
  );

  return (
    <PreferencesContext.Provider value={{ preferences, update }}>
      {children}
    </PreferencesContext.Provider>
  );
}

export function usePreferences(): Value {
  const value = useContext(PreferencesContext);
  if (!value) throw new Error("usePreferences needs a PreferencesProvider");
  return value;
}
