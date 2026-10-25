import { useCallback, useEffect, useState } from "react";
import { EditorScreen } from "@/features/canvas-editor";
import { ProjectBrowser, type SpriteTarget } from "@/features/project-browser";
import { SettingsScreen } from "@/features/settings";
import { Toast } from "@/shared/components";
import { useT } from "@/shared/lib/i18n";
import { defaultPreferences } from "@/shared/lib/preferences";
import {
  addRecentProject,
  folderExists,
  loadPreferences,
  mergePreferences,
  removeRecentProject,
  saveSettings,
  type RecentProject,
  type StoredPreferences,
} from "./api/preferences";
import {
  asProjectFailure,
  closeProject,
  createProject,
  defaultProjectLocation,
  openProject,
  pickFolder,
  type ProjectInfo,
} from "./api/project";
import { formatShortcut, detectPlatform } from "./commands/shortcut";
import { commands } from "./commands/registry";
import { shortcutGroups } from "./commands/shortcutGroups";
import { useShortcuts } from "./commands/useShortcuts";
import { AppProviders } from "./AppProviders";
import { ProjectMenu } from "./shell/ProjectMenu";
import { SaveStatus } from "./shell/SaveStatus";
import { Shell } from "./shell/Shell";
import type { Tab } from "./shell/TopBar";
import { useSaveStatus } from "./shell/useSaveStatus";
import { useWindowTitle } from "./shell/windowTitle";
import { NewProjectDialog } from "./welcome/NewProjectDialog";
import { WelcomeScreen } from "./welcome/WelcomeScreen";

type OpenedProject = { name: string; path: string };

// The Project tab listens to the shortcuts of the list of commands, with the
// same rules as the rest of the app (see useShortcuts).
const useBrowserShortcuts = (handlers: Record<string, () => void>) =>
  useShortcuts(handlers, detectPlatform());

function Main({ initialRecents }: { initialRecents: RecentProject[] }) {
  const t = useT();
  const platform = detectPlatform();
  const [tab, setTab] = useState<Tab>("project");
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [recents, setRecents] = useState(initialRecents);
  // The core owns the project; this is only what the UI shows about it.
  const [project, setProject] = useState<OpenedProject | null>(null);
  const [newProjectFrom, setNewProjectFrom] = useState<string | null>(null);
  // The sprite open in the Editor tab.
  const [activeSprite, setActiveSprite] = useState<SpriteTarget | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  // What the editor commands do, given by the editor while it is open.
  const [editorHandlers, setEditorHandlers] = useState<
    Record<string, () => void>
  >({});
  const registerEditorCommands = useCallback(
    (next: Record<string, () => void> | null) => setEditorHandlers(next ?? {}),
    [],
  );
  const save = useSaveStatus(project !== null);
  useWindowTitle(project?.name ?? null);

  // A message at the bottom that goes away by itself.
  useEffect(() => {
    if (!notice) return;
    const timer = setTimeout(() => setNotice(null), 6000);
    return () => clearTimeout(timer);
  }, [notice]);

  const afterOpen = (info: ProjectInfo) => {
    setProject({ name: info.name, path: info.path });
    setActiveSprite(null);
    setTab("project");
    setSettingsOpen(false);
    addRecentProject(info.path, info.name).then(setRecents);
  };

  const openAt = async (path: string) => {
    try {
      afterOpen(await openProject(path));
    } catch (error) {
      const failure = asProjectFailure(error);
      setNotice(
        failure.code === "not_a_project"
          ? t("open.notAProject")
          : t("error.io", {
              message: "message" in failure ? failure.message : "",
            }),
      );
    }
  };

  const chooseAndOpen = async () => {
    const path = await pickFolder();
    if (path) await openAt(path);
  };

  // From the menu, a recent project may have been moved.
  const openRecentFromMenu = async (path: string) => {
    if (await folderExists(path)) await openAt(path);
    else setNotice(t("welcome.folderMissing", { path }));
  };

  const showNewProject = async () => {
    setNewProjectFrom(await defaultProjectLocation().catch(() => ""));
  };

  const close = async () => {
    await closeProject();
    setProject(null);
    setActiveSprite(null);
    setTab("project");
  };

  // What each command does. The other project menu commands belong to other
  // specs, so they have no action yet and stay disabled.
  const handlers: Record<string, () => void> = {
    "project.new": showNewProject,
    "project.open": chooseAndOpen,
    ...(project ? { "project.close": close } : {}),
  };
  useShortcuts({ ...handlers, ...editorHandlers }, platform);

  const groups = shortcutGroups(platform, t).map((group) => ({
    title: t(group.titleKey),
    rows: group.rows.map((row) => ({
      id: row.id,
      label: t(row.labelKey),
      keys: row.keys,
      alternative: row.alternative,
    })),
  }));

  let body;
  if (settingsOpen) {
    body = <SettingsScreen shortcutGroups={groups} />;
  } else if (tab === "project" && project) {
    // Keyed by the project: another project starts with a clean view.
    body = (
      <ProjectBrowser
        key={project.path}
        shortcuts={{
          useHandlers: useBrowserShortcuts,
          hint: (id) => {
            const shortcut = commands.find((c) => c.id === id)?.shortcut;
            return shortcut && formatShortcut(shortcut, platform, t);
          },
        }}
        onOpenSprite={(sprite) => {
          setActiveSprite(sprite);
          setTab("editor");
        }}
        // The palette editor belongs to another spec.
        onOpenPalette={() => setNotice(t("body.soon"))}
        onNewPalette={() => setNotice(t("body.soon"))}
      />
    );
  } else if (tab === "project") {
    body = (
      <WelcomeScreen
        recents={recents}
        onNew={showNewProject}
        onOpen={chooseAndOpen}
        onOpenRecent={openAt}
        onRemoveRecent={(path) => removeRecentProject(path).then(setRecents)}
        folderExists={folderExists}
      />
    );
  } else if (tab === "editor") {
    body = (
      <EditorScreen
        key={project?.path}
        sprite={activeSprite}
        onCommands={registerEditorCommands}
        onGoToProject={() => setTab("project")}
      />
    );
  } else {
    body = <p className="p-6 text-text-muted">{t("body.soon")}</p>;
  }

  return (
    <Shell
      topBar={{
        project: project ? (
          <ProjectMenu
            projectName={project.name}
            platform={platform}
            handlers={handlers}
            recents={recents}
            onOpenRecent={openRecentFromMenu}
          />
        ) : (
          <span className="text-sm text-text-muted">{t("project.none")}</span>
        ),
        status: project ? (
          <SaveStatus state={save.state} onRetry={save.retry} />
        ) : null,
        hasProject: project !== null,
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
      {newProjectFrom !== null && (
        <NewProjectDialog
          defaultLocation={newProjectFrom}
          pickFolder={pickFolder}
          onCreate={(parent, name) =>
            createProject(parent, name).then((info) => {
              afterOpen(info);
              return null;
            }, asProjectFailure)
          }
          onClose={() => setNewProjectFrom(null)}
        />
      )}
      {notice && <Toast tone="error" message={notice} />}
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
