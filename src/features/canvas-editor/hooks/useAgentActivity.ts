import { useEffect, useState } from "react";
import type { AgentActivity, EditorApi, SpriteRef } from "../api/types";

// Which AI agent is writing now, from the events of the core. It reads the
// state once when the editor opens (an agent may already be at work) and then
// follows the events. `null` means that no agent is writing.
export function useAgentActivity(api: EditorApi) {
  const [activity, setActivity] = useState<AgentActivity | null>(null);
  // How many times an agent ended its work. A change of this number tells
  // the editor to send the strokes it held and to read the sprite again.
  const [endCount, setEndCount] = useState(0);

  useEffect(() => {
    let stopped = false;
    let unlisten: (() => void) | undefined;
    // The events first, so none is lost while the state is being read.
    api
      .onAgentActivity((next) => {
        if (stopped) return;
        setActivity(next);
        if (next === null) setEndCount((count) => count + 1);
      })
      .then((stop) => {
        if (stopped) stop();
        else unlisten = stop;
      })
      .catch(() => {});
    api
      .agentActivity()
      .then((current) => {
        if (!stopped && current) setActivity(current);
      })
      .catch(() => {});
    return () => {
      stopped = true;
      unlisten?.();
    };
  }, [api]);

  return { activity, endCount };
}

// The same sprite, ignoring the case of the names like the core does.
export function sameSprite(a: SpriteRef, b: SpriteRef) {
  const same = (x: string, y: string) => x.toLowerCase() === y.toLowerCase();
  return (
    a.folder.length === b.folder.length &&
    a.folder.every((name, i) => same(name, b.folder[i])) &&
    same(a.name, b.name)
  );
}
