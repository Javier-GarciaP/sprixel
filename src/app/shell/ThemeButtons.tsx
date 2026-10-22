import { useT } from "@/shared/lib/i18n";
import { usePreferences } from "@/shared/lib/preferences";
import { useResolvedTheme } from "@/shared/lib/preferences/useResolvedTheme";

// Quick sun and moon buttons in the bottom bar. The shown theme is marked.
export function ThemeButtons() {
  const t = useT();
  const { update } = usePreferences();
  const shown = useResolvedTheme();

  const button = (theme: "light" | "dark", glyph: string, label: string) => (
    <button
      type="button"
      aria-label={label}
      aria-pressed={shown === theme}
      onClick={() => update({ theme })}
      className={`rounded px-1.5 text-xs ${shown === theme ? "bg-amber text-ink" : "text-text-muted"}`}
    >
      {glyph}
    </button>
  );

  return (
    <div className="flex items-center gap-1 rounded-md bg-surface-2 p-0.5">
      {button("light", "☀", t("theme.setLight"))}
      {button("dark", "☾", t("theme.setDark"))}
    </div>
  );
}
