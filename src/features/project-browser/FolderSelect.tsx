import { useT } from "@/shared/lib/i18n";
import { pathKey, type FolderOption } from "./paths";

type Props = {
  options: FolderOption[];
  value: string[];
  onChange: (path: string[]) => void;
};

// The list "Folder" of the dialogs.
export function FolderSelect({ options, value, onChange }: Props) {
  const t = useT();
  return (
    <label className="flex flex-col gap-1 text-xs text-text-muted">
      {t("dialog.folder")}
      <select
        value={pathKey(value)}
        onChange={(event) =>
          onChange(
            options.find((o) => pathKey(o.path) === event.target.value)?.path ??
              [],
          )
        }
        className="rounded-md border border-border bg-bg px-3 py-2 text-sm text-text"
      >
        {options.map((option) => (
          <option key={pathKey(option.path)} value={pathKey(option.path)}>
            {option.label}
          </option>
        ))}
      </select>
    </label>
  );
}
