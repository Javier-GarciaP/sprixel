// @vitest-environment jsdom
import {
  act,
  cleanup,
  fireEvent,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { AgentsIndicator } from "./AgentsIndicator";
import { AgentsScreen } from "./AgentsScreen";
import { ApprovalDialog } from "./ApprovalDialog";
import { notifyApproval } from "./notify";
import { renderWithApp } from "./test-utils";
import { client, FakeAgentsApi } from "./testing";
import { useAgents } from "./useAgents";

afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
  vi.restoreAllMocks();
});

// What the app shell does: the indicator of the top bar, the Agents tab and
// the global approval prompt, all fed by the same state.
function App({ api, tab = true }: { api: FakeAgentsApi; tab?: boolean }) {
  const state = useAgents(api);
  return (
    <>
      <header data-testid="top-bar">
        <AgentsIndicator clients={state.clients} />
      </header>
      {tab && <AgentsScreen api={api} state={state} />}
      <ApprovalDialog
        api={api}
        clients={state.clients}
        permissions={state.permissions}
      />
    </>
  );
}

const asks = (api: FakeAgentsApi, name = "Claude Code") => {
  api.clientList = [...api.clientList, client(name, "pending")];
  api.emit({ type: "pending", name, version: "1.0" });
  api.emit({ type: "clients" });
};

const start = async (api: FakeAgentsApi, tab = true) => {
  renderWithApp(<App api={api} tab={tab} />);
  await waitFor(() => expect(api.listening).toBe(true));
};

describe("Approval prompt", () => {
  // Checks the prompt content: the name of the agent, the kind of
  // connection, what it could do and what it could not, and the two buttons.
  // It also checks that the top bar waits for the answer.
  it("shows who connects, what it may do and Waiting for approval", async () => {
    const api = new FakeAgentsApi();
    await start(api, false);
    act(() => asks(api));

    const dialog = within(
      await screen.findByRole("dialog", {
        name: "Claude Code wants to connect",
      }),
    );
    expect(dialog.getByText("Local · this computer")).toBeTruthy();
    expect(
      await dialog.findByText("Create sprites, Edit pixels, Export files"),
    ).toBeTruthy();
    expect(dialog.getByText("Delete sprites")).toBeTruthy();
    expect(dialog.getByRole("button", { name: "Allow" })).toBeTruthy();
    expect(dialog.getByRole("button", { name: "Reject" })).toBeTruthy();
    expect(
      within(screen.getByTestId("top-bar")).getByText("Waiting for approval"),
    ).toBeTruthy();
  });

  // Checks the "Allow" scenario: the core is told, the card shows Connected
  // and the top bar shows the agent as connected.
  it("Allow connects the agent", async () => {
    const api = new FakeAgentsApi();
    await start(api);
    act(() => asks(api));

    // The card also has Allow: the one of the dialog is the one pressed.
    const dialog = await screen.findByRole("dialog");
    fireEvent.click(within(dialog).getByRole("button", { name: "Allow" }));

    await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
    expect(api.calls).toContain("answer Claude Code true");
    const card = within(screen.getByRole("region", { name: "Claude Code" }));
    expect(card.getByText("Connected")).toBeTruthy();
    expect(
      within(screen.getByTestId("top-bar")).getByText("Claude Code"),
    ).toBeTruthy();
  });

  // Checks Reject: the core is told, the dialog closes and the card goes
  // back to "Not connected".
  it("Reject keeps the agent out", async () => {
    const api = new FakeAgentsApi();
    await start(api);
    act(() => asks(api, "OpenCode"));

    fireEvent.click(
      within(await screen.findByRole("dialog")).getByRole("button", {
        name: "Reject",
      }),
    );

    await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
    expect(api.calls).toContain("answer OpenCode false");
    expect(
      within(screen.getByRole("region", { name: "OpenCode" })).getByText(
        "Not connected",
      ),
    ).toBeTruthy();
    expect(
      within(screen.getByTestId("top-bar")).getByText("No agents"),
    ).toBeTruthy();
  });

  // Checks the risk of "closed by accident": closing the dialog does not
  // answer. The agent still waits, the top bar still says so, and the card
  // can answer.
  it("keeps the request when the dialog is closed", async () => {
    const api = new FakeAgentsApi();
    await start(api);
    act(() => asks(api));

    fireEvent.click(
      within(await screen.findByRole("dialog")).getAllByRole("button", {
        name: "Close",
      })[0],
    );

    await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
    expect(api.calls).toEqual([]);
    expect(
      within(screen.getByTestId("top-bar")).getByText("Waiting for approval"),
    ).toBeTruthy();
    const card = within(screen.getByRole("region", { name: "Claude Code" }));
    fireEvent.click(card.getByRole("button", { name: "Allow" }));
    await waitFor(() => expect(api.calls).toContain("answer Claude Code true"));
  });

  // Checks the safety rule: a name with HTML is shown as text, no element is
  // made from it.
  it("shows the name of the agent as plain text", async () => {
    const api = new FakeAgentsApi();
    await start(api, false);
    const evil = '<img src=x onerror="alert(1)">';
    act(() => asks(api, evil));

    const dialog = await screen.findByRole("dialog");
    expect(dialog.textContent).toContain(evil);
    expect(dialog.querySelector("img")).toBeNull();
    expect(document.body.querySelector("img")).toBeNull();
  });

  // Checks that the prompt is global: it shows with no Agents tab at all
  // (the app shell mounts it on every tab).
  it("shows on any tab", async () => {
    const api = new FakeAgentsApi();
    await start(api, false);
    act(() => asks(api));
    expect(await screen.findByRole("dialog")).toBeTruthy();
  });

  // Checks the Spanish texts.
  it("is translated to Spanish", async () => {
    const api = new FakeAgentsApi({
      clientList: [client("OpenCode", "pending")],
    });
    renderWithApp(<App api={api} tab={false} />, "es");
    expect(
      await screen.findByRole("dialog", { name: "OpenCode quiere conectarse" }),
    ).toBeTruthy();
  });
});

describe("System notification", () => {
  const stub = (permission: string, focused: boolean) => {
    const shown: string[] = [];
    class Fake {
      static permission = permission;
      static requestPermission = vi.fn(async () => "granted");
      constructor(title: string) {
        shown.push(title);
      }
    }
    vi.stubGlobal("Notification", Fake);
    vi.spyOn(document, "hasFocus").mockReturnValue(focused);
    return { shown, Fake };
  };

  // Checks that a window in the background gets a notification.
  it("notifies when the window is not focused", () => {
    const { shown } = stub("granted", false);
    notifyApproval("Claude Code wants to connect", "Open Sprixel");
    expect(shown).toEqual(["Claude Code wants to connect"]);
  });

  // Checks that a focused window gets none: the person already sees the prompt.
  it("does not notify when the window is focused", () => {
    const { shown } = stub("granted", true);
    notifyApproval("x", "y");
    expect(shown).toEqual([]);
  });

  // Checks that the first time the system is asked for permission.
  it("asks for permission the first time", async () => {
    const { shown, Fake } = stub("default", false);
    notifyApproval("x", "y");
    await waitFor(() => expect(shown).toEqual(["x"]));
    expect(Fake.requestPermission).toHaveBeenCalled();
  });

  // Checks that a system without notifications does not break the app.
  it("does nothing where notifications do not exist", () => {
    vi.stubGlobal("Notification", undefined);
    expect(() => notifyApproval("x", "y")).not.toThrow();
  });
});
