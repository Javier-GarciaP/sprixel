// Helpers for the tests of this feature: render a screen with the language
// and theme providers, and a fake core.
import { render } from "@testing-library/react";
import type { ReactNode } from "react";
import { AppProviders } from "@/app/AppProviders";
import { defaultPreferences } from "@/shared/lib/preferences";

export function renderWithApp(ui: ReactNode, language: "en" | "es" = "en") {
  return render(
    <AppProviders initial={{ ...defaultPreferences("en"), language }}>
      {ui}
    </AppProviders>,
  );
}
