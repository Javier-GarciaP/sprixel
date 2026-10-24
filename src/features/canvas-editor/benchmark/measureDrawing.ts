import { ColorBook } from "../controller/colorBook";
import { DrawingController } from "../controller/DrawingController";
import type { Scene, View } from "../render/renderer";
import { Surface } from "../render/surface";

export type DrawingMeasure = {
  size: number;
  samples: number;
  // Time from a pointer event to the next painted frame, in milliseconds.
  medianMs: number;
  p95Ms: number;
  maxMs: number;
  // For information: the time between two frames of this screen. About 16.7
  // at 60 Hz. If it is much more (a window that is hidden or a 30 Hz screen),
  // `medianMs` cannot meet the budget whatever the code does.
  frameMs: number;
  // For information: the median time the pointer move itself takes (the
  // painting in JavaScript, before the browser shows the frame).
  workMs: number;
  // For information, not part of the budget: one whole-frame Bucket fill and
  // one repaint of the whole canvas area (what a zoom or a pan step costs).
  fillMs: number;
  fullRepaintMs: number;
};

const SIZE = 512;
const SAMPLES = 300;
const WARMUP = 20;
// The size of the canvas area of a window of 1440 x 900.
const AREA = { width: 976, height: 760 };
// A pointer event arrives at any moment of a frame (about 16.7 ms at 60 Hz).
// Each sample waits a random part of a frame, so the result does not depend
// on the lucky timing of the test.
const MAX_PHASE_MS = 15;

export function median(values: number[]): number {
  const sorted = [...values].sort((a, b) => a - b);
  const middle = Math.floor(sorted.length / 2);
  return sorted.length % 2 === 1
    ? sorted[middle]
    : (sorted[middle - 1] + sorted[middle]) / 2;
}

// A long wave across the whole sprite, like a fast stroke with the mouse.
export function wavePoints(size: number, count: number) {
  const margin = 8;
  const amplitude = size / 2 - margin;
  return Array.from({ length: count }, (_, i) => {
    const t = i / (count - 1);
    return {
      x: Math.round(margin + (size - 1 - 2 * margin) * t),
      y: Math.round((size - 1) / 2 + amplitude * Math.sin(t * Math.PI * 6)),
    };
  });
}

// Resolves just after the browser has painted the next frame.
function afterNextFrame(): Promise<void> {
  return new Promise((resolve) => {
    requestAnimationFrame(() => {
      const channel = new MessageChannel();
      channel.port1.onmessage = () => resolve();
      channel.port2.postMessage(null);
    });
  });
}

const wait = (ms: number) =>
  new Promise<void>((resolve) => setTimeout(resolve, ms));

// Draws one long stroke on a 512x512 sprite with the real editor code: the
// drawing controller, the renderer and the surface that sends only the
// changed area to the canvas. Onion skin is on, which is the heaviest case
// for the renderer. For each pointer move it measures the time until the
// pixels are on the screen.
export async function measureEditorDrawing(
  canvas: HTMLCanvasElement,
): Promise<DrawingMeasure> {
  const surface = new Surface(canvas);
  surface.resize(AREA.width, AREA.height);
  const view: View = {
    zoom: 1,
    x: Math.floor((AREA.width - SIZE) / 2),
    y: Math.floor((AREA.height - SIZE) / 2),
  };

  const colors = new ColorBook([{ key: "A", hex: "#2D2DFF" }]);
  const pixels = new Uint8Array(SIZE * SIZE).fill(46);
  const previous = new Uint8Array(SIZE * SIZE).fill(46);
  for (let i = 0; i < previous.length; i += 7) previous[i] = 65;
  const scene: Scene = {
    width: SIZE,
    height: SIZE,
    pixels,
    colors: colors.table,
    onion: { pixels: previous, opacity: 0.4 },
    grid: false,
    selection: null,
  };
  const controller = new DrawingController(SIZE, SIZE, pixels, colors, {
    onDirty: (area) => surface.draw(scene, view, area),
    onCommit: () => {},
    onPick: () => {},
  });
  controller.main = "#2D2DFF";

  surface.draw(scene, view);
  const points = wavePoints(SIZE, SAMPLES + WARMUP);
  const latencies: number[] = [];
  const frames: number[] = [];
  const work: number[] = [];
  const none = { shift: false };

  // The time between frames, with no drawing.
  for (let i = 0; i < 31; i++) {
    const before = performance.now();
    await afterNextFrame();
    if (i > 0) frames.push(performance.now() - before);
  }

  controller.pointerDown(points[0].x, points[0].y, 0, none);
  for (let i = 1; i < points.length; i++) {
    await afterNextFrame();
    await wait(Math.random() * MAX_PHASE_MS); // the pointer event arrives
    const start = performance.now();
    controller.pointerMove(points[i].x, points[i].y, none);
    const worked = performance.now() - start;
    await afterNextFrame();
    if (i > WARMUP) {
      latencies.push(performance.now() - start);
      work.push(worked);
    }
  }
  controller.pointerUp();

  const repaintStart = performance.now();
  surface.draw(scene, view);
  const fullRepaintMs = performance.now() - repaintStart;

  controller.tool = "bucket";
  controller.main = "#FF2D2D";
  const fillStart = performance.now();
  controller.pointerDown(1, 1, 0, none);
  controller.pointerUp();
  const fillMs = performance.now() - fillStart;

  const sorted = [...latencies].sort((a, b) => a - b);
  return {
    size: SIZE,
    samples: latencies.length,
    medianMs: median(latencies),
    p95Ms: sorted[Math.floor(sorted.length * 0.95)],
    maxMs: sorted[sorted.length - 1],
    frameMs: median(frames),
    workMs: median(work),
    fillMs,
    fullRepaintMs,
  };
}
