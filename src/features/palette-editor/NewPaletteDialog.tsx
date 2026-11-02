import { useState } from "react";
import { Button, Dialog, Field } from "@/shared/components";
import { useT, type TranslationKey } from "@/shared/lib/i18n";
import { asPaletteFailure } from "./api";
import type { PaletteApi, Preset } from "./types";

type Props = {
  api: PaletteApi;
  // The presets to start from (read by the app, see `PaletteApi`).
  presets: Preset[];
  // Called with the name of the new palette, right before the dialog closes.
  onCreated?: (name: string) => void;
  onClose: () => void;
};

const failures: Record<string, TranslationKey> = {
  name_exists: "palette.errorNameTaken",
  invalid_name: "palette.errorInvalidName",
};

// The dialog to make a palette: empty, or from a built-in preset.
export function NewPaletteDialog({ api, presets, onCreated, onClose }: Props) {
  const t = useT();
  // `null` is the empty palette.
  const [preset, setPreset] = useState<string | null>(null);
  // The name follows the choice until the user types one.
  const [typed, setTyped] = useState<string | null>(null);
  const [failure, setFailure] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const chosen = presets.find((p) => p.id === preset);
  const name = typed ?? chosen?.name ?? t("palette.defaultName");

  const submit = async () => {
    setBusy(true);
    setFailure(null);
    try {
      const created = await api.createPalette(name.trim(), preset);
      onCreated?.(created);
      onClose();
    } catch (error) {
      const { code, message } = asPaletteFailure(error);
      setFailure(failures[code] ? t(failures[code]) : (message ?? code));
      setBusy(false);
    }
  };

  return (
    <Dialog
      title={t("palette.new")}
      closeLabel={t("dialog.close")}
      onClose={onClose}
    >
      <form
        className="flex flex-col gap-4"
        onSubmit={(event) => {
          event.preventDefault();
          if (name.trim() !== "" && !busy) submit();
        }}
      >
        <Field
          label={t("palette.newName")}
          value={name}
          error={failure ?? undefined}
          onChange={(event) => setTyped(event.target.value)}
        />
        <fieldset className="flex flex-col gap-2">
          <legend className="mb-1 text-xs text-text-muted">
            {t("palette.newStart")}
          </legend>
          <Option
            label={t("palette.newEmpty")}
            checked={preset === null}
            onSelect={() => setPreset(null)}
          />
          {presets.map((p) => (
            <Option
              key={p.id}
              label={p.name}
              detail={t("palette.newColors", { count: p.colors.length })}
              colors={p.colors.map((c) => c.hex)}
              checked={preset === p.id}
              onSelect={() => setPreset(p.id)}
            />
          ))}
        </fieldset>
        <div className="flex justify-end gap-2">
          <Button onClick={onClose}>{t("dialog.cancel")}</Button>
          <Button
            type="submit"
            variant="primary"
            disabled={name.trim() === "" || busy}
          >
            {t("palette.newCreate")}
          </Button>
        </div>
      </form>
    </Dialog>
  );
}

function Option({
  label,
  detail,
  colors,
  checked,
  onSelect,
}: {
  label: string;
  detail?: string;
  colors?: string[];
  checked: boolean;
  onSelect: () => void;
}) {
  return (
    <label
      className={`flex cursor-pointer items-center gap-3 rounded-md border px-3 py-2 text-sm ${checked ? "border-amber" : "border-border"}`}
    >
      <input
        type="radio"
        name="palette-start"
        checked={checked}
        onChange={onSelect}
      />
      <span className="flex-1">
        {label}
        {detail && <span className="ml-2 text-text-muted">{detail}</span>}
      </span>
      {colors && (
        <span aria-hidden className="flex">
          {colors.slice(0, 16).map((hex, i) => (
            <span key={i} className="h-3 w-2" style={{ background: hex }} />
          ))}
        </span>
      )}
    </label>
  );
}
