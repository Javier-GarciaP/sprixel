import { useEffect, useMemo, useState } from "react";
import { useT } from "@/shared/lib/i18n";
import { defaultPreferences } from "@/shared/lib/preferences";
import { useNativeMenu } from "./commands/nativeMenu";
import { detectPlatform } from "./commands/shortcut";
import { useShortcuts } from "./commands/useShortcuts";
import {
  folderExists,
  loadPreferences,
  removeRecentProject,
  type RecentProject,
} from "./api/preferences";
import { AppProviders } from "./AppProviders";
import { Shell } from "./shell/Shell";
import type { Tab } from "./shell/TopBar";
import { WelcomeScreen } from "./welcome/WelcomeScreen";
import { useWindowTitle } from "./shell/windowTitle";

function Main() {
  const t = useT();
  const [tab, setTab] = useState<Tab>("project");
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [recents, setRecents] = useState<RecentProject[]>([]);
  useWindowTitle(null);

  useEffect(() => {
    loadPreferences().then((stored) => setRecents(stored.recentProjects));
  }, []);

  // What each command does; New and Open arrive with the storage layer.
  const handlers = useMemo<Record<string, () => void>>(() => ({}), []);
  useNativeMenu(handlers);
  useShortcuts(handlers, detectPlatform());

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
      {tab === "project" && !settingsOpen ? (
        <WelcomeScreen
          recents={recents}
          // Creating and opening a project arrive with the storage layer (task 5.2).
          onNew={() => {}}
          onOpen={() => {}}
          onOpenRecent={() => {}}
          onRemoveRecent={(path) => removeRecentProject(path).then(setRecents)}
          folderExists={folderExists}
        />
      ) : (
        <p className="p-6 text-text-muted">{t("body.soon")}</p>
      )}
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
