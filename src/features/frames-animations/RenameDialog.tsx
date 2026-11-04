import { useState } from "react";
import { Button, Dialog, Field } from "@/shared/components";
import { useT } from "@/shared/lib/i18n";

type Props = {
  name: string;
  // The names of the OTHER animations of the sprite.
  others: string[];
  onSave: (newName: string) => Promise<unknown>;
  onClose: () => void;
};

// A small dialog to give an animation another name.
export function RenameDialog({ name, others, onSave, onClose }: Props) {
  const t = useT();
  const [value, setValue] = useState(name);
  const [failure, setFailure] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const next = value.trim();
  const taken = others.some((o) => o.toLowerCase() === next.toLowerCase());
  const error = taken ? t("anim.errorTaken") : undefined;
  const valid = next !== "" && !taken && next !== name;

  const submit = async () => {
    setBusy(true);
    try {
      await onSave(next);
      onClose();
    } catch (e) {
      const { code, message } = e as { code?: string; message?: string };
      setFailure(
        code === "name_exists" ? t("anim.errorTaken") : message || String(e),
      );
      setBusy(false);
    }
  };

  return (
    <Dialog
      title={t("anim.renameTitle")}
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
        <Field
          label={t("dialog.name")}
          value={value}
          autoFocus
          error={error}
          onChange={(event) => {
            setValue(event.target.value);
            setFailure(null);
          }}
        />
        {failure && (
          <p role="alert" className="text-xs text-error">
            {failure}
          </p>
        )}
        <div className="flex justify-end gap-2">
          <Button onClick={onClose}>{t("dialog.cancel")}</Button>
          <Button type="submit" variant="primary" disabled={!valid || busy}>
            {t("anim.save")}
          </Button>
        </div>
      </form>
    </Dialog>
  );
}
