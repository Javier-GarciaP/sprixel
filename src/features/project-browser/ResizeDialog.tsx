import { useState } from "react";
import { Button, Dialog, Field } from "@/shared/components";
import { asCoreFailure } from "@/shared/lib/core";
import { useT } from "@/shared/lib/i18n";
import { MAX_SIZE } from "./NewSpriteDialog";

// The nine places where the old drawing can stay, in reading order. The
// names are the ones the core uses.
export const ANCHORS = [
  "top-left",
  "top",
  "top-right",
  "left",
  "center",
  "right",
  "bottom-left",
  "bottom",
  "bottom-right",
] as const;
export type CanvasAnchor = (typeof ANCHORS)[number];

type Props = {
  sprite: { folder: string[]; name: string; width: number; height: number };
  resize: (
    width: number,
    height: number,
    anchor: CanvasAnchor,
  ) => Promise<unknown>;
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

// "Change canvas": the size of a sprite is chosen when it is created and
// changed only here. The anchor says where the old drawing stays.
export function ResizeDialog({ sprite, resize, onClose }: Props) {
  const t = useT();
  const [width, setWidth] = useState(String(sprite.width));
  const [height, setHeight] = useState(String(sprite.height));
  const [anchor, setAnchor] = useState<CanvasAnchor>("center");
  const [failure, setFailure] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const w = side(width);
  const h = side(height);
  const changed = w !== sprite.width || h !== sprite.height;
  const valid = w !== null && h !== null && changed;
  // Pixels (and anchors such as `feet`) can be lost only when a side shrinks.
  const crops =
    (w ?? sprite.width) < sprite.width || (h ?? sprite.height) < sprite.height;

  const submit = async () => {
    if (w === null || h === null) return;
    setBusy(true);
    try {
      await resize(w, h, anchor);
      onClose();
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
        setFailure(null);
      }}
    />
  );

  return (
    <Dialog
      title={t("resize.title")}
      closeLabel={t("dialog.close")}
      onClose={onClose}
    >
      <form
        className="flex flex-col gap-4"
        onSubmit={(event) => {
          event.preventDefault();
          if (valid && !busy) submit();
        }}
      >
        <p className="text-xs text-text-muted">
          {t("resize.current", {
            name: sprite.name,
            width: sprite.width,
            height: sprite.height,
          })}
        </p>
        <div className="grid grid-cols-2 gap-3">
          {sizeField(t("newSprite.width"), width, setWidth)}
          {sizeField(t("newSprite.height"), height, setHeight)}
        </div>

        <fieldset className="flex flex-col gap-2">
          <legend className="mb-1 text-xs text-text-muted">
            {t("resize.anchor")}
          </legend>
          <div
            role="radiogroup"
            aria-label={t("resize.anchor")}
            className="grid w-28 grid-cols-3 gap-1"
          >
            {ANCHORS.map((id) => (
              <button
                key={id}
                type="button"
                role="radio"
                aria-checked={anchor === id}
                aria-label={t(`resize.anchor.${id}`)}
                title={t(`resize.anchor.${id}`)}
                onClick={() => setAnchor(id)}
                className={`h-8 rounded border ${anchor === id ? "border-amber bg-amber/30" : "border-border bg-surface-2"}`}
              />
            ))}
          </div>
          <p className="text-xs text-text-muted">{t("resize.anchorHint")}</p>
        </fieldset>

        {crops && (
          <p
            role="status"
            className="rounded-md border border-warning p-3 text-xs text-warning"
          >
            {t("resize.cropWarning")}
          </p>
        )}
        {failure && (
          <p role="alert" className="text-xs text-error">
            {t("error.io", { message: failure })}
          </p>
        )}
        <div className="flex justify-end gap-2">
          <Button onClick={onClose}>{t("dialog.cancel")}</Button>
          <Button type="submit" variant="primary" disabled={!valid || busy}>
            {t("resize.apply")}
          </Button>
        </div>
      </form>
    </Dialog>
  );
}
