import { useState } from "react";
import { Button, Dialog } from "@/shared/components";
import { useT, type TranslationKey } from "@/shared/lib/i18n";
import type { AgentClient, AgentsApi, Permissions } from "./types";

const abilities: { name: keyof Permissions; label: TranslationKey }[] = [
  { name: "create", label: "agents.permissions.create" },
  { name: "edit", label: "agents.permissions.edit" },
  { name: "export", label: "agents.permissions.export" },
  { name: "delete", label: "agents.permissions.delete" },
];

const idOf = (c: AgentClient) => `${c.version}\u0000${c.name}`;

type Props = {
  api: AgentsApi;
  clients: AgentClient[];
  // What the agent would be allowed to do. Not shown until the core says.
  permissions: Permissions | null;
};

// The prompt for a new agent. It is mounted once by the app shell, so it
// appears on any tab. The name is text of the agent: React shows it as plain
// text, never as HTML (the core also removes invisible characters).
export function ApprovalDialog({ api, clients, permissions }: Props) {
  const t = useT();
  // Closing the dialog does not answer: the agent keeps waiting and can be
  // answered from its card in the Agents tab.
  const [closed, setClosed] = useState<string[]>([]);
  const waiting = clients.find(
    (c) => c.state === "pending" && !closed.includes(idOf(c)),
  );
  if (!waiting) return null;

  const answer = (allow: boolean) =>
    void api.answer(waiting.name, waiting.version, allow).catch(() => {});
  const names = (allowed: boolean) =>
    abilities
      .filter(({ name }) => permissions?.[name] === allowed)
      .map(({ label }) => t(label))
      .join(", ") || t("agents.approve.nothing");

  return (
    <Dialog
      title={t("agents.approve.title", { agent: waiting.name })}
      closeLabel={t("agents.approve.close")}
      onClose={() => setClosed((now) => [...now, idOf(waiting)])}
    >
      <p className="text-text-muted">{t("agents.approve.text")}</p>
      <dl className="mt-4 grid grid-cols-[auto_1fr] gap-x-6 gap-y-2 rounded-md border border-border bg-bg p-4 text-sm">
        <dt className="text-text-muted">{t("agents.approve.connection")}</dt>
        <dd className="text-right">{t("agents.approve.local")}</dd>
        {permissions && (
          <>
            <dt className="text-text-muted">{t("agents.approve.can")}</dt>
            <dd className="text-right">{names(true)}</dd>
            <dt className="text-text-muted">{t("agents.approve.cannot")}</dt>
            <dd className="text-right">{names(false)}</dd>
          </>
        )}
      </dl>
      <div className="mt-5 flex justify-end gap-3">
        <Button onClick={() => answer(false)}>
          {t("agents.approve.reject")}
        </Button>
        <Button variant="primary" onClick={() => answer(true)}>
          {t("agents.approve.allow")}
        </Button>
      </div>
    </Dialog>
  );
}
