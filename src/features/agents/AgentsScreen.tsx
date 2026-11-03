import { useState } from "react";
import { useT } from "@/shared/lib/i18n";
import { AgentCard } from "./AgentCard";
import { cardsFor, nameOf, type AgentKindId } from "./clients";
import type { AgentsApi } from "./types";
import type { AgentsState } from "./useAgents";

type Props = { api: AgentsApi; state: AgentsState };

// The Agents tab: one card for each agent, what they did and what they may do.
export function AgentsScreen({ api, state }: Props) {
  const t = useT();
  const cards = cardsFor(state.clients);
  // The agent whose Connect dialog is open (built in the next task).
  const [, setConnecting] = useState<AgentKindId | null>(null);
  const lost = cards.find((card) => card.state === "lost");

  return (
    <div className="flex h-full">
      <div className="min-w-0 flex-1 overflow-auto p-10">
        <h1 className="font-title text-3xl font-bold">{t("agents.title")}</h1>
        <p className="mt-1 text-text-muted">{t("agents.subtitle")}</p>
        {lost && (
          <p
            role="alert"
            className="mt-6 rounded-md border border-error bg-error/10 px-4 py-3 text-sm"
          >
            {t("agents.lostBanner", { agent: nameOf(lost) })}
          </p>
        )}
        <div className="mt-6 grid grid-cols-2 gap-4">
          {cards.map((card) => (
            <AgentCard
              key={card.kind.id}
              card={card}
              onConnect={() => setConnecting(card.kind.id)}
              onDisconnect={() =>
                void Promise.all(
                  card.clients.map((c) => api.remove(c.name, c.version)),
                )
              }
              onAnswer={(allow) =>
                void Promise.all(
                  card.clients
                    .filter((c) => c.state === "pending")
                    .map((c) => api.answer(c.name, c.version, allow)),
                )
              }
              onResume={() =>
                void Promise.all(
                  card.clients
                    .filter((c) => c.stopped)
                    .map((c) => api.resume(c.name)),
                )
              }
            />
          ))}
        </div>
      </div>
    </div>
  );
}
