import { useState } from "react";
import { Button, Dialog, Field } from "@/shared/components";
import { useT, type TranslationKey } from "@/shared/lib/i18n";
import type { RepeatMode, Tag } from "./types";
import { draftOf, isValid, validateTag, type TagErrors } from "./validate";

type Props = {
  frameCount: number;
  // The animation that is edited, or none for a new one.
  tag?: Tag;
  // The names the other animations of the sprite use.
  others: string[];
  // A failure of the core is shown in the dialog.
  onSave: (tag: Tag) => Promise<unknown>;
  onClose: () => void;
};

const modes: { id: RepeatMode; labelKey: TranslationKey }[] = [
  { id: "loop", labelKey: "anim.modeLoop" },
  { id: "once", labelKey: "anim.modeOnce" },
  { id: "pingpong", labelKey: "anim.modePingpong" },
];

const messages: Record<string, TranslationKey> = {
  empty: "anim.errorEmpty",
  taken: "anim.errorTaken",
  order: "anim.errorOrder",
};

// The dialog to make or edit an animation: a name, the first and the last
// frame, the speed and what happens at the end. Errors are shown next to the
// field and the button stays off until everything is valid.
export function AnimationDialog({
  frameCount,
  tag,
  others,
  onSave,
  onClose,
}: Props) {
  const t = useT();
  const [draft, setDraft] = useState(
    tag
      ? draftOf(tag)
      : { name: "", from: "1", to: String(frameCount), fps: "12" },
  );
  const [mode, setMode] = useState<RepeatMode>(tag?.mode ?? "loop");
  const [touched, setTouched] = useState<Record<string, boolean>>({});
  const [failure, setFailure] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const errors: TagErrors = validateTag(draft, frameCount, others);

  const message = (field: keyof TagErrors) => {
    const code = errors[field];
    // An empty name is not shown as an error until the user has typed.
    if (!code || (field === "name" && code === "empty" && !touched.name)) {
      return undefined;
    }
    if (code === "range") {
      return field === "fps"
        ? t("anim.errorFps")
        : t("anim.errorFrame", { count: frameCount });
    }
    return t(messages[code]);
  };

  const change = (field: keyof typeof draft) => (value: string) => {
    setDraft({ ...draft, [field]: value });
    setTouched({ ...touched, [field]: true });
    setFailure(null);
  };

  const submit = async () => {
    setBusy(true);
    try {
      await onSave({
        name: draft.name.trim(),
        from: Number(draft.from),
        to: Number(draft.to),
        fps: Number(draft.fps),
        mode,
      });
      onClose();
    } catch (error) {
      const text = (error as { message?: string })?.message;
      setFailure(text || String(error));
      setBusy(false);
    }
  };

  return (
    <Dialog
      title={tag ? t("anim.editTitle") : t("anim.newTitle")}
      closeLabel={t("dialog.close")}
      onClose={onClose}
    >
      <form
        className="flex flex-col gap-4"
        onSubmit={(event) => {
          event.preventDefault();
          if (isValid(errors) && !busy) void submit();
        }}
      >
        <Field
          label={t("dialog.name")}
          value={draft.name}
          autoFocus
          error={message("name")}
          onChange={(event) => change("name")(event.target.value)}
        />
        <div className="grid grid-cols-3 gap-3">
          <Field
            label={t("anim.from")}
            inputMode="numeric"
            value={draft.from}
            error={message("from")}
            onChange={(event) => change("from")(event.target.value)}
          />
          <Field
            label={t("anim.to")}
            inputMode="numeric"
            value={draft.to}
            error={message("to")}
            onChange={(event) => change("to")(event.target.value)}
          />
          <Field
            label={t("anim.fps")}
            inputMode="numeric"
            value={draft.fps}
            error={message("fps")}
            onChange={(event) => change("fps")(event.target.value)}
          />
        </div>
        <fieldset className="flex flex-col gap-1">
          <legend className="mb-1 text-xs text-text-muted">
            {t("anim.mode")}
          </legend>
          <div className="flex gap-2">
            {modes.map(({ id, labelKey }) => (
              <button
                key={id}
                type="button"
                role="radio"
                aria-checked={mode === id}
                onClick={() => setMode(id)}
                className={`rounded-md border px-3 py-1.5 text-sm ${mode === id ? "border-amber bg-surface-2 font-semibold" : "border-border"}`}
              >
                {t(labelKey)}
              </button>
            ))}
          </div>
        </fieldset>
        {failure && (
          <p role="alert" className="text-xs text-error">
            {failure}
          </p>
        )}
        <div className="flex justify-end gap-2">
          <Button onClick={onClose}>{t("dialog.cancel")}</Button>
          <Button
            type="submit"
            variant="primary"
            disabled={!isValid(errors) || busy}
          >
            {tag ? t("anim.save") : t("anim.create")}
          </Button>
        </div>
      </form>
    </Dialog>
  );
}
