import { useState } from "react";
import { Switch } from "@/shared/components";
import { useT, type TranslationKey } from "@/shared/lib/i18n";
import type { AgentsApi, Permissions } from "./types";

const switches: { name: keyof Permissions; label: TranslationKey }[] = [
  { name: "create", label: "agents.permissions.create" },
  { name: "edit", label: "agents.permissions.edit" },
  { name: "delete", label: "agents.permissions.delete" },
  { name: "export", label: "agents.permissions.export" },
  { name: "askOnConnect", label: "agents.permissions.askOnConnect" },
];

// What agents may do. A change is sent to the core at once, and the core
// applies it to the next call of every agent. The switches show what the core
// says: they move when the core sends its answer.
export function PermissionsPanel({
  api,
  permissions,
}: {
  api: AgentsApi;
  permissions: Permissions;
}) {
  const t = useT();
  const [failed, setFailed] = useState(false);

  const change = (name: keyof Permissions, value: boolean) => {
    setFailed(false);
    api
      .setPermissions({ ...permissions, [name]: value })
      .catch(() => setFailed(true));
  };

  return (
    <aside className="w-80 shrink-0 overflow-auto border-l border-border p-7">
      <h2 className="font-title text-xl font-bold">
        {t("agents.permissions")}
      </h2>
      <p className="mt-3 text-sm text-text-muted">
        {t("agents.permissions.hint")}
      </p>
      <ul className="mt-4 flex flex-col gap-4 rounded-lg border border-border bg-surface p-4">
        {switches.map(({ name, label }) => (
          <li
            key={name}
            className="flex items-center justify-between gap-3 text-sm"
          >
            <span>{t(label)}</span>
            <Switch
              label={t(label)}
              checked={permissions[name]}
              onChange={(value) => change(name, value)}
            />
          </li>
        ))}
      </ul>
      <p className="mt-4 text-xs text-text-faint">
        {t("agents.permissions.note")}
      </p>
      {failed && (
        <p role="alert" className="mt-2 text-xs text-error">
          {t("agents.permissions.error")}
        </p>
      )}
    </aside>
  );
}
