import {
  previewBackgrounds,
  type PreviewBackground,
} from "@/features/frames-animations";
import { Switch } from "@/shared/components";
import { useT, type TranslationKey } from "@/shared/lib/i18n";

const labels: Record<PreviewBackground, TranslationKey> = {
  checker: "preview.bgChecker",
  light: "preview.bgLight",
  dark: "preview.bgDark",
};

type Props = {
  background: PreviewBackground;
  onBackground: (background: PreviewBackground) => void;
  tiles: boolean;
  onTiles: (on: boolean) => void;
};

// The tools under the preview: the background it is shown on.
export function PreviewTools({
  background,
  onBackground,
  tiles,
  onTiles,
}: Props) {
  const t = useT();
  return (
    <>
      <div
        role="group"
        aria-label={t("preview.background")}
        className="mt-2 flex rounded-md border border-border bg-bg p-0.5 text-xs"
      >
        {previewBackgrounds.map((value) => (
          <button
            key={value}
            type="button"
            aria-pressed={background === value}
            onClick={() => onBackground(value)}
            className={`flex-1 rounded px-2 py-1 ${background === value ? "bg-surface-2 font-semibold text-text" : "text-text-muted"}`}
          >
            {t(labels[value])}
          </button>
        ))}
      </div>
      <div className="mt-2 flex items-center justify-between text-xs">
        <span>{t("preview.tile")}</span>
        <Switch checked={tiles} onChange={onTiles} label={t("preview.tile")} />
      </div>
    </>
  );
}
