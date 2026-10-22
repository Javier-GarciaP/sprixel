import { Switch } from "@/shared/components";
import { useT, type Language, type TranslationKey } from "@/shared/lib/i18n";
import { usePreferences, type Theme } from "@/shared/lib/preferences";

const themes: { id: Theme; labelKey: TranslationKey }[] = [
  { id: "light", labelKey: "theme.light" },
  { id: "dark", labelKey: "theme.dark" },
  { id: "system", labelKey: "theme.system" },
];

// A small mock window: a frame, a sidebar, a content block and a yellow bar.
// The fixed data-theme makes it use the real color tokens of that theme.
function MockWindow({ theme }: { theme: "light" | "dark" }) {
  return (
    <div
      data-theme={theme}
      data-preview={theme}
      className="flex h-full w-full gap-1.5 rounded border border-border bg-bg p-1.5"
    >
      <div className="w-1/4 rounded-sm bg-surface" />
      <div className="relative flex-1 rounded-sm bg-surface-2">
        <div className="absolute right-1.5 top-1.5 h-1 w-1/3 rounded-sm bg-amber" />
      </div>
    </div>
  );
}

function ThemePreview({ theme }: { theme: Theme }) {
  if (theme !== "system") return <MockWindow theme={theme} />;
  // System: the left half is light and the right half is dark.
  return (
    <div className="relative h-full w-full">
      <MockWindow theme="light" />
      <div className="absolute inset-0 [clip-path:inset(0_0_0_50%)]">
        <MockWindow theme="dark" />
      </div>
    </div>
  );
}

// Spanish first, like the design.
const languageOrder: Language[] = ["es", "en"];

const cardClass = (selected: boolean) =>
  `rounded-lg border-2 bg-surface p-2 text-left text-sm font-semibold ${selected ? "border-amber" : "border-border"}`;

// Theme, language and canvas options. Every choice applies at once.
export function AppearancePage() {
  const t = useT();
  const { preferences, update } = usePreferences();

  return (
    <div className="flex max-w-[640px] flex-col gap-8">
      <section>
        <h3 className="mb-2 text-sm font-semibold text-text-muted">
          {t("settings.theme")}
        </h3>
        <div
          role="radiogroup"
          aria-label={t("settings.theme")}
          className="grid grid-cols-3 gap-3"
        >
          {themes.map(({ id, labelKey }) => (
            <button
              key={id}
              type="button"
              role="radio"
              aria-checked={preferences.theme === id}
              onClick={() => update({ theme: id })}
              className={cardClass(preferences.theme === id)}
            >
              <div className="mb-2 h-20">
                <ThemePreview theme={id} />
              </div>
              {t(labelKey)}
            </button>
          ))}
        </div>
      </section>

      <section>
        <h3 className="mb-2 text-sm font-semibold text-text-muted">
          {t("settings.language")}
        </h3>
        <div
          role="radiogroup"
          aria-label={t("settings.language")}
          className="grid grid-cols-2 gap-3"
        >
          {languageOrder.map((language) => (
            <button
              key={language}
              type="button"
              role="radio"
              aria-checked={preferences.language === language}
              onClick={() => update({ language })}
              className={cardClass(preferences.language === language)}
            >
              {t(`language.option.${language}`)}
            </button>
          ))}
        </div>
      </section>

      <section>
        <h3 className="mb-2 text-sm font-semibold text-text-muted">
          {t("settings.canvas")}
        </h3>
        <div className="flex items-center justify-between rounded-lg border border-border bg-surface p-4 text-sm">
          <span>{t("settings.pixelGrid")}</span>
          <Switch
            checked={preferences.pixelGrid}
            onChange={(pixelGrid) => update({ pixelGrid })}
            label={t("settings.pixelGrid")}
          />
        </div>
      </section>
    </div>
  );
}
