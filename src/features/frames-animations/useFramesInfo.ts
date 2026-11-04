import { useEffect, useState } from "react";
import type { FramesApi, FramesInfo, SpriteRef } from "./types";
import { useSpriteChanges } from "./useSpriteChanges";

// The frames and animations of the sprite, read again every time the project
// changes it. `null` until the first answer.
export function useFramesInfo(api: FramesApi, sprite: SpriteRef) {
  const changes = useSpriteChanges(api, sprite);
  const [info, setInfo] = useState<FramesInfo | null>(null);
  const path = JSON.stringify([...sprite.folder, sprite.name]);

  useEffect(() => {
    let stopped = false;
    api
      .loadInfo(sprite)
      .then((loaded) => {
        if (!stopped) setInfo(loaded);
      })
      .catch(() => {});
    return () => {
      stopped = true;
    };
  }, [api, path, changes]);

  return info;
}
