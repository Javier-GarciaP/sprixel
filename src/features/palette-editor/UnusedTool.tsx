import { useState } from "react";
import { Button } from "@/shared/components";
import { useT } from "@/shared/lib/i18n";
import { asPaletteFailure } from "./api";
import type { PaletteApi, PaletteEntry } from "./types";

type Props = {
  api: PaletteApi;
  name: string;
  // The colors of the saved palette that no sprite uses.
  unused: PaletteEntry[];
  // Unsaved edits have to wait, like the ramp.
  disabled: boolean;
  onRemoved: () => void;
};

// Remove unused colors: it lists what will go and asks before it removes.
// Nothing is removed until the person confirms.
export function UnusedTool({ api, name, unused, disabled, onRemoved }: Props) {
  const t = useT();
  const [asking, setAsking] = useState(false);
  const [failure, setFailure] = useState<string | null>(null);

  const remove = () =>
    api
      .removeUnused(name)
      .then(() => {
        setAsking(false);
        setFailure(null);
        onRemoved();
      })
      .catch((error) => {
        const { code, message } = asPaletteFailure(error);
        setFailure(message ?? code);
      });

  return (
    <section
      aria-label={t("palette.unusedAsk")}
      className="flex flex-col gap-2 rounded-md border border-border p-3"
    >
      {!asking ? (
        <>
          <Button onClick={() => setAsking(true)} disabled={disabled}>
            {t("palette.unusedAsk")}
          </Button>
          {unused.length === 0 && (
            <p className="text-xs text-text-muted">{t("palette.unusedNone")}</p>
          )}
        </>
      ) : unused.length === 0 ? (
        <>
          <p className="text-xs">{t("palette.unusedNone")}</p>
          <Button onClick={() => setAsking(false)}>{t("dialog.cancel")}</Button>
        </>
      ) : (
        <>
          <p className="text-xs">{t("palette.unusedList")}</p>
          <ul className="flex flex-col gap-1">
            {unused.map((color) => (
              <li key={color.key} className="flex items-center gap-2 text-sm">
                <span
                  className="h-4 w-4 rounded border border-border-strong"
                  style={{ background: color.hex }}
                />
                <span className="w-5 text-center font-mono text-xs">
                  {color.key}
                </span>
                <span className="truncate">{color.name}</span>
              </li>
            ))}
          </ul>
          {failure && (
            <p role="alert" className="text-xs text-error">
              {failure}
            </p>
          )}
          <div className="flex gap-2">
            <Button variant="danger" onClick={remove}>
              {t("palette.unusedConfirm", { count: unused.length })}
            </Button>
            <Button onClick={() => setAsking(false)}>
              {t("dialog.cancel")}
            </Button>
          </div>
        </>
      )}
    </section>
  );
}
