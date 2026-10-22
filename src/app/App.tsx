import { useState } from "react";
import { useT } from "@/shared/lib/i18n";
import { defaultPreferences } from "@/shared/lib/preferences";
import { AppProviders } from "./AppProviders";
import { Shell } from "./shell/Shell";
import type { Tab } from "./shell/TopBar";
import { useWindowTitle } from "./shell/windowTitle";

function Main() {
  const t = useT();
  const [tab, setTab] = useState<Tab>("project");
  const [settingsOpen, setSettingsOpen] = useState(false);
  useWindowTitle(null);

  return (
    <Shell
      topBar={{
        project: (
          <span className="text-sm text-text-muted">{t("project.none")}</span>
        ),
        status: null,
        hasProject: false,
        activeTab: settingsOpen ? null : tab,
        onTab: (next) => {
          setSettingsOpen(false);
          setTab(next);
        },
      }}
      settingsOpen={settingsOpen}
      onSettings={() => setSettingsOpen(true)}
    >
      <p className="p-6 text-text-muted">{t("body.soon")}</p>
    </Shell>
  );
}

export default function App() {
  return (
    <AppProviders initial={defaultPreferences(navigator.language)}>
      <Main />
    </AppProviders>
  );
}
