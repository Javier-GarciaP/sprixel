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
  // How many previous frames onion skin shows (0 is off).
  count: number,
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
        const pixels = await api.loadFrame(sprite, current);
        if (stale) return;
        setData({ status: "ready", info, pixels: keysToCodes(pixels) });
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
  }, [api, key, frame, version]);

  // The previous frames for onion skin, nearest first (`count` of them, 0
  // when it is off). They are only pictures to look at: they are read on
  // their own, so turning onion skin on or off does not reload the frame that
  // is being drawn. Frame 1 has none; frame 2 has one.
  const [previous, setPrevious] = useState<Uint8Array[]>([]);
  useEffect(() => {
    const wanted = Array.from(
      { length: Math.min(count, Math.max(0, frame - 1)) },
      (_, i) => frame - 1 - i,
    );
    if (!sprite || wanted.length === 0) {
      setPrevious([]);
      return;
    }
    let stale = false;
    Promise.all(wanted.map((n) => api.loadFrame(sprite, n)))
      .then((list) => {
        if (!stale) setPrevious(list.map(keysToCodes));
      })
      .catch(() => {
        if (!stale) setPrevious([]);
      });
    return () => {
      stale = true;
    };
  }, [api, key, frame, count, version]);

  return {
    data: sprite ? data : ({ status: "loading" } as SpriteData),
    previous,
    reload,
  };
}
