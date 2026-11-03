// @vitest-environment jsdom
import { cleanup, fireEvent, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import { AgentsScreen } from "./AgentsScreen";
import { renderWithApp } from "./test-utils";
import { client, FakeAgentsApi } from "./testing";
import { useAgents } from "./useAgents";

afterEach(cleanup);

function Screen({ api }: { api: FakeAgentsApi }) {
  const state = useAgents(api);
  return <AgentsScreen api={api} state={state} />;
}

describe("Permissions panel", () => {
  // Checks the "Two agents connected" scenario: with agents connected the
  // panel shows the six switches in the state the core has (delete is off
  // by default) and the note about undo.
  it("shows the switches when an agent is connected", async () => {
    const api = new FakeAgentsApi({ clientList: [client("Claude Code")] });
    renderWithApp(<Screen api={api} />);

    const read = async (name: string) =>
      (await screen.findByRole("switch", { name })).getAttribute(
        "aria-checked",
      );
    expect(await read("Create sprites")).toBe("true");
    expect(await read("Edit pixels")).toBe("true");
    expect(await read("Delete sprites")).toBe("false");
    expect(await read("Export files")).toBe("true");
    expect(await read("Open projects")).toBe("false");
    expect(await read("Ask me when a new agent connects")).toBe("true");
    expect(
      screen.getByText("AI changes can be undone, except exported files."),
    ).toBeTruthy();
  });

  // Checks the "Nobody connected" scenario: the panel is hidden. It also
  // stays hidden while an agent only waits for approval.
  it("is hidden when no agent is connected", async () => {
    const api = new FakeAgentsApi({
      clientList: [client("OpenCode", "pending")],
    });
    renderWithApp(<Screen api={api} />);
    await screen.findByText("Waiting for approval");
    // Let the permissions be read too, so the panel is hidden by the rule
    // and not because it has not loaded.
    await waitFor(() => expect(api.listening).toBe(true));
    expect(screen.queryByText("Permissions")).toBeNull();
  });

  // Checks the "Turn delete on" scenario: pressing the switch sends the whole
  // new set of permissions to the core at once (the core then applies it to
  // the next call), and the switch follows the answer of the core.
  it("sends a change at once", async () => {
    const api = new FakeAgentsApi({ clientList: [client("Claude Code")] });
    renderWithApp(<Screen api={api} />);

    fireEvent.click(
      await screen.findByRole("switch", { name: "Delete sprites" }),
    );

    await waitFor(() =>
      expect(api.calls).toContain(
        `setPermissions ${JSON.stringify({ create: true, edit: true, delete: true, export: true, openProject: false, askOnConnect: true })}`,
      ),
    );
    await waitFor(() =>
      expect(
        screen
          .getByRole("switch", { name: "Delete sprites" })
          .getAttribute("aria-checked"),
      ).toBe("true"),
    );
  });

  // Checks the "Turn it on" scenario of Open projects: the switch is off by
  // default and sends the change at once, like the others.
  it("turns Open projects on", async () => {
    const api = new FakeAgentsApi({ clientList: [client("Claude Code")] });
    renderWithApp(<Screen api={api} />);

    fireEvent.click(
      await screen.findByRole("switch", { name: "Open projects" }),
    );

    await waitFor(() =>
      expect(
        screen
          .getByRole("switch", { name: "Open projects" })
          .getAttribute("aria-checked"),
      ).toBe("true"),
    );
    expect(api.permissionValues.openProject).toBe(true);
  });

  // Checks that a failed save is said, and the switch does not move.
  it("says when the permissions could not be saved", async () => {
    const api = new FakeAgentsApi({ clientList: [client("Claude Code")] });
    api.setPermissions = async () => {
      throw new Error("disk full");
    };
    renderWithApp(<Screen api={api} />);

    fireEvent.click(
      await screen.findByRole("switch", { name: "Export files" }),
    );

    expect(await screen.findByRole("alert")).toBeTruthy();
    expect(
      screen
        .getByRole("switch", { name: "Export files" })
        .getAttribute("aria-checked"),
    ).toBe("true");
  });

  // Checks the Spanish texts.
  it("is translated to Spanish", async () => {
    const api = new FakeAgentsApi({ clientList: [client("Claude Code")] });
    renderWithApp(<Screen api={api} />, "es");
    expect(
      await screen.findByRole("switch", { name: "Borrar sprites" }),
    ).toBeTruthy();
    expect(
      screen.getByRole("switch", { name: "Abrir proyectos" }),
    ).toBeTruthy();
  });
});
