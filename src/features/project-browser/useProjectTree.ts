import { useEffect, useState } from "react";
import {
  onProjectChanged,
  projectTree,
  type ProjectTree,
} from "@/shared/lib/core";
import type { FrameCache } from "@/shared/lib/render";

// The tree of the open project, as the core sends it. It is read once and
// again after every `project://changed` event. The UI never changes the tree
// by itself: it asks the core and waits for the event.
export function useProjectTree(cache: FrameCache) {
  const [tree, setTree] = useState<ProjectTree | null>(null);

  useEffect(() => {
    let stopped = false;
    let unlisten: (() => void) | undefined;
    const load = () =>
      projectTree()
        .then((next) => {
          if (!stopped) setTree(next);
        })
        .catch(() => {});
    load();
    onProjectChanged((event) => {
      cache.invalidate(event.touched);
      load();
    }).then((stop) => {
      if (stopped) stop();
      else unlisten = stop;
    });
    return () => {
      stopped = true;
      unlisten?.();
    };
  }, [cache]);

  return tree;
}
