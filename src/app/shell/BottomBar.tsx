import { Icon } from "@/shared/components";
import { useT } from "@/shared/lib/i18n";
import { LanguageButton } from "./LanguageButton";
import { ThemeButtons } from "./ThemeButtons";

type Props = { settingsOpen: boolean; onSettings: () => void };

export function BottomBar({ settingsOpen, onSettings }: Props) {
  const t = useT();
  return (
    <footer className="flex h-7 shrink-0 items-center justify-between border-t border-border bg-bg px-3">
      <div className="flex items-center gap-4">
        <button
          type="button"
          aria-pressed={settingsOpen}
          onClick={onSettings}
          className={`flex items-center gap-1.5 rounded px-1.5 text-xs ${settingsOpen ? "bg-surface-2 font-semibold text-text" : "text-text-muted"}`}
        >
          <Icon name="gear" className="h-3.5 w-3.5" />
          {t("bottom.settings")}
        </button>
        <LanguageButton />
        <ThemeButtons />
      </div>
      <span className="font-mono text-xs text-text-faint">
        v{__APP_VERSION__}
      </span>
    </footer>
  );
}
