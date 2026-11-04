import type { Tag } from "./types";

// The frames of the chosen animation, or all of them (`null`). An animation
// that does not exist any more (deleted or renamed) counts as "All".
export function chosenTag(tags: Tag[], animation: string | null): Tag | null {
  return tags.find((tag) => tag.name === animation) ?? null;
}

export function rangeOf(
  tags: Tag[],
  animation: string | null,
): { from: number; to: number } | null {
  const tag = chosenTag(tags, animation);
  return tag && { from: tag.from, to: tag.to };
}
