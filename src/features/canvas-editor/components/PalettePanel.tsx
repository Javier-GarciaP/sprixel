import { useT } from "@/shared/lib/i18n";
import type { PaletteColor } from "../types";

type Props = { colors: PaletteColor[] };

// The colors of the palette of the sprite. Choosing them is added with the
// colors task.
export function PalettePanel({ colors }: Props) {
  const t = useT();
  return (
    <div className="flex flex-wrap gap-1.5">
      <span
        role="img"
        aria-label={t("editor.transparent")}
        title={t("editor.transparent")}
        className="h-7 w-7 rounded border border-border-strong"
        style={{
          background:
            "repeating-conic-gradient(#b4b4b4 0 25%, #9c9c9c 0 50%) 0 0 / 8px 8px",
        }}
      />
      {colors.map((color) => (
        <span
          key={color.key}
          role="img"
          aria-label={color.hex}
          title={color.hex}
          className="h-7 w-7 rounded border border-border-strong"
          style={{ background: color.hex }}
        />
      ))}
    </div>
  );
}
