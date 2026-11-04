// A fake core for the tests of this feature. It keeps the data in memory and
// lets a test send events like the real core does.

import type { FrameData } from "@/shared/lib/core";
import type {
  ActivityRow,
  AgentClient,
  AgentsApi,
  AgentsEvent,
  AgentWrite,
  ClientState,
  Permissions,
} from "./types";

export const defaultPermissions: Permissions = {
  create: true,
  edit: true,
  delete: false,
  export: true,
  openProject: false,
  askOnConnect: true,
};

export const client = (
  name: string,
  state: ClientState = "allowed",
  stopped = false,
): AgentClient => ({ name, version: "1.0", state, stopped });

export const row = (
  id: number,
  over: Partial<ActivityRow> = {},
): ActivityRow => ({
  id,
  at_ms: Date.UTC(2026, 0, 1, 12, 41),
  client: "Claude Code",
  tool: "create_sprite",
  summary: "Created the sprite goblin",
  sprite: "goblin",
  result: "ok",
  error: null,
  revision: id,
  can_undo: false,
  ...over,
});

const emptyFrame: FrameData = {
  revision: 1,
  width: 2,
  height: 2,
  rows: ["..", ".."],
  colors: [],
};

export class FakeAgentsApi implements AgentsApi {
  clientList: AgentClient[] = [];
  permissionValues: Permissions = { ...defaultPermissions };
  rows: ActivityRow[] = [];
  current: AgentWrite | null = null;
  calls: string[] = [];
  // Tests set these to play a core that fails or has lost the request.
  answerError: string | null = null;
  answerGone = false;
  copied: string[] = [];
  private listeners = new Set<(event: AgentsEvent) => void>();

  constructor(init: Partial<Pick<FakeAgentsApi, "clientList" | "rows">> = {}) {
    Object.assign(this, init);
  }

  // The test plays the part of the core: it changes the data and sends the
  // event the core would send.
  emit(event: AgentsEvent) {
    this.listeners.forEach((listener) => listener(event));
  }

  get listening() {
    return this.listeners.size > 0;
  }

  clients = async () => this.clientList.map((c) => ({ ...c }));
  permissions = async () => ({ ...this.permissionValues });
  setPermissions = async (permissions: Permissions) => {
    this.calls.push(`setPermissions ${JSON.stringify(permissions)}`);
    this.permissionValues = permissions;
    this.emit({ type: "permissions", permissions });
  };
  activity = async () => this.rows.map((r) => ({ ...r }));
  writing = async () => this.current;
  answer = async (name: string, version: string, allow: boolean) => {
    this.calls.push(`answer ${name} ${allow}`);
    if (this.answerError) throw { code: "io", message: this.answerError };
    if (this.answerGone) {
      this.clientList = this.clientList.filter((c) => c.name !== name);
      return false;
    }
    this.clientList = this.clientList.map((c) =>
      c.name === name && c.version === version
        ? { ...c, state: allow ? "allowed" : "rejected" }
        : c,
    );
    this.emit({ type: "clients" });
    return true;
  };
  remove = async (name: string, version: string) => {
    this.calls.push(`remove ${name}`);
    this.clientList = this.clientList.filter(
      (c) => !(c.name === name && c.version === version),
    );
    this.emit({ type: "clients" });
    return true;
  };
  stop = async (name: string) => {
    this.calls.push(`stop ${name}`);
  };
  resume = async (name: string) => {
    this.calls.push(`resume ${name}`);
  };
  undo = async () => {
    this.calls.push("undo");
  };
  frame = async () => emptyFrame;
  onEvent = async (listener: (event: AgentsEvent) => void) => {
    this.listeners.add(listener);
    return () => {
      this.listeners.delete(listener);
    };
  };
}
