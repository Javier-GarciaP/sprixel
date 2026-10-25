import { useEffect, useRef } from "react";
import { drawFrame } from "@/shared/lib/render";
import type { FramesApi, SpriteRef } from "./types";

// A neutral gray checkerboard, the same in both themes.
const checkerboard = {
  backgroundColor: "#3a3a42",
  backgroundImage:
    "conic-gradient(#2e2e35 25%, transparent 25% 50%, #2e2e35 50% 75%, transparent 75%)",
  backgroundSize: "12px 12px",
} as const;

type Props = {
  api: FramesApi;
  sprite: SpriteRef;
  frame: number;
  // Changes when the sprite changes, so the picture is read again.
  revision: number;
  box: number;
};

// The picture of one frame, scaled by a whole number to fit in `box` pixels.
export function FrameThumb({ api, sprite, frame, revision, box }: Props) {
  const canvas = useRef<HTMLCanvasElement>(null);
  const path = JSON.stringify([...sprite.folder, sprite.name]);

  useEffect(() => {
    let stopped = false;
    api
      .loadFrameData(sprite, frame)
      .then((data) => {
        if (!stopped && canvas.current) drawFrame(canvas.current, data, box);
      })
      .catch(() => {});
    return () => {
      stopped = true;
    };
  }, [api, path, frame, revision, box]);

  return (
    <div
      className="flex items-center justify-center rounded"
      style={{ ...checkerboard, width: box, height: box }}
    >
      <canvas
        ref={canvas}
        aria-hidden
        className="[image-rendering:pixelated]"
      />
    </div>
  );
}
