import { useEffect, useMemo, useState } from "react";
import { SettingsScreen } from "@/features/settings";
import { useT } from "@/shared/lib/i18n";
import { defaultPreferences } from "@/shared/lib/preferences";
import {
  folderExists,
  loadPreferences,
  mergePreferences,
  removeRecentProject,
  saveSettings,
  type RecentProject,
  type StoredPreferences,
} from "./api/preferences";
import { detectPlatform } from "./commands/shortcut";
import { shortcutGroups } from "./commands/shortcutGroups";
import { useShortcuts } from "./commands/useShortcuts";
import { AppProviders } from "./AppProviders";
import { Shell } from "./shell/Shell";
import type { Tab } from "./shell/TopBar";
import { useWindowTitle } from "./shell/windowTitle";
import { WelcomeScreen } from "./welcome/WelcomeScreen";

function Main({ initialRecents }: { initialRecents: RecentProject[] }) {
  const t = useT();
  const platform = detectPlatform();
  const [tab, setTab] = useState<Tab>("project");
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [recents, setRecents] = useState(initialRecents);
  useWindowTitle(null);
  // What each command does; New and Open arrive with the storage layer.
  const handlers = useMemo<Record<string, () => void>>(() => ({}), []);
  useShortcuts(handlers, platform);

  const groups = shortcutGroups(platform).map((group) => ({
    title: t(group.titleKey),
    rows: group.rows.map((row) => ({
      id: row.id,
      label: t(row.labelKey),
      keys: row.keys,
    })),
  }));

  let body;
  if (settingsOpen) {
    body = <SettingsScreen shortcutGroups={groups} />;
  } else if (tab === "project") {
    body = (
      <WelcomeScreen
        recents={recents}
        // Creating and opening a project arrive with the storage layer (task 5.2).
        onNew={() => {}}
        onOpen={() => {}}
        onOpenRecent={() => {}}
        onRemoveRecent={(path) => removeRecentProject(path).then(setRecents)}
        folderExists={folderExists}
      />
    );
  } else {
    body = <p className="p-6 text-text-muted">{t("body.soon")}</p>;
  }

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
      {body}
    </Shell>
  );
}

export default function App() {
  const [stored, setStored] = useState<StoredPreferences | null>(null);

  // The saved preferences are read once at start.
  useEffect(() => {
    loadPreferences().then(setStored);
  }, []);

  if (!stored) return null;

  return (
    <AppProviders
      initial={mergePreferences(
        defaultPreferences(navigator.language),
        stored.settings,
      )}
      onChange={saveSettings}
    >
      <Main initialRecents={stored.recentProjects} />
    </AppProviders>
  );
}
