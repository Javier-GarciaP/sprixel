import { useState } from "react";
import { Button, Dialog, Field } from "@/shared/components";
import { asCoreFailure } from "@/shared/lib/core";
import { useT } from "@/shared/lib/i18n";
import { nameErrorKey } from "./nameError";
import { useNameCheck } from "./useNameCheck";

type Props = {
  kind: "sprite" | "folder";
  // The folder that holds the item (the root is an empty path).
  parent: string[];
  name: string;
  rename: (newName: string) => Promise<unknown>;
  onClose: () => void;
};

// Asks for a new name for a sprite or a folder, with the same rules as
// creating one.
export function RenameDialog({ kind, parent, name, rename, onClose }: Props) {
  const t = useT();
  const [value, setValue] = useState(name);
  const [failure, setFailure] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const trimmed = value.trim();
  const check = useNameCheck(parent, trimmed, name);
  // Nothing to do when the name did not change.
  const unchanged = trimmed === name;
  const errorKey = nameErrorKey(check, true);

  const submit = async () => {
    setBusy(true);
    try {
      await rename(trimmed);
      onClose();
    } catch (error) {
      setFailure(asCoreFailure(error).message);
      setBusy(false);
    }
  };

  return (
    <Dialog
      title={t(kind === "sprite" ? "rename.sprite" : "rename.folder")}
      closeLabel={t("dialog.close")}
      onClose={onClose}
    >
      <form
        className="flex flex-col gap-4"
        onSubmit={(event) => {
          event.preventDefault();
          if (check.status === "ok" && !unchanged && !busy) submit();
        }}
      >
        <Field
          label={t("dialog.name")}
          value={value}
          autoFocus
          onFocus={(event) => event.currentTarget.select()}
          error={errorKey && t(errorKey)}
          onChange={(event) => {
            setValue(event.target.value);
            setFailure(null);
          }}
        />
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
            disabled={check.status !== "ok" || unchanged || busy}
          >
            {t("rename.confirm")}
          </Button>
        </div>
      </form>
    </Dialog>
  );
}
