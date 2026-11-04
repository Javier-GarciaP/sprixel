import { useEffect, useState } from "react";
import { Button, Field } from "@/shared/components";
import { useT } from "@/shared/lib/i18n";
import { asPaletteFailure } from "./api";
import { isHex } from "./draft";
import type { PaletteApi } from "./types";

type Props = {
  api: PaletteApi;
  name: string;
  // The color the base starts with (the selected one).
  startHex: string;
  // The ramp goes into the saved palette, so unsaved edits have to wait.
  disabled: boolean;
  // Called after the ramp was added, so the dialog reads the palette again.
  onAdded: () => void;
};

// Generate ramp: tones of one color from dark to light. The preview comes
// from the core (the same tones the agent tool adds) and nothing is written
// until the button is pressed.
export function RampTool({ api, name, startHex, disabled, onAdded }: Props) {
  const t = useT();
  const [base, setBase] = useState(startHex);
  const [steps, setSteps] = useState(5);
  const [hue, setHue] = useState(0);
  const [tones, setTones] = useState<string[]>([]);
  const [failure, setFailure] = useState<string | null>(null);

  useEffect(() => {
    if (!isHex(base)) {
      setTones([]);
      return;
    }
    let stale = false;
    api
      .rampPreview(base.toUpperCase(), steps, hue)
      .then((list) => !stale && setTones(list))
      .catch(() => !stale && setTones([]));
    return () => {
      stale = true;
    };
  }, [api, base, steps, hue]);

  const add = () =>
    api
      .rampApply(name, base.toUpperCase(), steps, hue)
      .then(() => {
        setFailure(null);
        onAdded();
      })
      .catch((error) =>
        setFailure(
          t("palette.rampError", {
            message: asPaletteFailure(error).message ?? "",
          }),
        ),
      );

  return (
    <section
      aria-label={t("palette.ramp")}
      className="flex flex-col gap-2 rounded-md border border-border p-3"
    >
      <h3 className="text-xs font-semibold">{t("palette.ramp")}</h3>
      <div className="flex items-end gap-2">
        <div className="w-28">
          <Field
            label={t("palette.rampBase")}
            value={base}
            spellCheck={false}
            onChange={(event) => setBase(event.target.value)}
            error={isHex(base) ? undefined : t("palette.errorBadHex")}
          />
        </div>
        <label className="flex flex-col gap-1 text-xs text-text-muted">
          {t("palette.rampSteps")}
          <input
            type="number"
            min={3}
            max={9}
            value={steps}
            onChange={(event) =>
              setSteps(Math.min(9, Math.max(3, Number(event.target.value))))
            }
            className="w-16 rounded-md border border-border bg-bg px-2 py-2 text-sm text-text"
          />
        </label>
        <label className="flex flex-col gap-1 text-xs text-text-muted">
          {t("palette.rampHue")}
          <input
            type="number"
            min={-60}
            max={60}
            value={hue}
            onChange={(event) =>
              setHue(Math.min(60, Math.max(-60, Number(event.target.value))))
            }
            className="w-16 rounded-md border border-border bg-bg px-2 py-2 text-sm text-text"
          />
        </label>
      </div>
      <ul
        aria-label={t("palette.rampPreview", { count: tones.length })}
        className="flex h-6 overflow-hidden rounded border border-border"
      >
        {tones.map((hex, i) => (
          <li
            key={`${hex}-${i}`}
            title={hex}
            className="flex-1"
            style={{ background: hex }}
          />
        ))}
      </ul>
      {disabled && (
        <p className="text-xs text-text-muted">{t("palette.rampDirty")}</p>
      )}
      {failure && (
        <p role="alert" className="text-xs text-error">
          {failure}
        </p>
      )}
      <Button onClick={add} disabled={disabled || tones.length === 0}>
        {t("palette.rampAdd")}
      </Button>
    </section>
  );
}
