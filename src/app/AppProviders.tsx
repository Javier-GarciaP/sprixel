import type { ReactNode } from "react";
import { I18nProvider } from "@/shared/lib/i18n";
import {
  PreferencesProvider,
  usePreferences,
  type Preferences,
} from "@/shared/lib/preferences";

// The language of the whole app comes from the preferences.
function Language({ children }: { children: ReactNode }) {
  const { preferences } = usePreferences();
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
      <Language>{children}</Language>
    </PreferencesProvider>
  );
}
