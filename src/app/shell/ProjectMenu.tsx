import { useState } from "react";
import { Icon, Menu } from "@/shared/components";
import { useT } from "@/shared/lib/i18n";
import { projectMenuCommands } from "../commands/registry";
import { formatShortcut, type Platform } from "../commands/shortcut";

type Props = {
  projectName: string;
  platform: Platform;
  // What each command does. A command without a handler is shown disabled.
  handlers: Record<string, () => void>;
};

// The project name in the top bar. Clicking it opens the menu right below it.
export function ProjectMenu({ projectName, platform, handlers }: Props) {
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
            disabled: !handlers[command.id],
            onSelect: () => handlers[command.id]?.(),
          }))}
        />
      )}
    </div>
  );
}
