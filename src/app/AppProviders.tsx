import { useEffect, type ReactNode } from "react";
import { I18nProvider } from "@/shared/lib/i18n";
import {
  PreferencesProvider,
  usePreferences,
  type Preferences,
} from "@/shared/lib/preferences";
import { useResolvedTheme } from "@/shared/lib/preferences/useResolvedTheme";

// Language and theme of the whole app come from the preferences.
function Applied({ children }: { children: ReactNode }) {
  const { preferences } = usePreferences();
  const theme = useResolvedTheme();

  useEffect(() => {
    document.documentElement.dataset.theme = theme;
  }, [theme]);

  return (
    <I18nProvider language={preferences.language}>{children}</I18nProvider>
  );
}

export function AppProviders({
  initial,
  onChange,
  children,
}: {
  initial: Preferences;
  onChange?: (preferences: Preferences) => void;
  children: ReactNode;
}) {
  return (
    <PreferencesProvider initial={initial} onChange={onChange}>
      <Applied>{children}</Applied>
    </PreferencesProvider>
  );
}
