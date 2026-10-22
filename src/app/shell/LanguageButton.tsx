import { useT } from "@/shared/lib/i18n";
import { usePreferences } from "@/shared/lib/preferences";

// Quick switch in the bottom bar: shows the current language, click for the other.
export function LanguageButton() {
  const t = useT();
  const { preferences, update } = usePreferences();

  return (
    <button
      type="button"
      aria-label={t("language.switch")}
      onClick={() =>
        update({ language: preferences.language === "es" ? "en" : "es" })
      }
      className="flex items-center gap-1.5 text-xs text-text-muted"
    >
      <span aria-hidden>🌐</span>
      {t("language.current")}
    </button>
  );
}
