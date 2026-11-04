import type { Anchor, AnchorKey } from "../api/types";

const same = (a: string, b: string) => a.toLowerCase() === b.toLowerCase();

// The anchors that count on a frame: the ones of that frame, and the ones of
// the whole sprite, except those a frame anchor of the same name replaces
// (the core uses the same rule).
export function effectiveAnchors(all: Anchor[], frame: number): Anchor[] {
  const own = all.filter((a) => a.frame === frame);
  const shared = all.filter(
    (a) => a.frame === null && !own.some((o) => same(o.name, a.name)),
  );
  return [...shared, ...own];
}

export function anchorAt(
  list: Anchor[],
  x: number,
  y: number,
): Anchor | undefined {
  return list.find((a) => a.x === x && a.y === y);
}

export const keyOf = (anchor: Anchor): AnchorKey => ({
  name: anchor.name,
  frame: anchor.frame,
});

export const sameKey = (a: AnchorKey, b: AnchorKey) =>
  a.frame === b.frame && same(a.name, b.name);
