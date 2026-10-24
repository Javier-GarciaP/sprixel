import type { Point } from "../view/viewMath";

// A straight line between two pixels, both ends included, with no gaps
// (Bresenham). It is also what joins two pointer positions of a fast stroke.
export function linePoints(from: Point, to: Point): Point[] {
  const points: Point[] = [];
  let x = from.x;
  let y = from.y;
  const dx = Math.abs(to.x - x);
  const dy = -Math.abs(to.y - y);
  const sx = x < to.x ? 1 : -1;
  const sy = y < to.y ? 1 : -1;
  let error = dx + dy;
  for (;;) {
    points.push({ x, y });
    if (x === to.x && y === to.y) return points;
    const doubled = 2 * error;
    if (doubled >= dy) {
      error += dy;
      x += sx;
    }
    if (doubled <= dx) {
      error += dx;
      y += sy;
    }
  }
}

// Holding Shift: a line snaps to the nearest 45-degree step (horizontal,
// vertical or diagonal) around its start.
export function snapLine(from: Point, to: Point): Point {
  const dx = to.x - from.x;
  const dy = to.y - from.y;
  if (dx === 0 && dy === 0) return to;
  const step = Math.round(Math.atan2(dy, dx) / (Math.PI / 4));
  const direction = [
    [1, 0],
    [1, 1],
    [0, 1],
    [-1, 1],
    [-1, 0],
    [-1, -1],
    [0, -1],
    [1, -1],
  ][((step % 8) + 8) % 8];
  // Diagonals use the longer side, so the line keeps its size.
  const length =
    direction[0] !== 0 && direction[1] !== 0
      ? Math.max(Math.abs(dx), Math.abs(dy))
      : Math.round(Math.hypot(dx, dy));
  return {
    x: from.x + direction[0] * length,
    y: from.y + direction[1] * length,
  };
}

// Holding Shift: a rectangle or ellipse becomes a square or a circle. The
// shorter side grows to the longer one, in the direction of the drag.
export function squareCorner(from: Point, to: Point): Point {
  const size = Math.max(Math.abs(to.x - from.x), Math.abs(to.y - from.y));
  return {
    x: from.x + (to.x < from.x ? -size : size),
    y: from.y + (to.y < from.y ? -size : size),
  };
}

type Box = { left: number; top: number; right: number; bottom: number };

function box(a: Point, b: Point): Box {
  return {
    left: Math.min(a.x, b.x),
    top: Math.min(a.y, b.y),
    right: Math.max(a.x, b.x),
    bottom: Math.max(a.y, b.y),
  };
}

// The pixels of the rectangle with these two opposite corners.
export function rectanglePoints(a: Point, b: Point, filled: boolean): Point[] {
  const { left, top, right, bottom } = box(a, b);
  const points: Point[] = [];
  for (let y = top; y <= bottom; y++) {
    for (let x = left; x <= right; x++) {
      const edge = x === left || x === right || y === top || y === bottom;
      if (filled || edge) points.push({ x, y });
    }
  }
  return points;
}

// The pixels of the ellipse that fits the rectangle with these two corners.
// A pixel is inside when its center is inside the ellipse; the outline is the
// inside pixels that touch the outside.
export function ellipsePoints(a: Point, b: Point, filled: boolean): Point[] {
  const { left, top, right, bottom } = box(a, b);
  const halfWidth = (right - left + 1) / 2;
  const halfHeight = (bottom - top + 1) / 2;
  const centerX = left + halfWidth;
  const centerY = top + halfHeight;
  const inside = (x: number, y: number) =>
    ((x + 0.5 - centerX) / halfWidth) ** 2 +
      ((y + 0.5 - centerY) / halfHeight) ** 2 <=
    1;
  const points: Point[] = [];
  for (let y = top; y <= bottom; y++) {
    for (let x = left; x <= right; x++) {
      if (!inside(x, y)) continue;
      const edge =
        !inside(x - 1, y) ||
        !inside(x + 1, y) ||
        !inside(x, y - 1) ||
        !inside(x, y + 1);
      if (filled || edge) points.push({ x, y });
    }
  }
  return points;
}
