import { Switch } from "@/shared/components";
import { useT } from "@/shared/lib/i18n";

export const MIN_OPACITY = 10;
export const MAX_OPACITY = 80;
export const DEFAULT_OPACITY = 40;

type Props = {
  on: boolean;
  onToggle: (on: boolean) => void;
  // Percent, from 10 to 80.
  opacity: number;
  onOpacity: (percent: number) => void;
};

// The onion skin options: the switch and the opacity of the previous frame.
export function OnionPanel({ on, onToggle, opacity, onOpacity }: Props) {
  const t = useT();
  return (
    <div className="flex flex-col gap-2 text-xs text-text">
      <div className="flex items-center justify-between">
        <span>{t("editor.onionShow")}</span>
        <Switch
          checked={on}
          onChange={onToggle}
          label={t("editor.onionShow")}
        />
      </div>
      <label className="flex items-center justify-between gap-2 text-text-muted">
        {t("editor.onionOpacity")}
        <input
          type="range"
          min={MIN_OPACITY}
          max={MAX_OPACITY}
          step={5}
          value={opacity}
          disabled={!on}
          onChange={(event) => onOpacity(Number(event.target.value))}
          aria-label={t("editor.onionOpacity")}
          className="flex-1 accent-amber"
        />
        <span className="w-9 text-right font-mono text-text">{opacity}%</span>
      </label>
    </div>
  );
}
