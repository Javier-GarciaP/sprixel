import {
  measureEditorDrawing,
  type DrawingMeasure,
} from "@/features/canvas-editor";

export interface BenchmarkResult extends DrawingMeasure {
  platform: string;
  userAgent: string;
}

// Measures drawing on a 512x512 sprite with the real editor code (the drawing
// controller and the canvas renderer of the editor feature, see
// `measureEditorDrawing`) and adds what is known about the system.
export async function runBenchmark(
  canvas: HTMLCanvasElement,
): Promise<BenchmarkResult> {
  return {
    ...(await measureEditorDrawing(canvas)),
    platform: navigator.platform,
    userAgent: navigator.userAgent,
  };
}
