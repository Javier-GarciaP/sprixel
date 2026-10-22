export interface Rect {
  x: number;
  y: number;
  width: number;
  height: number;
}

export function median(values: number[]): number {
  const sorted = [...values].sort((a, b) => a - b);
  const middle = Math.floor(sorted.length / 2);
  return sorted.length % 2 === 1
    ? sorted[middle]
    : (sorted[middle - 1] + sorted[middle]) / 2;
}

// Paints a round brush into the palette indexes and returns the area that
// changed (clipped to the sprite), so only that area is sent to the canvas.
export function paintBrush(
  indexes: Uint8Array,
  size: number,
  cx: number,
  cy: number,
  radius: number,
  colorIndex: number,
): Rect {
  const x0 = Math.max(0, cx - radius);
  const y0 = Math.max(0, cy - radius);
  const x1 = Math.min(size - 1, cx + radius);
  const y1 = Math.min(size - 1, cy + radius);

  for (let y = y0; y <= y1; y++) {
    for (let x = x0; x <= x1; x++) {
      const dx = x - cx;
      const dy = y - cy;
      if (dx * dx + dy * dy <= radius * radius) {
        indexes[y * size + x] = colorIndex;
      }
    }
  }
  return { x: x0, y: y0, width: x1 - x0 + 1, height: y1 - y0 + 1 };
}

// A long wave across the whole sprite, like a fast stroke with the mouse.
export function strokePoints(size: number, count: number) {
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
