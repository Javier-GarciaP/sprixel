import type { Tag } from "./types";

// A block of the frames strip: the "Base" frames, one animation, or the frames
// that belong to no animation.
export type Group = {
  id: string;
  kind: "base" | "tag" | "none";
  tag?: Tag;
  frames: number[];
};

const range = (from: number, to: number) =>
  Array.from({ length: Math.max(0, to - from + 1) }, (_, i) => from + i);

// Splits the frames of a sprite into groups. "Base" is every frame before the
// first animation; "No animation" is every later frame that no animation owns.
// A frame that two old animations share is listed in the first one (by start
// frame) and `overlaps` is true so the strip can warn about it.
export function groupsOf(
  tags: Tag[],
  frameCount: number,
): { groups: Group[]; overlaps: boolean } {
  const sorted = [...tags].sort((a, b) => a.from - b.from);
  const owner = new Map<number, string>();
  let overlaps = false;
  const owned = new Map<string, number[]>();
  for (const tag of sorted) {
    const own: number[] = [];
    for (const n of range(tag.from, Math.min(tag.to, frameCount))) {
      if (owner.has(n)) overlaps = true;
      else {
        owner.set(n, tag.name);
        own.push(n);
      }
    }
    owned.set(tag.name, own);
  }
  const firstFrom = sorted.length ? sorted[0].from : frameCount + 1;
  const base = range(1, Math.min(firstFrom - 1, frameCount));
  const none = range(firstFrom, frameCount).filter((n) => !owner.has(n));

  const groups: Group[] = [];
  if (base.length) groups.push({ id: "base", kind: "base", frames: base });
  for (const tag of sorted) {
    groups.push({
      id: `tag:${tag.name}`,
      kind: "tag",
      tag,
      frames: owned.get(tag.name) ?? [],
    });
  }
  if (none.length) groups.push({ id: "none", kind: "none", frames: none });
  return { groups, overlaps };
}
