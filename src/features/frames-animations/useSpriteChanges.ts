import { useEffect, useState } from "react";
import type { FramesApi, SpriteRef } from "./types";

// A number that grows each time the project changes the sprite (a stroke, an
// undo, a frame change, an agent). Thumbnails and the preview read the
// frames again when it changes.
export function useSpriteChanges(api: FramesApi, sprite: SpriteRef): number {
  const [count, setCount] = useState(0);
  const path = JSON.stringify([...sprite.folder, sprite.name]);

  useEffect(() => {
    let stopped = false;
    let stop: (() => void) | undefined;
    api
      .onChanged((event) => {
        const mine = event.touched.some(
          (part) =>
            part.kind === "sprite" && JSON.stringify(part.path) === path,
        );
        if (mine && !stopped) setCount((n) => n + 1);
      })
      .then((unlisten) => {
        if (stopped) unlisten();
        else stop = unlisten;
      })
      .catch(() => {});
    return () => {
      stopped = true;
      stop?.();
    };
  }, [api, path]);

  return count;
}
