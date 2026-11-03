// @vitest-environment jsdom
import {
  cleanup,
  fireEvent,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import { act } from "react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { AgentsScreen } from "./AgentsScreen";
import { renderWithApp } from "./test-utils";
import { client, FakeAgentsApi, row } from "./testing";
import { useAgents } from "./useAgents";

// jsdom has no canvas drawing: the thumbnails stay empty.
beforeEach(() => {
  vi.spyOn(HTMLCanvasElement.prototype, "getContext").mockReturnValue(null);
});
afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
});

function Screen({ api }: { api: FakeAgentsApi }) {
  const state = useAgents(api);
  return <AgentsScreen api={api} state={state} />;
}

const drawing = {
  agent: "Claude Code",
  sprite: { folder: [], name: "goblin" },
  frame: 2,
};

describe("Activity list", () => {
  // Checks the "In progress" scenario: while an agent draws frame 2, the
  // first row is highlighted and has Stop; Stop asks the core to stop that
  // agent by name.
  it("highlights the action in progress and offers Stop", async () => {
    const api = new FakeAgentsApi({
      clientList: [client("Claude Code")],
      rows: [row(1)],
    });
    renderWithApp(<Screen api={api} />);
    await screen.findByText("Created the sprite goblin");
    await waitFor(() => expect(api.listening).toBe(true));

    act(() => api.emit({ type: "writing", write: drawing }));

    const first = within(await screen.findByTestId("in-progress"));
    expect(first.getByText("Drawing frame 2 of goblin")).toBeTruthy();
    // It is the first row of the list.
    expect(screen.getAllByRole("listitem")[0]).toBe(
      screen.getByTestId("in-progress"),
    );
    fireEvent.click(first.getByRole("button", { name: "Stop" }));
    await waitFor(() => expect(api.calls).toContain("stop Claude Code"));

    // The row goes away when the work ends.
    act(() => api.emit({ type: "writing", write: null }));
    await waitFor(() => expect(screen.queryByTestId("in-progress")).toBeNull());
  });

  // Checks the "latest only" rule: with three finished actions only the one
  // the core marks (can_undo) has Undo, and pressing it undoes the project.
  it("offers Undo only on the latest finished action", async () => {
    const api = new FakeAgentsApi({
      rows: [
        row(3, { tool: "apply_grid", can_undo: true }),
        row(2, { tool: "set_pixels", summary: "x", can_undo: false }),
        row(1),
      ],
    });
    renderWithApp(<Screen api={api} />);

    const undo = await screen.findAllByRole("button", { name: "Undo" });
    expect(undo).toHaveLength(1);
    const first = screen.getAllByRole("listitem")[0];
    expect(
      within(first).getByText("Applied a text grid to goblin"),
    ).toBeTruthy();
    expect(within(first).getByRole("button", { name: "Undo" })).toBe(undo[0]);

    fireEvent.click(undo[0]);
    await waitFor(() => expect(api.calls).toContain("undo"));
  });

  // Checks that the automatic outline and shading have their own text.
  it("names the outline and shade actions", async () => {
    const api = new FakeAgentsApi({
      rows: [row(2, { tool: "auto_shade" }), row(1, { tool: "auto_outline" })],
    });
    renderWithApp(<Screen api={api} />);

    await screen.findByText("Shaded goblin");
    expect(screen.getByText("Outlined goblin")).toBeTruthy();
  });

  // Checks that Undo moves with the core: when a newer action arrives, the
  // button leaves the old row.
  it("moves Undo to a newer action", async () => {
    const api = new FakeAgentsApi({ rows: [row(1, { can_undo: true })] });
    renderWithApp(<Screen api={api} />);
    await screen.findByRole("button", { name: "Undo" });
    await waitFor(() => expect(api.listening).toBe(true));

    act(() => {
      api.rows = [
        row(2, { tool: "draw_shape", can_undo: true }),
        row(1, { can_undo: false }),
      ];
      api.emit({ type: "activity" });
    });

    await screen.findByText("Drew a shape on goblin");
    const items = screen.getAllByRole("listitem");
    await waitFor(() =>
      expect(
        within(items[1]).queryByRole("button", { name: "Undo" }),
      ).toBeNull(),
    );
    expect(within(items[0]).getByRole("button", { name: "Undo" })).toBeTruthy();
  });

  // Checks what each row shows: the client, the time, and a failed call
  // (which has no Undo) with its code.
  it("shows the client, the time and failures", async () => {
    const api = new FakeAgentsApi({
      rows: [
        row(2, {
          tool: "delete_sprite",
          result: "error",
          error: "permission_denied",
        }),
        row(1, { client: "OpenCode" }),
      ],
    });
    renderWithApp(<Screen api={api} />);

    const items = await screen.findAllByRole("listitem");
    expect(
      within(items[0]).getByText(
        "Could not run delete_sprite (permission_denied)",
      ),
    ).toBeTruthy();
    expect(
      within(items[1]).getByText(/^OpenCode · \d{1,2}:\d{2}/),
    ).toBeTruthy();
  });

  // Checks the empty area for "Nobody connected" and the Spanish texts.
  it("explains that the list will show what agents draw", async () => {
    renderWithApp(<Screen api={new FakeAgentsApi()} />);
    expect(
      await screen.findByText(
        "Connect an agent and you will see here what it draws.",
      ),
    ).toBeTruthy();
    cleanup();
    renderWithApp(<Screen api={new FakeAgentsApi({ rows: [row(1)] })} />, "es");
    expect(await screen.findByText("Creó el sprite goblin")).toBeTruthy();
  });

  // Checks that a tool the UI does not know shows the text of the core, so
  // a new tool is never shown empty.
  it("falls back to the text of the core", async () => {
    const api = new FakeAgentsApi({
      rows: [row(1, { tool: "new_tool", summary: "Did something new" })],
    });
    renderWithApp(<Screen api={api} />);
    expect(await screen.findByText("Did something new")).toBeTruthy();
  });
});
