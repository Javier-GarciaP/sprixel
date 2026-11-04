// Symmetry mode: what the person draws is also drawn mirrored. The mirror
// lines are the center of the sprite (the same as the core and the agent
// tools use when no axis is given).
export type Symmetry = "off" | "horizontal" | "vertical" | "both";

// The order the S key goes through.
export const symmetryModes: Symmetry[] = [
  "off",
  "horizontal",
  "vertical",
  "both",
];

export function nextSymmetry(mode: Symmetry): Symmetry {
  return symmetryModes[
    (symmetryModes.indexOf(mode) + 1) % symmetryModes.length
  ];
}

// The pixel and its mirrors. `horizontal` mirrors left to right (x changes)
// and `vertical` top to bottom (y changes), like `symmetry` in the tools of
// the agent. The first point is always the pixel itself.
export function mirrorPoints(
  x: number,
  y: number,
  width: number,
  height: number,
  mode: Symmetry,
): { x: number; y: number }[] {
  const points = [{ x, y }];
  const mx = width - 1 - x;
  const my = height - 1 - y;
  if (mode === "horizontal" || mode === "both") points.push({ x: mx, y });
  if (mode === "vertical" || mode === "both") points.push({ x, y: my });
  if (mode === "both") points.push({ x: mx, y: my });
  return points;
}

// The mirror lines to draw on the canvas, in sprite pixels: `x` is the
// position of the vertical line and `y` of the horizontal one (half pixels
// when the size is odd or even), or null when that line is off.
export function axisLines(
  width: number,
  height: number,
  mode: Symmetry,
): { x: number | null; y: number | null } {
  return {
    x: mode === "horizontal" || mode === "both" ? width / 2 : null,
    y: mode === "vertical" || mode === "both" ? height / 2 : null,
  };
}
