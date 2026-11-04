// @vitest-environment jsdom
import { cleanup, screen } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import { AgentsIndicator } from "./AgentsIndicator";
import { renderWithApp } from "./test-utils";
import { client } from "./testing";

afterEach(cleanup);

// The chip is the element that holds the text; its first child is the dot.
const dotOf = (text: string) =>
  screen.getByText(text).firstElementChild?.className ?? "";

describe("Top bar indicator", () => {
  // Checks "No agents" with a neutral dot when nobody is connected, and that
  // a rejected agent does not count as connected.
  it("says No agents when nobody is connected", () => {
    renderWithApp(
      <AgentsIndicator clients={[client("OpenCode", "rejected")]} />,
    );
    expect(dotOf("No agents")).toContain("bg-text-faint");
  });

  // Checks that the names of the connected agents are listed, with a green dot.
  it("lists the connected agents", () => {
    renderWithApp(
      <AgentsIndicator clients={[client("claude-code"), client("OpenCode")]} />,
    );
    expect(dotOf("Claude Code · OpenCode")).toContain("bg-success");
  });

  // Checks that one connected agent shows only its name.
  it("lists one connected agent", () => {
    renderWithApp(<AgentsIndicator clients={[client("OpenCode")]} />);
    expect(screen.getByText("OpenCode")).toBeTruthy();
  });

  // Checks "Waiting for approval" with an amber dot, and that it wins over
  // the names of the agents that are already connected.
  it("says Waiting for approval while an agent waits", () => {
    renderWithApp(
      <AgentsIndicator
        clients={[client("Claude Code"), client("OpenCode", "pending")]}
      />,
    );
    expect(dotOf("Waiting for approval")).toContain("bg-amber");
    expect(screen.queryByText("Claude Code")).toBeNull();
  });

  // Checks the Spanish text.
  it("is translated to Spanish", () => {
    renderWithApp(<AgentsIndicator clients={[]} />, "es");
    expect(screen.getByText("Sin agentes")).toBeTruthy();
  });
});
