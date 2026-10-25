import type { Tag } from "./types";

export type TagErrors = {
  name?: "empty" | "taken";
  from?: "range";
  to?: "range" | "order";
  fps?: "range";
};

// The input of the dialog: numbers are kept as typed until they are valid.
export type TagDraft = {
  name: string;
  from: string;
  to: string;
  fps: string;
};

const whole = (text: string) =>
  /^\d+$/.test(text.trim()) ? Number(text) : NaN;

// Checks an animation against the sprite it belongs to. `others` are the
// names the other animations of the sprite already use.
export function validateTag(
  draft: TagDraft,
  frameCount: number,
  others: string[],
): TagErrors {
  const errors: TagErrors = {};
  const name = draft.name.trim();
  if (!name) errors.name = "empty";
  else if (others.some((other) => other.toLowerCase() === name.toLowerCase())) {
    errors.name = "taken";
  }
  const from = whole(draft.from);
  const to = whole(draft.to);
  const fps = whole(draft.fps);
  if (!(from >= 1 && from <= frameCount)) errors.from = "range";
  if (!(to >= 1 && to <= frameCount)) errors.to = "range";
  else if (!errors.from && to < from) errors.to = "order";
  if (!(fps >= 1 && fps <= 60)) errors.fps = "range";
  return errors;
}

export const isValid = (errors: TagErrors) => Object.keys(errors).length === 0;

export const draftOf = (tag: Tag): TagDraft => ({
  name: tag.name,
  from: String(tag.from),
  to: String(tag.to),
  fps: String(tag.fps),
});
