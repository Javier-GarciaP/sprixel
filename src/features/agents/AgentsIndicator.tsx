import { Chip } from "@/shared/components";
import { useT } from "@/shared/lib/i18n";
import { cardsFor } from "./clients";
import type { AgentClient } from "./types";

// The state of the agents in the top bar: "No agents", the names of the
// connected agents, or "Waiting for approval" (which wins, so the person
// sees that something needs an answer).
export function AgentsIndicator({ clients }: { clients: AgentClient[] }) {
  const t = useT();
  const cards = cardsFor(clients);
  if (cards.some((card) => card.state === "waiting")) {
    return <Chip tone="warning">{t("agents.indicator.waiting")}</Chip>;
  }
  const connected = cards.filter((card) => card.state === "connected");
  if (connected.length === 0) return <Chip>{t("agents.none")}</Chip>;
  return (
    <Chip tone="success">
      {connected.map((card) => card.kind.name).join(" · ")}
    </Chip>
  );
}
