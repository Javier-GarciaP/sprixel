import { useCallback, useEffect, useRef, useState } from "react";
import type {
  ActivityRow,
  AgentClient,
  AgentsApi,
  AgentsEvent,
  AgentWrite,
  Permissions,
} from "./types";

export type AgentsState = {
  clients: AgentClient[];
  // `null` until the core answers (and outside the app).
  permissions: Permissions | null;
  activity: ActivityRow[];
  // The agent that writes to a frame now, if any.
  writing: AgentWrite | null;
};

// Follows the events of the core. The core owns the data; this keeps only the
// last answer to show it. Every event asks the core again, and an answer that
// arrives late (after a newer one) is dropped.
export function useAgents(
  api: AgentsApi,
  onEvent?: (event: AgentsEvent) => void,
): AgentsState {
  const [clients, setClients] = useState<AgentClient[]>([]);
  const [permissions, setPermissions] = useState<Permissions | null>(null);
  const [activity, setActivity] = useState<ActivityRow[]>([]);
  const [writing, setWriting] = useState<AgentWrite | null>(null);
  const handler = useRef(onEvent);
  useEffect(() => {
    handler.current = onEvent;
  });

  const refreshClients = useLatest(api, loadClients, setClients);
  const refreshActivity = useLatest(api, loadActivity, setActivity);

  useEffect(() => {
    let stopped = false;
    let unlisten: (() => void) | undefined;
    // The events first, so none is lost while the state is being read.
    api
      .onEvent((event) => {
        if (stopped) return;
        if (event.type === "clients") refreshClients();
        else if (event.type === "permissions")
          setPermissions(event.permissions);
        else if (event.type === "activity") refreshActivity();
        else if (event.type === "writing") setWriting(event.write);
        handler.current?.(event);
      })
      .then((stop) => {
        if (stopped) stop();
        else unlisten = stop;
      })
      .catch(() => {});
    refreshClients();
    refreshActivity();
    api
      .permissions()
      .then((next) => !stopped && setPermissions((now) => now ?? next))
      .catch(() => {});
    api
      .writing()
      .then((now) => !stopped && now && setWriting((was) => was ?? now))
      .catch(() => {});
    return () => {
      stopped = true;
      unlisten?.();
    };
  }, [api, refreshClients, refreshActivity]);

  return { clients, permissions, activity, writing };
}

const loadClients = (api: AgentsApi) => api.clients();
const loadActivity = (api: AgentsApi) => api.activity();

// Asks the core and shows the answer, unless a newer ask was made meanwhile.
function useLatest<T>(
  api: AgentsApi,
  load: (api: AgentsApi) => Promise<T>,
  show: (value: T) => void,
) {
  const last = useRef(0);
  return useCallback(() => {
    const mine = ++last.current;
    load(api)
      .then((value) => mine === last.current && show(value))
      .catch(() => {});
  }, [api, load, show]);
}
