import { useState } from "react";
import { Button, Dialog, Field } from "@/shared/components";
import { asCoreFailure } from "@/shared/lib/core";
import { useT } from "@/shared/lib/i18n";
import { FolderSelect } from "./FolderSelect";
import { nameErrorKey } from "./nameError";
import type { FolderOption } from "./paths";
import { useNameCheck } from "./useNameCheck";

type Props = {
  folders: FolderOption[];
  // The folder that is selected in the Project tab.
  defaultParent: string[];
  create: (parent: string[], name: string) => Promise<unknown>;
  onClose: () => void;
};

// A small dialog to make a folder inside the project root or another folder.
export function NewFolderDialog({
  folders,
  defaultParent,
  create,
  onClose,
}: Props) {
  const t = useT();
  const [name, setName] = useState("");
  const [typed, setTyped] = useState(false);
  const [parent, setParent] = useState(defaultParent);
  const [failure, setFailure] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const check = useNameCheck(parent, name.trim());
  const errorKey = nameErrorKey(check, typed);

  const submit = async () => {
    setBusy(true);
    try {
      await create(parent, name.trim());
      onClose();
    } catch (error) {
      setFailure(asCoreFailure(error).message);
      setBusy(false);
    }
  };

  return (
    <Dialog
      title={t("newFolder.title")}
      closeLabel={t("dialog.close")}
      onClose={onClose}
    >
      <form
        className="flex flex-col gap-4"
        onSubmit={(event) => {
          event.preventDefault();
          if (check.status === "ok" && !busy) submit();
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
        <FolderSelect
          options={folders}
          value={parent}
          onChange={(path) => setParent(path)}
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
            disabled={check.status !== "ok" || busy}
          >
            {t("newFolder.create")}
          </Button>
        </div>
      </form>
    </Dialog>
  );
}
