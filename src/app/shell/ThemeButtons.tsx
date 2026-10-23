import { Icon, type IconName } from "@/shared/components";
import { useT } from "@/shared/lib/i18n";
import { usePreferences } from "@/shared/lib/preferences";
import { useResolvedTheme } from "@/shared/lib/preferences/useResolvedTheme";

// Quick sun and moon buttons in the bottom bar. The shown theme is marked.
export function ThemeButtons() {
  const t = useT();
  const { update } = usePreferences();
  const shown = useResolvedTheme();

  const button = (theme: "light" | "dark", icon: IconName, label: string) => (
    <button
      type="button"
      aria-label={label}
      aria-pressed={shown === theme}
      onClick={() => update({ theme })}
      className={`flex items-center rounded px-1.5 py-0.5 text-xs ${shown === theme ? "bg-amber text-ink" : "text-text-muted"}`}
    >
      <Icon name={icon} className="h-3.5 w-3.5" />
    </button>
  );

  return (
    <div className="flex items-center gap-1 rounded-md bg-surface-2 p-0.5">
      {button("light", "sun", t("theme.setLight"))}
      {button("dark", "moon", t("theme.setDark"))}
    </div>
  );
}
