// @vitest-environment jsdom
import {
  cleanup,
  fireEvent,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { AgentCard } from "./AgentCard";
import { AgentsScreen } from "./AgentsScreen";
import { cardsFor } from "./clients";
import { renderWithApp } from "./test-utils";
import { client, FakeAgentsApi } from "./testing";
import type { AgentsState } from "./useAgents";
import { useAgents } from "./useAgents";

afterEach(cleanup);

// The screen with the state that the real hook gives, from a fake core.
function Screen({ api }: { api: FakeAgentsApi }) {
  const state: AgentsState = useAgents(api);
  return <AgentsScreen api={api} state={state} />;
}

const card = (name: string) => screen.getByRole("region", { name });

describe("Agents cards", () => {
  // Checks the "Two agents connected" scenario: both cards say Connected and
  // offer View command and Disconnect.
  it("shows both agents connected with View command and Disconnect", async () => {
    const api = new FakeAgentsApi({
      clientList: [client("Claude Code"), client("opencode")],
    });
    renderWithApp(<Screen api={api} />);

    await waitFor(() =>
      expect(within(card("Claude Code")).getByText("Connected")).toBeTruthy(),
    );
    for (const name of ["Claude Code", "OpenCode"]) {
      const one = within(await screen.findByRole("region", { name }));
      await one.findByText("Connected");
      expect(one.getByRole("button", { name: "View command" })).toBeTruthy();
      expect(one.getByRole("button", { name: "Disconnect" })).toBeTruthy();
    }
  });

  // Checks the "Nobody connected" scenario: both cards offer Connect.
  it("offers Connect when no agent has connected", async () => {
    renderWithApp(<Screen api={new FakeAgentsApi()} />);
    for (const name of ["Claude Code", "OpenCode"]) {
      const one = within(card(name));
      expect(one.getByText("Not connected")).toBeTruthy();
      expect(one.getByRole("button", { name: "Connect" })).toBeTruthy();
    }
  });

  // Checks that Disconnect asks the core to forget the agent, and that the
  // card goes back to Connect when the core says the list changed.
  it("Disconnect removes the agent", async () => {
    const api = new FakeAgentsApi({ clientList: [client("Claude Code")] });
    renderWithApp(<Screen api={api} />);

    const one = card("Claude Code");
    fireEvent.click(
      await within(one).findByRole("button", { name: "Disconnect" }),
    );

    await within(card("Claude Code")).findByRole("button", { name: "Connect" });
    expect(api.calls).toContain("remove Claude Code");
  });

  // Checks the waiting state: a client that asks to connect can be allowed
  // or rejected from its card (the approval dialog may have been closed).
  it("answers a waiting agent from its card", async () => {
    const api = new FakeAgentsApi({
      clientList: [client("OpenCode", "pending")],
    });
    renderWithApp(<Screen api={api} />);

    const one = within(card("OpenCode"));
    expect(await one.findByText("Waiting for approval")).toBeTruthy();
    fireEvent.click(one.getByRole("button", { name: "Allow" }));

    await within(card("OpenCode")).findByText("Connected");
    expect(api.calls).toContain("answer OpenCode true");
  });

  // Checks that a stopped agent can be let continue from its card.
  it("lets a stopped agent continue", async () => {
    const api = new FakeAgentsApi({
      clientList: [client("Claude Code", "allowed", true)],
    });
    renderWithApp(<Screen api={api} />);

    fireEvent.click(
      await screen.findByRole("button", { name: "Let it continue" }),
    );
    await waitFor(() => expect(api.calls).toContain("resume Claude Code"));
  });

  // Checks the Spanish texts of the card states.
  it("is translated to Spanish", async () => {
    renderWithApp(<Screen api={new FakeAgentsApi()} />, "es");
    expect(await screen.findAllByText("No conectado")).toHaveLength(2);
  });
});

describe("Agent card states", () => {
  // Checks the lost state of the design: "Connection lost" with Reconnect.
  // The core does not report it yet, so the card is drawn from its props.
  it("draws the lost connection state with Reconnect", () => {
    const [claude] = cardsFor([client("Claude Code")]);
    const onConnect = vi.fn();
    renderWithApp(
      <AgentCard
        card={{ ...claude, state: "lost" }}
        onConnect={onConnect}
        onDisconnect={() => {}}
        onAnswer={() => {}}
        onResume={() => {}}
      />,
    );
    expect(screen.getByText("Connection lost")).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "Reconnect" }));
    expect(onConnect).toHaveBeenCalled();
  });

  // Checks how clients are matched to cards: by name without case, spaces or
  // dashes, and a rejected client is not connected.
  it("matches clients to cards by a loose name", () => {
    const cards = cardsFor([
      client("claude-code"),
      client("OpenCode", "rejected"),
    ]);
    expect(cards.map((c) => c.state)).toEqual(["connected", "notConnected"]);
  });
});
