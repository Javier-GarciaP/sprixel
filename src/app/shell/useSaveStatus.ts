import { useCallback, useEffect, useState } from "react";
import { projectStatus, retrySave } from "../api/project";
import type { SaveState } from "./SaveStatus";

// Reads the save status of the open project from the core every half second,
// but only while a project is open. The polling stops when the project closes
// or the component goes away.
export function useSaveStatus(active: boolean, intervalMs = 500) {
  const [state, setState] = useState<SaveState>({ status: "saved" });

  useEffect(() => {
    if (!active) return;
    let stopped = false;
    const read = () =>
      projectStatus()
        .then((status) => {
          if (!stopped && status) setState(status);
        })
        .catch(() => {});
    read();
    const timer = setInterval(read, intervalMs);
    return () => {
      stopped = true;
      clearInterval(timer);
    };
  }, [active, intervalMs]);

  // The "Retry" action: the core saves again and answers with the new status.
  const retry = useCallback(() => {
    retrySave()
      .then((status) => status && setState(status))
      .catch(() => {});
  }, []);

  return { state, retry };
}
