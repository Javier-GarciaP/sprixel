import { useEffect, useRef } from "react";
import { drawFrame } from "@/shared/lib/render";
import { useT } from "@/shared/lib/i18n";
import type { GridPreview as Preview } from "./types";

// The biggest the picture gets on the screen, in pixels.
const BOX = 320;

// A neutral gray checkerboard, the same in both themes.
const checkerboard = {
  backgroundColor: "#3a3a42",
  backgroundImage:
    "conic-gradient(#2e2e35 25%, transparent 25% 50%, #2e2e35 50% 75%, transparent 75%)",
  backgroundSize: "16px 16px",
} as const;

// The picture of a valid grid, scaled by a whole number to fit in the box.
// `dimmed` shows that it is the last valid state and not the current text.
export function GridPreview({
  preview,
  dimmed,
}: {
  preview: Preview | null;
  dimmed: boolean;
}) {
  const t = useT();
  const canvas = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    if (preview && canvas.current) {
      drawFrame(canvas.current, { revision: 0, ...preview }, BOX);
    }
  }, [preview]);

  return (
    <div
      className="flex items-center justify-center rounded"
      style={{ ...checkerboard, width: BOX, height: BOX }}
    >
      {preview && (
        <canvas
          ref={canvas}
          role="img"
          aria-label={t("text.preview")}
          data-rows={preview.rows.join("|")}
          className={`[image-rendering:pixelated] ${dimmed ? "opacity-60" : ""}`}
        />
      )}
    </div>
  );
}
