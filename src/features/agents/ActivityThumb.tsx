import { useEffect, useRef } from "react";
import { drawFrame } from "@/shared/lib/render";
import type { AgentsApi } from "./types";

const BOX = 40;

// The first frame of the sprite an action was about, on a gray checkerboard.
// A sprite that is gone (deleted, renamed) leaves the empty checkerboard.
export function ActivityThumb({
  api,
  sprite,
  label,
}: {
  api: AgentsApi;
  // A path like "Characters/goblin".
  sprite: string | null;
  label: string;
}) {
  const canvas = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    if (!sprite) return;
    let stopped = false;
    const parts = sprite.split("/");
    const name = parts.pop() ?? "";
    api
      .frame(parts, name)
      .then((frame) => {
        if (!stopped && canvas.current) drawFrame(canvas.current, frame, BOX);
      })
      .catch(() => {});
    return () => {
      stopped = true;
    };
  }, [api, sprite]);

  return (
    <div
      className="flex shrink-0 items-center justify-center rounded-md"
      style={{
        width: BOX,
        height: BOX,
        backgroundColor: "#3a3a42",
        backgroundImage:
          "conic-gradient(#2e2e35 25%, transparent 25% 50%, #2e2e35 50% 75%, transparent 75%)",
        backgroundSize: "8px 8px",
      }}
    >
      <canvas
        ref={canvas}
        role="img"
        aria-label={label}
        className="[image-rendering:pixelated]"
      />
    </div>
  );
}
