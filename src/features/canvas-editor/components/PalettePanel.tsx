import { useState } from "react";
import { useT } from "@/shared/lib/i18n";
import { normalizeHex } from "../controller/colorBook";
import type { PaletteColor } from "../types";

type Props = {
  colors: PaletteColor[];
  main: string;
  secondary: string;
  onMain: (hex: string) => void;
  onSecondary: (hex: string) => void;
};

const HEX = /^#?([0-9a-f]{6})$/i;

// A hex field for the main color. It keeps what the user types until it is a
// valid color, and starts again from the color when that changes elsewhere.
function HexField({
  main,
  onMain,
}: {
  main: string;
  onMain: (hex: string) => void;
}) {
  const t = useT();
  const [text, setText] = useState(main);
  const valid = HEX.test(text);
  return (
    <input
      key={main}
      aria-label={t("colors.hex")}
      aria-invalid={!valid}
      spellCheck={false}
      value={text}
      onChange={(event) => {
        setText(event.target.value);
        const match = HEX.exec(event.target.value);
        if (match) onMain(normalizeHex(`#${match[1]}`));
      }}
      className={`mt-2 w-full rounded border bg-surface-2 px-2 py-1 font-mono text-xs text-text ${valid ? "border-border" : "border-error"}`}
    />
  );
}

// The colors of the palette of the sprite. A click chooses the main color and
// a right click the secondary one. Any other color comes from the color
// boxes (the system color picker) or from the hex field.
export function PalettePanel({
  colors,
  main,
  secondary,
  onMain,
  onSecondary,
}: Props) {
  const t = useT();
  return (
    <div>
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
        {colors.map((color) => {
          const hex = normalizeHex(color.hex);
          const outline =
            hex === normalizeHex(main)
              ? "border-amber ring-1 ring-amber"
              : hex === normalizeHex(secondary)
                ? "border-text"
                : "border-border-strong";
          return (
            <button
              key={color.key}
              type="button"
              aria-label={hex}
              aria-pressed={hex === normalizeHex(main)}
              title={hex}
              onClick={() => onMain(hex)}
              onContextMenu={(event) => {
                event.preventDefault();
                onSecondary(hex);
              }}
              className={`h-7 w-7 rounded border ${outline}`}
              style={{ background: hex }}
            />
          );
        })}
      </div>
      <HexField main={main} onMain={onMain} />
    </div>
  );
}
