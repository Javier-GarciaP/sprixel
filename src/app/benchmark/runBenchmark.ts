import { median, paintBrush, strokePoints } from "./stroke";

export interface BenchmarkResult {
  size: number;
  samples: number;
  medianMs: number;
  p95Ms: number;
  maxMs: number;
  platform: string;
  userAgent: string;
}

const SIZE = 512;
const SAMPLES = 300;
const WARMUP = 20;
const BRUSH_RADIUS = 4;
// A real pointer event arrives at any moment of a frame (about 16.7 ms at
// 60 Hz). Each sample waits a random part of a frame, so the result does not
// depend on the lucky timing of the test.
const MAX_PHASE_MS = 15;

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

function wait(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

// Paints one long stroke on a 512x512 sprite the way the editor will: palette
// indexes in a typed array, one canvas, only the changed area sent to it.
// For each point it measures the time until the pixels are on screen.
export async function runBenchmark(
  canvas: HTMLCanvasElement,
): Promise<BenchmarkResult> {
  canvas.width = SIZE;
  canvas.height = SIZE;
  const context = canvas.getContext("2d");
  if (!context) throw new Error("Canvas 2D is not available");

  const palette = new Uint32Array([0xffffffff, 0xff2d2dff, 0xff3c9b2d]); // ABGR
  const indexes = new Uint8Array(SIZE * SIZE);
  const image = context.createImageData(SIZE, SIZE);
  const pixels = new Uint32Array(image.data.buffer);
  pixels.fill(palette[0]);
  context.putImageData(image, 0, 0);

  const latencies: number[] = [];
  const points = strokePoints(SIZE, SAMPLES + WARMUP);

  for (let i = 0; i < points.length; i++) {
    await afterNextFrame();
    await wait(Math.random() * MAX_PHASE_MS); // the "pointer event" arrives
    const start = performance.now();

    const { x, y } = points[i];
    const rect = paintBrush(indexes, SIZE, x, y, BRUSH_RADIUS, 1 + (i % 2));
    for (let row = rect.y; row < rect.y + rect.height; row++) {
      for (let col = rect.x; col < rect.x + rect.width; col++) {
        pixels[row * SIZE + col] = palette[indexes[row * SIZE + col]];
      }
    }
    context.putImageData(image, 0, 0, rect.x, rect.y, rect.width, rect.height);

    await afterNextFrame();
    if (i >= WARMUP) latencies.push(performance.now() - start);
  }

  const sorted = [...latencies].sort((a, b) => a - b);
  return {
    size: SIZE,
    samples: latencies.length,
    medianMs: median(latencies),
    p95Ms: sorted[Math.floor(sorted.length * 0.95)],
    maxMs: sorted[sorted.length - 1],
    platform: navigator.platform,
    userAgent: navigator.userAgent,
  };
}
