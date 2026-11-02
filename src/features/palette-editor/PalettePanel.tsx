import { useState } from "react";
import { Switch } from "@/shared/components";
import { useT } from "@/shared/lib/i18n";
import type { PaletteColor } from "./types";

type Props = {
  colors: PaletteColor[];
  main: string;
  secondary: string;
  onMain: (hex: string) => void;
  onSecondary: (hex: string) => void;
  // Opens the palette editor. Without it there is no Edit button.
  onEdit?: () => void;
};

const HEX = /^#?([0-9a-f]{6})$/i;
const normalizeHex = (hex: string) => hex.toUpperCase();

// A letter that can be read on top of the color.
function readableOn(hex: string): string {
  const [r, g, b] = [1, 3, 5].map((i) => parseInt(hex.slice(i, i + 2), 16));
  return r * 0.299 + g * 0.587 + b * 0.114 > 150 ? "#14141F" : "#FFFFFF";
}

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
// boxes (the system color picker) or from the hex field. "Show keys" writes
// the one-letter key on each color.
export function PalettePanel({
  colors,
  main,
  secondary,
  onMain,
  onSecondary,
  onEdit,
}: Props) {
  const t = useT();
  const [showKeys, setShowKeys] = useState(false);
  return (
    <div>
      <div className="flex flex-wrap gap-1.5">
        <span
          role="img"
          aria-label={t("editor.transparent")}
          title={t("editor.transparent")}
          className="flex h-7 w-7 items-center justify-center rounded border border-border-strong font-mono text-xs text-ink"
          style={{
            background:
              "repeating-conic-gradient(#b4b4b4 0 25%, #9c9c9c 0 50%) 0 0 / 8px 8px",
          }}
        >
          {showKeys && "."}
        </span>
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
              className={`flex h-7 w-7 items-center justify-center rounded border font-mono text-xs font-bold ${outline}`}
              style={{ background: hex, color: readableOn(hex) }}
            >
              {showKeys && color.key}
            </button>
          );
        })}
      </div>
      <HexField main={main} onMain={onMain} />
      <div className="mt-3 flex items-center justify-between text-xs text-text-muted">
        <span>{t("palette.showKeys")}</span>
        <Switch
          checked={showKeys}
          onChange={setShowKeys}
          label={t("palette.showKeys")}
        />
      </div>
      {onEdit && (
        <button
          type="button"
          onClick={onEdit}
          className="mt-3 w-full rounded-md border border-border px-2 py-1 text-xs font-semibold text-text hover:bg-surface-2"
        >
          {t("palette.edit")}
        </button>
      )}
    </div>
  );
}
