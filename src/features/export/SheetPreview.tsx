import { useEffect, useRef } from "react";
import type { Preview } from "./types";

// The pixels of the sheet, sharp, on the gray of the canvas. The core draws
// them; this only shows them, as large as fits (up to 4 times).
export function SheetPreview({ preview }: { preview: Preview }) {
  const canvas = useRef<HTMLCanvasElement>(null);
  useEffect(() => {
    const context = canvas.current?.getContext("2d");
    if (!context) return;
    const data = new ImageData(
      new Uint8ClampedArray(preview.rgba),
      preview.width,
      preview.height,
    );
    context.putImageData(data, 0, 0);
  }, [preview]);
  const zoom = preview.width <= 250 ? 4 : preview.width <= 500 ? 2 : 1;
  return (
    <canvas
      ref={canvas}
      width={preview.width}
      height={preview.height}
      role="img"
      aria-label="Sheet"
      style={{ width: preview.width * zoom, imageRendering: "pixelated" }}
      className="max-w-full"
    />
  );
}
