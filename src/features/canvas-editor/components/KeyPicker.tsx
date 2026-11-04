import { useT } from "@/shared/lib/i18n";
import type { PaletteColor } from "../types";

type Props = {
  label: string;
  palette: PaletteColor[];
  // The chosen keys.
  chosen: string[];
  onToggle: (key: string) => void;
};

// A row of palette swatches to choose from. Each one is a button with the
// key and the color in its name, so it works with the keyboard and a screen
// reader.
export function KeyPicker({ label, palette, chosen, onToggle }: Props) {
  const t = useT();
  return (
    <div role="group" aria-label={label} className="flex flex-col gap-1">
      <span className="text-xs text-text-muted">{label}</span>
      <div className="flex flex-wrap gap-1">
        {palette.map(({ key, hex }) => {
          const name = t("panel.pickColor", { key, hex });
          return (
            <button
              key={key}
              type="button"
              aria-label={name}
              title={name}
              aria-pressed={chosen.includes(key)}
              onClick={() => onToggle(key)}
              style={{ backgroundColor: hex }}
              className={`h-6 w-6 rounded border-2 ${chosen.includes(key) ? "border-amber outline-2 outline-text" : "border-border"}`}
            />
          );
        })}
      </div>
    </div>
  );
}
