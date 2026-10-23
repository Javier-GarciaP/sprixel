import { useState } from "react";
import { Icon, Menu } from "@/shared/components";
import { useT } from "@/shared/lib/i18n";
import type { RecentProject } from "../api/preferences";
import { projectMenuCommands } from "../commands/registry";
import { formatShortcut, type Platform } from "../commands/shortcut";

type Props = {
  projectName: string;
  platform: Platform;
  // What each command does. A command without a handler is shown disabled.
  handlers: Record<string, () => void>;
  // The entries of "Open recent".
  recents: RecentProject[];
  onOpenRecent: (path: string) => void;
};

// The project name in the top bar. Clicking it opens the menu right below it.
export function ProjectMenu({
  projectName,
  platform,
  handlers,
  recents,
  onOpenRecent,
}: Props) {
  const t = useT();
  const [open, setOpen] = useState(false);

  return (
    <div className="relative">
      <button
        type="button"
        aria-haspopup="menu"
        aria-expanded={open}
        onClick={() => setOpen(!open)}
        className={`flex items-center gap-1 rounded-md px-2 py-1 text-sm ${open ? "bg-surface-2 font-semibold text-text" : "text-text-muted"}`}
      >
        {projectName} <Icon name="chevronDown" />
      </button>
      {open && (
        <Menu
          onClose={() => setOpen(false)}
          footer={t("menu.autosave")}
          items={projectMenuCommands.map((command) => ({
            id: command.id,
            label: t(command.labelKey),
            hint:
              command.shortcut && formatShortcut(command.shortcut, platform, t),
            separator: command.separator,
            ...(command.id === "project.openRecent"
              ? {
                  disabled: recents.length === 0,
                  submenu: recents.map((project) => ({
                    id: project.path,
                    label: project.name,
                    hint: project.path,
                    onSelect: () => onOpenRecent(project.path),
                  })),
                }
              : { disabled: !handlers[command.id] }),
            onSelect: () => handlers[command.id]?.(),
          }))}
        />
      )}
    </div>
  );
}
