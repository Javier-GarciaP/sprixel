import type { ReactNode } from "react";
import logo from "@/assets/sprixel-icon.png";
import { Chip } from "@/shared/components";
import { useT, type TranslationKey } from "@/shared/lib/i18n";

export type Tab = "project" | "editor" | "export" | "agents";

const tabs: { id: Tab; labelKey: TranslationKey }[] = [
  { id: "project", labelKey: "tab.project" },
  { id: "editor", labelKey: "tab.editor" },
  { id: "export", labelKey: "tab.export" },
  { id: "agents", labelKey: "tab.agents" },
];

type Props = {
  // The project name area (and its menu) and the save status.
  project: ReactNode;
  status: ReactNode;
  // The tab that is shown, or null when another screen (Settings) is open.
  activeTab: Tab | null;
  onTab: (tab: Tab) => void;
};

export function TopBar({ project, status, activeTab, onTab }: Props) {
  const t = useT();
  return (
    <header className="relative flex h-12 shrink-0 items-center justify-between border-b border-border bg-bg px-4">
      <div className="flex items-center gap-3">
        <img
          src={logo}
          alt=""
          className="h-7 w-7 [image-rendering:pixelated]"
        />
        <span className="font-title text-base font-bold">{t("app.name")}</span>
        <span className="h-5 w-px bg-border" aria-hidden />
        {project}
        {status}
      </div>
      <nav className="absolute left-1/2 flex h-full -translate-x-1/2 gap-6">
        {tabs.map(({ id, labelKey }) => (
          <button
            key={id}
            type="button"
            aria-current={activeTab === id ? "page" : undefined}
            onClick={() => onTab(id)}
            className={`h-full border-b-2 px-1 text-sm ${activeTab === id ? "border-amber font-semibold text-text" : "border-transparent text-text-muted"}`}
          >
            {t(labelKey)}
          </button>
        ))}
      </nav>
      <Chip>{t("agents.none")}</Chip>
    </header>
  );
}
