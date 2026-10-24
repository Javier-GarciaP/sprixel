import { useCallback, useEffect, useState } from "react";
import type { EditorApi, SpriteInfo, SpriteRef } from "../api/types";
import { TRANSPARENT_CODE } from "../types";

export type SpriteData =
  | { status: "loading" }
  | { status: "error"; message: string }
  | {
      status: "ready";
      info: SpriteInfo;
      // The pixels of the current frame, one key code per pixel.
      pixels: Uint8Array;
      // The previous frame, when there is one.
      previous: Uint8Array | null;
    };

export function keysToCodes(keys: string): Uint8Array {
  const codes = new Uint8Array(keys.length);
  for (let i = 0; i < keys.length; i++) codes[i] = keys.charCodeAt(i);
  return codes;
}

export const isBlank = (pixels: Uint8Array) =>
  pixels.every((code) => code === TRANSPARENT_CODE);

// Reads the sprite and one frame from the core. `reload` reads them again
// (after an undo, or when an agent finishes writing). The core is the truth:
// this is only a copy to show and to draw on.
export function useSpriteData(
  api: EditorApi,
  sprite: SpriteRef | null,
  frame: number,
  withPrevious: boolean,
) {
  const [data, setData] = useState<SpriteData>({ status: "loading" });
  const [version, setVersion] = useState(0);
  const reload = useCallback(() => setVersion((v) => v + 1), []);
  // The sprite as text: effects compare by value, not by object.
  const key = sprite ? JSON.stringify(sprite) : null;

  useEffect(() => {
    if (!sprite) return;
    let stale = false;
    (async () => {
      try {
        const info = await api.loadSprite(sprite);
        const current = Math.min(frame, info.frameCount);
        const [pixels, previous] = await Promise.all([
          api.loadFrame(sprite, current),
          withPrevious && current > 1
            ? api.loadFrame(sprite, current - 1)
            : Promise.resolve(null),
        ]);
        if (stale) return;
        setData({
          status: "ready",
          info,
          pixels: keysToCodes(pixels),
          previous: previous === null ? null : keysToCodes(previous),
        });
      } catch (error) {
        if (stale) return;
        const message =
          typeof error === "object" && error && "message" in error
            ? String((error as { message: unknown }).message)
            : String(error);
        setData({ status: "error", message });
      }
    })();
    return () => {
      stale = true;
    };
    // `key` stands for `sprite`.
  }, [api, key, frame, withPrevious, version]);

  return {
    data: sprite ? data : ({ status: "loading" } as SpriteData),
    reload,
  };
}
