import type { FrameData } from "@/shared/lib/core";
import { frameToRgba, integerScale, scaleRgba } from "./rgba";

// Draws a frame on a canvas, scaled by a whole number so that it fits in
// `box` pixels. The canvas takes the size of the result. Returns the scale.
export function drawFrame(
  canvas: HTMLCanvasElement,
  frame: FrameData,
  box: number,
): number {
  const scale = integerScale(frame.width, frame.height, box);
  canvas.width = frame.width * scale;
  canvas.height = frame.height * scale;
  const context = canvas.getContext("2d");
  if (!context) return scale;
  const pixels = scaleRgba(
    frameToRgba(frame),
    frame.width,
    frame.height,
    scale,
  );
  context.putImageData(
    new ImageData(pixels, canvas.width, canvas.height),
    0,
    0,
  );
  return scale;
}
