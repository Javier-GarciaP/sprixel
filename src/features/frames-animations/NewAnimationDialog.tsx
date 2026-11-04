import { useState } from "react";
import { Button, Dialog, Field } from "@/shared/components";
import { useT } from "@/shared/lib/i18n";
import { ModePicker } from "./ModePicker";
import type { NewAnimation, RepeatMode } from "./types";
import { MAX_NEW_FRAMES, validateNew } from "./validate";

type Props = {
  // The names the animations of the sprite already use.
  others: string[];
  // A failure of the core is shown in the dialog.
  onSave: (animation: NewAnimation) => Promise<unknown>;
  onClose: () => void;
};

// The dialog of "New animation...": a name, how many frames to copy from the
// base frame, the speed and what happens at the end. A repeated name is shown
// as an error under the field and the button stays off.
export function NewAnimationDialog({ others, onSave, onClose }: Props) {
  const t = useT();
  const [draft, setDraft] = useState({ name: "", count: "4", fps: "12" });
  const [mode, setMode] = useState<RepeatMode>("loop");
  const [typed, setTyped] = useState(false);
  const [failure, setFailure] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const errors = validateNew(draft, others);
  const valid = Object.keys(errors).length === 0;

  const change = (field: keyof typeof draft) => (value: string) => {
    setDraft({ ...draft, [field]: value });
    if (field === "name") setTyped(true);
    setFailure(null);
  };

  const submit = async () => {
    setBusy(true);
    try {
      await onSave({
        name: draft.name.trim(),
        count: Number(draft.count),
        fps: Number(draft.fps),
        mode,
      });
      onClose();
    } catch (error) {
      const { code, message } = error as { code?: string; message?: string };
      setFailure(
        code === "name_exists"
          ? t("anim.errorTaken")
          : message || String(error),
      );
      setBusy(false);
    }
  };

  const nameError =
    errors.name === "taken"
      ? t("anim.errorTaken")
      : errors.name === "empty" && typed
        ? t("anim.errorEmpty")
        : undefined;

  return (
    <Dialog
      title={t("anim.newTitle")}
      closeLabel={t("dialog.close")}
      onClose={onClose}
    >
      <form
        className="flex flex-col gap-4"
        onSubmit={(event) => {
          event.preventDefault();
          if (valid && !busy) void submit();
        }}
      >
        <p className="text-xs text-text-muted">{t("anim.newHint")}</p>
        <Field
          label={t("dialog.name")}
          value={draft.name}
          autoFocus
          error={nameError}
          onChange={(event) => change("name")(event.target.value)}
        />
        <div className="grid grid-cols-2 gap-3">
          <Field
            label={t("anim.count")}
            inputMode="numeric"
            value={draft.count}
            error={
              errors.count
                ? t("anim.errorCount", { max: MAX_NEW_FRAMES })
                : undefined
            }
            onChange={(event) => change("count")(event.target.value)}
          />
          <Field
            label={t("anim.fps")}
            inputMode="numeric"
            value={draft.fps}
            error={errors.fps ? t("anim.errorFps") : undefined}
            onChange={(event) => change("fps")(event.target.value)}
          />
        </div>
        <ModePicker value={mode} onChange={setMode} />
        {failure && (
          <p role="alert" className="text-xs text-error">
            {failure}
          </p>
        )}
        <div className="flex justify-end gap-2">
          <Button onClick={onClose}>{t("dialog.cancel")}</Button>
          <Button type="submit" variant="primary" disabled={!valid || busy}>
            {t("anim.create")}
          </Button>
        </div>
      </form>
    </Dialog>
  );
}
