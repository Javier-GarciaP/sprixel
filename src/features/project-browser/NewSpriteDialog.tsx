import { useState } from "react";
import { Button, Dialog, Field } from "@/shared/components";
import {
  asCoreFailure,
  type PaletteSummary,
  type SpriteTarget,
} from "@/shared/lib/core";
import { useT } from "@/shared/lib/i18n";
import { FolderSelect } from "./FolderSelect";
import { nameErrorKey } from "./nameError";
import type { FolderOption } from "./paths";
import { useNameCheck } from "./useNameCheck";

export const PRESETS = [8, 16, 32, 64];
export const MAX_SIZE = 512;
// From this size the dialog warns that AI drawing and the editor suffer.
export const LARGE_SIZE = 128;

type Props = {
  folders: FolderOption[];
  palettes: PaletteSummary[];
  defaultFolder: string[];
  create: (
    folder: string[],
    name: string,
    width: number,
    height: number,
    palette: string | null,
  ) => Promise<SpriteTarget>;
  // Called with the new sprite after it was created.
  onCreated: (sprite: SpriteTarget) => void;
  onClose: () => void;
};

// A side is valid when it is a whole number from 1 to 512.
const side = (text: string) => {
  const value = Number(text);
  return text.trim() !== "" &&
    Number.isInteger(value) &&
    value >= 1 &&
    value <= MAX_SIZE
    ? value
    : null;
};

export function NewSpriteDialog({
  folders,
  palettes,
  defaultFolder,
  create,
  onCreated,
  onClose,
}: Props) {
  const t = useT();
  const [name, setName] = useState("");
  const [typed, setTyped] = useState(false);
  const [folder, setFolder] = useState(defaultFolder);
  const [preset, setPreset] = useState<number | null>(16);
  const [width, setWidth] = useState("16");
  const [height, setHeight] = useState("16");
  const [palette, setPalette] = useState("");
  const [failure, setFailure] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const check = useNameCheck(folder, name.trim());
  const errorKey = nameErrorKey(check, typed);

  const w = side(width);
  const h = side(height);
  const large = (w ?? 0) >= LARGE_SIZE || (h ?? 0) >= LARGE_SIZE;
  const sizeOk = w !== null && h !== null;

  const choosePreset = (size: number) => {
    setPreset(size);
    setWidth(String(size));
    setHeight(String(size));
  };

  const submit = async () => {
    if (w === null || h === null) return;
    setBusy(true);
    try {
      const sprite = await create(folder, name.trim(), w, h, palette || null);
      onClose();
      onCreated(sprite);
    } catch (error) {
      setFailure(asCoreFailure(error).message);
      setBusy(false);
    }
  };

  const sizeField = (
    label: string,
    value: string,
    set: (value: string) => void,
  ) => (
    <Field
      label={label}
      type="number"
      inputMode="numeric"
      min={1}
      max={MAX_SIZE}
      value={value}
      error={side(value) === null ? t("newSprite.sizeError") : undefined}
      onChange={(event) => {
        set(event.target.value);
        // Typing a size is not one of the presets any more.
        setPreset(null);
      }}
    />
  );

  return (
    <Dialog
      title={t("newSprite.title")}
      closeLabel={t("dialog.close")}
      onClose={onClose}
    >
      <form
        className="flex flex-col gap-4"
        onSubmit={(event) => {
          event.preventDefault();
          if (check.status === "ok" && sizeOk && !busy) submit();
        }}
      >
        <Field
          label={t("dialog.name")}
          value={name}
          autoFocus
          error={errorKey && t(errorKey)}
          onChange={(event) => {
            setName(event.target.value);
            setTyped(true);
            setFailure(null);
          }}
        />
        <FolderSelect options={folders} value={folder} onChange={setFolder} />

        <fieldset className="flex flex-col gap-2">
          <legend className="mb-1 text-xs text-text-muted">
            {t("newSprite.size")}
          </legend>
          <div className="flex gap-2">
            {PRESETS.map((size) => (
              <Button
                key={size}
                aria-pressed={preset === size}
                variant={preset === size ? "primary" : "secondary"}
                onClick={() => choosePreset(size)}
              >
                {size}x{size}
              </Button>
            ))}
          </div>
          <div className="grid grid-cols-2 gap-3">
            {sizeField(t("newSprite.width"), width, setWidth)}
            {sizeField(t("newSprite.height"), height, setHeight)}
          </div>
        </fieldset>

        {large && (
          <p
            role="status"
            className="rounded-md border border-warning p-3 text-xs text-warning"
          >
            {t("newSprite.largeWarning")}
          </p>
        )}

        <label className="flex flex-col gap-1 text-xs text-text-muted">
          {t("newSprite.palette")}
          <select
            value={palette}
            onChange={(event) => setPalette(event.target.value)}
            className="rounded-md border border-border bg-bg px-3 py-2 text-sm text-text"
          >
            <option value="">{t("newSprite.paletteNone")}</option>
            {palettes.map((p) => (
              <option key={p.name} value={p.name}>
                {p.name}
              </option>
            ))}
          </select>
        </label>

        {failure && (
          <p role="alert" className="text-xs text-error">
            {t("error.io", { message: failure })}
          </p>
        )}
        <div className="flex justify-end gap-2">
          <Button onClick={onClose}>{t("dialog.cancel")}</Button>
          <Button
            type="submit"
            variant="primary"
            disabled={check.status !== "ok" || !sizeOk || busy}
          >
            {t("newSprite.create")}
          </Button>
        </div>
      </form>
    </Dialog>
  );
}
