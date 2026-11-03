import { useEffect, useState } from "react";
import { Button, Dialog, Icon } from "@/shared/components";
import { useT } from "@/shared/lib/i18n";
import { agentKinds, commandFor, type AgentKindId, type Card } from "./clients";

// After this long without a connection the dialog suggests `sprixel doctor`.
export const CONNECT_TIMEOUT_MS = 60_000;

type Props = {
  // The tab that opens first.
  initial: AgentKindId;
  // The state of the agents, to know when the one that waits has connected.
  cards: Card[];
  onClose: () => void;
  timeoutMs?: number;
};

const Step = ({ n, children }: { n: number; children: string }) => (
  <h3 className="flex items-center gap-3 font-title text-base font-bold">
    <span
      aria-hidden
      className="flex h-6 w-6 items-center justify-center rounded bg-amber text-sm text-ink"
    >
      {n}
    </span>
    {children}
  </h3>
);

// Shows the command to connect an agent and waits for it. The body has a
// fixed height, and the help text has its own place, so nothing moves when
// the tab changes or when the help appears.
export function ConnectDialog({
  initial,
  cards,
  onClose,
  timeoutMs = CONNECT_TIMEOUT_MS,
}: Props) {
  const t = useT();
  const [tab, setTab] = useState(initial);
  const [copied, setCopied] = useState(false);
  const [copyFailed, setCopyFailed] = useState(false);
  const [slow, setSlow] = useState(false);

  useEffect(() => {
    const timer = setTimeout(() => setSlow(true), timeoutMs);
    return () => clearTimeout(timer);
  }, [timeoutMs]);

  const kind = agentKinds.find((k) => k.id === tab) ?? agentKinds[0];
  const state = cards.find((card) => card.kind.id === tab)?.state;
  const connected = state === "connected";
  const command = commandFor(tab);

  const copy = () => {
    setCopyFailed(false);
    const clipboard = navigator.clipboard as Clipboard | undefined;
    (clipboard?.writeText(command) ?? Promise.reject(new Error("no clipboard")))
      .then(() => setCopied(true))
      .catch(() => setCopyFailed(true));
  };

  const select = (id: AgentKindId) => {
    setTab(id);
    setCopied(false);
    setCopyFailed(false);
  };

  return (
    <Dialog
      title={t("agents.connect.title")}
      closeLabel={t("agents.connect.close")}
      onClose={onClose}
    >
      <div className="flex h-[440px] flex-col gap-3">
        <div
          role="tablist"
          className="flex w-fit gap-1 rounded-md border border-border bg-bg p-1"
        >
          {agentKinds.map((k) => (
            <button
              key={k.id}
              type="button"
              role="tab"
              aria-selected={tab === k.id}
              onClick={() => select(k.id)}
              className={`rounded px-3 py-1 text-sm font-semibold ${tab === k.id ? "bg-surface-2 text-text" : "text-text-muted"}`}
            >
              {k.name}
            </button>
          ))}
        </div>

        <Step n={1}>{t("agents.connect.step1")}</Step>
        <div className="flex h-32 flex-col justify-between rounded-md border border-border bg-bg p-3">
          <code className="break-all font-mono text-sm">{command}</code>
          <div className="flex items-end justify-between gap-3">
            <p className="text-xs text-text-muted">
              {t("agents.connect.install")}
            </p>
            <Button
              variant="primary"
              className="min-w-28 shrink-0 justify-center"
              onClick={copy}
            >
              <Icon name={copied ? "check" : "copy"} />
              {t(copied ? "agents.connect.copied" : "agents.connect.copy")}
            </Button>
          </div>
        </div>

        <Step n={2}>{t("agents.connect.step2")}</Step>
        <Step n={3}>{t("agents.connect.step3")}</Step>
        <div
          role="status"
          className={`flex items-center gap-3 rounded-md border px-4 py-3 text-sm font-semibold ${connected ? "border-success text-success" : "border-amber text-amber"}`}
        >
          <span
            aria-hidden
            className={`h-2 w-2 rounded-full ${connected ? "bg-success" : "bg-amber"}`}
          />
          {t(
            connected
              ? "agents.state.connected"
              : state === "waiting"
                ? "agents.state.waiting"
                : "agents.connect.waiting",
          )}
        </div>
        <p
          role={copyFailed ? "alert" : undefined}
          className={`min-h-10 text-xs ${copyFailed ? "text-error" : "text-text-muted"}`}
        >
          {copyFailed
            ? t("agents.connect.copyError")
            : connected
              ? ""
              : t(slow ? "agents.connect.slow" : "agents.connect.help", {
                  agent: kind.name,
                })}
        </p>
        <div className="mt-auto flex justify-end">
          <Button className="min-w-24 justify-center" onClick={onClose}>
            {t(connected ? "agents.connect.close" : "agents.connect.cancel")}
          </Button>
        </div>
      </div>
    </Dialog>
  );
}
