import { useState } from "react";
import { Button, Dialog, Field } from "@/shared/components";
import { useT } from "@/shared/lib/i18n";
import type { ProjectFailure } from "../api/project";

type Props = {
  defaultLocation: string;
  pickFolder: () => Promise<string | null>;
  // Asks the core to create the project. Gives the failure, or null on success.
  onCreate: (parent: string, name: string) => Promise<ProjectFailure | null>;
  onClose: () => void;
};

// The folder that will be created, written with the separator of the system
// of the chosen location.
export function projectFolder(parent: string, name: string): string {
  const separator = parent.includes("\\") && !parent.includes("/") ? "\\" : "/";
  const base = parent.replace(/[\\/]+$/, "");
  return `${base}${separator}${name}${separator}`;
}

export function NewProjectDialog({
  defaultLocation,
  pickFolder,
  onCreate,
  onClose,
}: Props) {
  const t = useT();
  const [name, setName] = useState("");
  const [location, setLocation] = useState(defaultLocation);
  const [failure, setFailure] = useState<ProjectFailure | null>(null);
  const [busy, setBusy] = useState(false);

  const choose = async () => {
    const chosen = await pickFolder();
    if (chosen) {
      setLocation(chosen);
      setFailure(null);
    }
  };

  const submit = async () => {
    setBusy(true);
    const result = await onCreate(location, name.trim());
    setBusy(false);
    if (result) setFailure(result);
    else onClose();
  };

  const nameError =
    failure?.code === "folder_exists"
      ? t("error.folderExists")
      : failure?.code === "invalid_name"
        ? t("error.invalidName")
        : undefined;

  return (
    <Dialog
      title={t("newProject.title")}
      closeLabel={t("newProject.close")}
      onClose={onClose}
    >
      <form
        className="flex flex-col gap-4"
        onSubmit={(event) => {
          event.preventDefault();
          if (name.trim() && !busy) submit();
        }}
      >
        <Field
          label={t("newProject.name")}
          value={name}
          error={nameError}
          autoFocus
          onChange={(event) => {
            setName(event.target.value);
            setFailure(null);
          }}
        />

        <div className="flex items-end gap-2">
          <div className="flex-1">
            <Field
              label={t("newProject.location")}
              value={location}
              onChange={(event) => {
                setLocation(event.target.value);
                setFailure(null);
              }}
            />
          </div>
          <Button onClick={choose}>{t("newProject.choose")}</Button>
        </div>

        <label className="flex flex-col gap-1 text-xs text-text-muted">
          {t("newProject.palette")}
          {/* Built-in palettes arrive with the palette editor. */}
          <select
            defaultValue="none"
            className="rounded-md border border-border bg-bg px-3 py-2 text-sm text-text"
          >
            <option value="none">{t("newProject.paletteNone")}</option>
          </select>
        </label>

        {name.trim() !== "" && (
          <p className="font-mono text-xs text-text-faint">
            {t("newProject.willCreate", {
              path: projectFolder(location, name),
            })}
          </p>
        )}
        {failure?.code === "io" && (
          <p role="alert" className="text-xs text-error">
            {t("error.io", { message: failure.message })}
          </p>
        )}

        <div className="flex justify-end gap-2">
          <Button onClick={onClose}>{t("newProject.cancel")}</Button>
          <Button
            type="submit"
            variant="primary"
            disabled={!name.trim() || busy}
          >
            {t("newProject.create")}
          </Button>
        </div>
      </form>
    </Dialog>
  );
}
