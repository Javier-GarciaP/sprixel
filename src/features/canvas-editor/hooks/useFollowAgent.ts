import { useEffect, useRef, type RefObject } from "react";
import type { EditorApi, SpriteRef } from "../api/types";
import { sameSprite } from "./useAgentActivity";

// Follows an agent to another sprite: when it writes a sprite that is not the
// open one (or none is open), the editor asks the app to show it. It never
// does this while the person has a stroke in progress, so the view does not
// change under their hand. The editor is only mounted in the Editor tab, so
// the rule "only in the editor tab" is true whenever this runs.
// (A write to another frame of the open sprite is handled by `useAgentPlayback`.)
export function useFollowAgent(
  api: EditorApi,
  sprite: SpriteRef | null,
  strokeActive: RefObject<boolean>,
  onFollow?: (sprite: SpriteRef, frame: number) => void,
) {
  const latest = useRef({ sprite, onFollow });
  latest.current = { sprite, onFollow };
  useEffect(() => {
    let stopped = false;
    let unlisten: (() => void) | undefined;
    api
      .onAgentOps((ops) => {
        const { sprite: open, onFollow: follow } = latest.current;
        if (stopped || (open && sameSprite(ops.sprite, open))) return;
        if (!strokeActive.current) follow?.(ops.sprite, ops.frame);
        // The sprite shown next reads the final state from the core, so there
        // is nothing to replay: the frame is free.
        api.playbackDone(ops.id).catch(() => {});
      })
      .then((stop) => {
        if (stopped) stop();
        else unlisten = stop;
      })
      .catch(() => {});
    return () => {
      stopped = true;
      unlisten?.();
    };
  }, [api, strokeActive]);
}
