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
import { AgentsScreen } from "./AgentsScreen";
import { CONNECT_TIMEOUT_MS, ConnectDialog } from "./ConnectDialog";
import { cardsFor } from "./clients";
import { renderWithApp } from "./test-utils";
import { client, FakeAgentsApi } from "./testing";
import { useAgents } from "./useAgents";

afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
});

function Screen({ api }: { api: FakeAgentsApi }) {
  const state = useAgents(api);
  return <AgentsScreen api={api} state={state} />;
}

// jsdom has no clipboard: this one remembers what was written.
function stubClipboard(fail = false) {
  const written: string[] = [];
  vi.stubGlobal("navigator", {
    ...navigator,
    clipboard: {
      writeText: async (text: string) => {
        if (fail) throw new Error("denied");
        written.push(text);
      },
    },
  });
  return written;
}

const openDialog = async (api: FakeAgentsApi) => {
  renderWithApp(<Screen api={api} />);
  const card = await screen.findByRole("region", { name: "Claude Code" });
  fireEvent.click(await within(card).findByRole("button", { name: "Connect" }));
  return screen.findByRole("dialog", { name: "Connect an AI" });
};

describe("Connect dialog", () => {
  // Checks the "Copy the command" scenario: Copy puts `sprixel mcp add
  // claude` on the clipboard (no token in it) and the dialog keeps waiting.
  it("copies the command and keeps waiting", async () => {
    const written = stubClipboard();
    const dialog = within(await openDialog(new FakeAgentsApi()));

    expect(dialog.getByText("sprixel mcp add claude")).toBeTruthy();
    fireEvent.click(dialog.getByRole("button", { name: "Copy" }));

    await dialog.findByRole("button", { name: "Copied" });
    expect(written).toEqual(["sprixel mcp add claude"]);
    expect(dialog.getByRole("status").textContent).toContain(
      "Waiting for connection...",
    );
  });

  // Checks that the command follows the tab: OpenCode has its own command.
  it("shows the OpenCode command on its tab", async () => {
    const written = stubClipboard();
    const dialog = within(await openDialog(new FakeAgentsApi()));

    fireEvent.click(dialog.getByRole("tab", { name: "OpenCode" }));
    fireEvent.click(dialog.getByRole("button", { name: "Copy" }));

    await waitFor(() => expect(written).toEqual(["sprixel mcp add opencode"]));
    expect(dialog.getByText("sprixel mcp add opencode")).toBeTruthy();
  });

  // Checks that a clipboard that refuses is said, with a way out.
  it("says when it could not copy", async () => {
    stubClipboard(true);
    const dialog = within(await openDialog(new FakeAgentsApi()));
    fireEvent.click(dialog.getByRole("button", { name: "Copy" }));
    expect((await dialog.findByRole("alert")).textContent).toContain(
      "copy it by hand",
    );
  });

  // Checks that the waiting state turns into "Connected" when the agent
  // connects, with no action of the person.
  it("turns into Connected when the agent connects", async () => {
    const api = new FakeAgentsApi();
    const dialog = within(await openDialog(api));
    await waitFor(() => expect(api.listening).toBe(true));

    act(() => {
      api.clientList = [client("Claude Code")];
      api.emit({ type: "clients" });
    });

    await waitFor(() =>
      expect(dialog.getByRole("status").textContent).toContain("Connected"),
    );
    // The cross of the dialog and the button at the bottom.
    expect(dialog.getAllByRole("button", { name: "Close" })).toHaveLength(2);
  });

  // Checks that Cancel closes the dialog.
  it("closes with Cancel", async () => {
    await openDialog(new FakeAgentsApi());
    fireEvent.click(screen.getByRole("button", { name: "Cancel" }));
    expect(screen.queryByRole("dialog")).toBeNull();
  });
});

describe("Connect dialog: it takes too long", () => {
  // Checks the "It takes too long" scenario. The real wait is 60 seconds
  // (the constant is checked); the test uses a short one so it does not
  // have to wait.
  it("suggests sprixel doctor after the timeout", async () => {
    expect(CONNECT_TIMEOUT_MS).toBe(60_000);
    renderWithApp(
      <ConnectDialog
        initial="claude"
        cards={cardsFor([])}
        onClose={() => {}}
        timeoutMs={20}
      />,
    );
    expect(screen.getByText(/Not connecting\?/)).toBeTruthy();
    expect(await screen.findByText(/Run sprixel doctor/)).toBeTruthy();
  });

  // Checks that a connected agent gets no help message, even after the
  // timeout. Fake timers move the clock without waiting.
  it("gives no help to a connected agent", () => {
    vi.useFakeTimers();
    try {
      renderWithApp(
        <ConnectDialog
          initial="claude"
          cards={cardsFor([client("Claude Code")])}
          onClose={() => {}}
          timeoutMs={20}
        />,
      );
      act(() => {
        vi.advanceTimersByTime(100);
      });
      expect(screen.getByRole("status").textContent).toContain("Connected");
      expect(screen.queryByText(/sprixel doctor/)).toBeNull();
    } finally {
      vi.useRealTimers();
    }
  });
});

describe("Connect dialog: size", () => {
  // Checks the "Switch tabs" scenario. jsdom has no layout, so this checks
  // what keeps the size: the same dialog element (it is not rebuilt), a body
  // with a fixed height, and the same fixed parts on both tabs.
  it("keeps its size when the tab changes", async () => {
    const dialog = await openDialog(new FakeAgentsApi());
    const body = dialog.querySelector("div.h-\\[440px\\]");
    expect(body).toBeTruthy();
    const before = dialog.className;

    fireEvent.click(within(dialog).getByRole("tab", { name: "OpenCode" }));

    expect(screen.getByRole("dialog", { name: "Connect an AI" })).toBe(dialog);
    expect(dialog.className).toBe(before);
    expect(dialog.querySelector("div.h-\\[440px\\]")).toBe(body);
    expect(
      within(dialog)
        .getByRole("tab", { name: "OpenCode" })
        .getAttribute("aria-selected"),
    ).toBe("true");
  });

  // Checks that the install note is there: a person without the command
  // knows where to look.
  it("says how to get the sprixel command", async () => {
    const dialog = within(await openDialog(new FakeAgentsApi()));
    expect(dialog.getByText(/Install section of the README/)).toBeTruthy();
  });
});
