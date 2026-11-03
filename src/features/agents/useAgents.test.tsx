// @vitest-environment jsdom
import { act, cleanup, renderHook, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import { client, FakeAgentsApi, row } from "./testing";
import { useAgents } from "./useAgents";

afterEach(cleanup);

async function start(
  api: FakeAgentsApi,
  onEvent?: Parameters<typeof useAgents>[1],
) {
  const view = renderHook(() => useAgents(api, onEvent));
  // The hook listens to the events once it is mounted.
  await waitFor(() => expect(api.listening).toBe(true));
  return view;
}

describe("useAgents", () => {
  // Checks that the first state is read from the core: the clients, the
  // permissions, the activity and the agent that is already writing.
  it("reads the state of the core when it starts", async () => {
    const api = new FakeAgentsApi({
      clientList: [client("Claude Code")],
      rows: [row(1)],
    });
    api.current = {
      agent: "Claude Code",
      sprite: { folder: [], name: "goblin" },
      frame: 2,
    };
    const { result } = await start(api);

    await waitFor(() => expect(result.current.clients).toHaveLength(1));
    await waitFor(() => expect(result.current.activity).toHaveLength(1));
    await waitFor(() => expect(result.current.permissions?.delete).toBe(false));
    await waitFor(() => expect(result.current.writing?.frame).toBe(2));
  });

  // Checks a simulated stream of events, in the order the core sends them:
  // a client asks to connect, the user allows it, the agent draws, the
  // activity grows, the permissions change and the agent ends.
  it("follows a stream of core events", async () => {
    const api = new FakeAgentsApi();
    const seen: string[] = [];
    const { result } = await start(api, (event) => seen.push(event.type));

    act(() => {
      api.clientList = [client("OpenCode", "pending")];
      api.emit({ type: "pending", name: "OpenCode", version: "1.0" });
      api.emit({ type: "clients" });
    });
    await waitFor(() =>
      expect(result.current.clients[0]?.state).toBe("pending"),
    );

    await act(() => api.answer("OpenCode", "1.0", true));
    await waitFor(() =>
      expect(result.current.clients[0]?.state).toBe("allowed"),
    );

    act(() => {
      api.emit({
        type: "writing",
        write: {
          agent: "OpenCode",
          sprite: { folder: [], name: "slime" },
          frame: 1,
        },
      });
    });
    await waitFor(() => expect(result.current.writing?.agent).toBe("OpenCode"));

    act(() => {
      api.rows = [row(2, { client: "OpenCode" })];
      api.emit({ type: "activity" });
    });
    await waitFor(() => expect(result.current.activity[0]?.id).toBe(2));

    act(() => {
      api.emit({
        type: "permissions",
        permissions: { ...api.permissionValues, delete: true },
      });
    });
    await waitFor(() => expect(result.current.permissions?.delete).toBe(true));

    act(() => api.emit({ type: "writing", write: null }));
    await waitFor(() => expect(result.current.writing).toBeNull());
    // The owner of the hook also sees every event (for notifications).
    expect(seen).toContain("pending");
  });

  // Checks that the hook stops listening when it goes away, so a closed
  // screen does not keep a listener.
  it("stops listening when it unmounts", async () => {
    const api = new FakeAgentsApi();
    const { unmount } = await start(api);
    unmount();
    expect(api.listening).toBe(false);
  });
});
