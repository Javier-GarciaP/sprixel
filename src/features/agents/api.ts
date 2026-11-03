// Calls to the Rust core for the Agents screens. The core owns the clients,
// the permissions and the activity; these functions only ask and show.
// Outside the app (`npm run dev`) there is no core: calls fail and no event
// is ever sent.

import { projectUndo, spriteFrame } from "@/shared/lib/core";
import type {
  ActivityRow,
  AgentClient,
  AgentsApi,
  AgentWrite,
  Permissions,
} from "./types";

const insideApp = () => "__TAURI_INTERNALS__" in window;

async function call<T>(
  command: string,
  args?: Record<string, unknown>,
): Promise<T> {
  if (!insideApp()) {
    throw { code: "no_core", message: "Not running inside the Sprixel app." };
  }
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<T>(command, args);
}

export const coreAgentsApi: AgentsApi = {
  clients: () => call<AgentClient[]>("mcp_clients"),
  permissions: () => call<Permissions>("mcp_permissions"),
  setPermissions: (permissions) => call("mcp_set_permissions", { permissions }),
  activity: () => call<ActivityRow[]>("mcp_activity"),
  writing: () => call<AgentWrite | null>("agent_activity"),
  answer: (name, version, allow) =>
    call<boolean>("mcp_answer_client", { name, version, allow }),
  remove: (name, version) =>
    call<boolean>("mcp_remove_client", { name, version }),
  stop: (name) => call("mcp_stop_agent", { name }),
  resume: (name) => call("mcp_resume_agent", { name }),
  undo: projectUndo,
  frame: (folder, name) => spriteFrame(folder, name, 1),
  async onEvent(listener) {
    if (!insideApp()) return () => {};
    const { listen } = await import("@tauri-apps/api/event");
    const stops = await Promise.all([
      listen("mcp://clients-changed", () => listener({ type: "clients" })),
      listen<{ name: string; version: string }>(
        "mcp://client-pending",
        (event) => listener({ type: "pending", ...event.payload }),
      ),
      listen<Permissions>("mcp://permissions-changed", (event) =>
        listener({ type: "permissions", permissions: event.payload }),
      ),
      listen("mcp://activity", () => listener({ type: "activity" })),
      // An undo or any other change can take Undo away from the latest row.
      listen("project://changed", () => listener({ type: "activity" })),
      listen<AgentWrite | null>("agent-activity", (event) =>
        listener({ type: "writing", write: event.payload }),
      ),
      listen<{ folder: string[]; name: string }>("mcp://open-sprite", (event) =>
        listener({ type: "open-sprite", sprite: event.payload }),
      ),
    ]);
    return () => stops.forEach((stop) => stop());
  },
};
