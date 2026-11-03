import { useT } from "@/shared/lib/i18n";
import type { ScopeKind } from "./types";

type Choice = {
  kind: ScopeKind;
  title: string;
  detail: string;
  disabled: boolean;
};

type Props = {
  choices: Choice[];
  value: ScopeKind;
  onChange: (kind: ScopeKind) => void;
  // The list of folders, shown when the folder card is chosen.
  folders: { path: string[]; label: string }[];
  folder: string[];
  onFolder: (path: string[]) => void;
};

// Step 1: the three cards (project, folder, selection).
export function ScopeStep({
  choices,
  value,
  onChange,
  folders,
  folder,
  onFolder,
}: Props) {
  const t = useT();
  return (
    <div className="flex flex-col gap-2">
      <div className="grid grid-cols-3 gap-2">
        {choices.map((choice) => (
          <button
            key={choice.kind}
            type="button"
            disabled={choice.disabled}
            aria-pressed={value === choice.kind}
            aria-label={`${choice.title} ${choice.detail}`}
            onClick={() => onChange(choice.kind)}
            className={`flex flex-col rounded-md border px-3 py-2 text-left disabled:cursor-not-allowed disabled:opacity-50 ${value === choice.kind ? "border-amber bg-amber/20" : "border-border bg-surface"}`}
          >
            <span className="text-sm font-semibold">{choice.title}</span>
            <span className="font-mono text-xs text-text-muted">
              {choice.detail}
            </span>
          </button>
        ))}
      </div>
      {value === "folder" && folders.length > 0 && (
        <label className="flex flex-col gap-1 text-xs text-text-muted">
          {t("export.scope.folderList")}
          <select
            value={folder.join("/")}
            onChange={(event) =>
              onFolder(
                folders.find((f) => f.path.join("/") === event.target.value)
                  ?.path ?? [],
              )
            }
            className="rounded-md border border-border bg-bg px-3 py-2 text-sm text-text"
          >
            {folders.map((f) => (
              <option key={f.path.join("/")} value={f.path.join("/")}>
                {f.label}
              </option>
            ))}
          </select>
        </label>
      )}
    </div>
  );
}
