import { Button } from "@/shared/components";
import { useT, type TranslationKey } from "@/shared/lib/i18n";
import type { Card, CardState } from "./clients";

const stateText: Record<CardState, TranslationKey> = {
  connected: "agents.state.connected",
  notConnected: "agents.state.notConnected",
  waiting: "agents.state.waiting",
  lost: "agents.state.lost",
};

const border: Record<CardState, string> = {
  connected: "border-success",
  notConnected: "border-border",
  waiting: "border-amber",
  lost: "border-error",
};

const textTone: Record<CardState, string> = {
  connected: "text-success",
  notConnected: "text-text-faint",
  waiting: "text-amber",
  lost: "text-error",
};

const dotTone: Record<CardState, string> = {
  connected: "bg-success",
  notConnected: "bg-text-faint",
  waiting: "bg-amber",
  lost: "bg-error",
};

type Props = {
  card: Card;
  // Opens the Connect dialog on this agent (Connect, View command, Reconnect).
  onConnect: () => void;
  onDisconnect: () => void;
  onAnswer: (allow: boolean) => void;
  onResume: () => void;
};

// One agent: its state and what the person can do with it.
export function AgentCard({
  card,
  onConnect,
  onDisconnect,
  onAnswer,
  onResume,
}: Props) {
  const t = useT();
  const { state, kind } = card;
  return (
    <section
      aria-label={kind.name}
      className={`rounded-lg border-2 bg-surface p-4 ${border[state]}`}
    >
      <div className="flex items-center justify-between gap-3">
        <div className="flex min-w-0 items-center gap-3">
          <span
            aria-hidden
            className="flex h-10 w-10 shrink-0 items-center justify-center rounded-md bg-surface-2 font-title font-bold"
          >
            {kind.name[0]}
          </span>
          <span className="truncate font-title text-lg font-bold">
            {kind.name}
          </span>
        </div>
        <span
          className={`flex shrink-0 items-center gap-2 text-xs font-semibold ${textTone[state]}`}
        >
          <span
            className={`h-2 w-2 rounded-full ${dotTone[state]}`}
            aria-hidden
          />
          {t(stateText[state])}
        </span>
      </div>
      <div className="mt-3 grid grid-cols-2 gap-2">
        {state === "notConnected" && (
          <Button
            variant="primary"
            className="col-span-2 justify-center"
            onClick={onConnect}
          >
            {t("agents.connect")}
          </Button>
        )}
        {state === "waiting" && (
          <>
            <Button className="justify-center" onClick={() => onAnswer(true)}>
              {t("agents.allow")}
            </Button>
            <Button className="justify-center" onClick={() => onAnswer(false)}>
              {t("agents.reject")}
            </Button>
          </>
        )}
        {(state === "connected" || state === "lost") && (
          <>
            <Button
              variant={state === "lost" ? "primary" : "secondary"}
              className="justify-center"
              onClick={onConnect}
            >
              {t(state === "lost" ? "agents.reconnect" : "agents.viewCommand")}
            </Button>
            <Button className="justify-center" onClick={onDisconnect}>
              {t("agents.disconnect")}
            </Button>
          </>
        )}
      </div>
      {card.stopped && (
        <div className="mt-3 flex items-center justify-between gap-3 text-xs text-warning">
          <span>{t("agents.stopped")}</span>
          <button
            type="button"
            className="font-semibold underline"
            onClick={onResume}
          >
            {t("agents.resume")}
          </button>
        </div>
      )}
    </section>
  );
}
