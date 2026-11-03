import type { AgentClient } from "./types";

// The two agents of the interface. Any MCP client can connect, but only
// these two get a card and a one-command setup.
export const agentKinds = [
  { id: "claude", name: "Claude Code" },
  { id: "opencode", name: "OpenCode" },
] as const;

export type AgentKind = (typeof agentKinds)[number];
export type AgentKindId = AgentKind["id"];

// `lost` is drawn but the core does not report it yet (it comes after the
// demo, with the detection of a lost connection).
export type CardState = "connected" | "notConnected" | "waiting" | "lost";

// "claude-code", "Claude Code" and "claude_code" are the same agent.
const key = (text: string) => text.toLowerCase().replace(/[^a-z0-9]/g, "");

export type Card = {
  kind: AgentKind;
  state: CardState;
  // Every version of the agent that the core knows. Disconnect removes all.
  clients: AgentClient[];
  stopped: boolean;
};

// What each card shows, from the list of clients of the core.
export function cardsFor(clients: AgentClient[]): Card[] {
  return agentKinds.map((kind) => {
    const mine = clients.filter((c) => key(c.name) === key(kind.name));
    const allowed = mine.filter((c) => c.state === "allowed");
    const waiting = mine.some((c) => c.state === "pending");
    const state: CardState = allowed.length
      ? "connected"
      : waiting
        ? "waiting"
        : "notConnected";
    return {
      kind,
      state,
      clients: mine,
      stopped: allowed.some((c) => c.stopped),
    };
  });
}

// The name an agent has in the core, for the buttons that name it.
export const nameOf = (card: Card) => card.clients[0]?.name ?? card.kind.name;

// The command the person pastes in a terminal. It is made from the id of the
// agent, so it stays the same as the CLI (`sprixel mcp add <client>`).
export const commandFor = (id: AgentKindId) => `sprixel mcp add ${id}`;
