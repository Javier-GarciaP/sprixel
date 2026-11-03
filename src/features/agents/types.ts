// What the Rust core tells the Agents screens. The names are the ones of the
// Rust structs (see src-tauri/src/mcp/ and src-tauri/src/ipc/mcp.rs).

import type { FrameData } from "@/shared/lib/core";

// `allowed` is a connected agent. `pending` waits for the user to answer.
export type ClientState = "allowed" | "rejected" | "pending";

export type AgentClient = {
  name: string;
  version: string;
  state: ClientState;
  // The user pressed Stop and has not let the agent continue.
  stopped: boolean;
};

export type Permissions = {
  create: boolean;
  edit: boolean;
  delete: boolean;
  export: boolean;
  askOnConnect: boolean;
};

// One thing an agent did. `sprite` is a path like "Characters/goblin".
export type ActivityEntry = {
  id: number;
  at_ms: number;
  client: string;
  tool: string;
  summary: string;
  sprite: string | null;
  result: "ok" | "error";
  error: string | null;
  revision: number | null;
};

// `can_undo` is true only for the latest AI action that is still the latest
// change of the project.
export type ActivityRow = ActivityEntry & { can_undo: boolean };

// An agent writing to a frame right now.
export type AgentWrite = {
  agent: string;
  sprite: { folder: string[]; name: string };
  frame: number;
};

// What the core sends while the app runs. The UI only shows it.
export type AgentsEvent =
  | { type: "clients" }
  | { type: "pending"; name: string; version: string }
  | { type: "permissions"; permissions: Permissions }
  | { type: "activity" }
  | { type: "writing"; write: AgentWrite | null }
  | { type: "open-sprite"; sprite: { folder: string[]; name: string } };

export type AgentsApi = {
  clients(): Promise<AgentClient[]>;
  permissions(): Promise<Permissions>;
  setPermissions(permissions: Permissions): Promise<void>;
  activity(): Promise<ActivityRow[]>;
  writing(): Promise<AgentWrite | null>;
  // Allow (true) or reject (false) a client that waits.
  answer(name: string, version: string, allow: boolean): Promise<boolean>;
  remove(name: string, version: string): Promise<boolean>;
  stop(name: string): Promise<void>;
  resume(name: string): Promise<void>;
  // Undoes the latest change of the project (the row with Undo is that one).
  undo(): Promise<void>;
  frame(folder: string[], name: string): Promise<FrameData>;
  // Calls back for every event. Returns a function that stops listening.
  onEvent(listener: (event: AgentsEvent) => void): Promise<() => void>;
};
