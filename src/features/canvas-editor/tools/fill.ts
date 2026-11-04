import type { Draft } from "./draft";

// Fills the area of the same code that touches (x, y) with `code`, through
// the draft. Pixels of other colors stop the fill; two pixels are connected
// when they share an edge. It is iterative (row spans and a stack of seeds),
// so a 512x512 sprite neither freezes the screen nor overflows the call stack.
// Clicking an area that already has the code changes nothing.
export function floodFill(
  draft: Draft,
  pixels: Uint8Array,
  x: number,
  y: number,
  code: number,
) {
  const { width, height } = draft;
  if (!draft.inside(x, y)) return;
  const target = pixels[y * width + x];
  if (target === code) return;

  const stack: number[] = [x, y];
  while (stack.length > 0) {
    const sy = stack.pop()!;
    const sx = stack.pop()!;
    if (pixels[sy * width + sx] !== target) continue;
    // Take the whole span of this row.
    let left = sx;
    let right = sx;
    while (left > 0 && pixels[sy * width + left - 1] === target) left--;
    while (right < width - 1 && pixels[sy * width + right + 1] === target)
      right++;
    for (let px = left; px <= right; px++) draft.set(px, sy, code);
    // Seed the rows above and below, once for each run inside the span.
    for (const ny of [sy - 1, sy + 1]) {
      if (ny < 0 || ny >= height) continue;
      let inRun = false;
      for (let px = left; px <= right; px++) {
        const match = pixels[ny * width + px] === target;
        if (match && !inRun) stack.push(px, ny);
        inRun = match;
      }
    }
  }
}
