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

export const MAX_NEW_FRAMES = 100;

export type NewErrors = {
  name?: "empty" | "taken";
  count?: "range";
  fps?: "range";
};

export type NewDraft = { name: string; count: string; fps: string };

// Checks "New animation": a name that is free and a number of frames to copy.
export function validateNew(draft: NewDraft, others: string[]): NewErrors {
  const errors: NewErrors = {};
  const name = draft.name.trim();
  if (!name) errors.name = "empty";
  else if (others.some((other) => other.toLowerCase() === name.toLowerCase())) {
    errors.name = "taken";
  }
  const count = whole(draft.count);
  const fps = whole(draft.fps);
  if (!(count >= 1 && count <= MAX_NEW_FRAMES)) errors.count = "range";
  if (!(fps >= 1 && fps <= 60)) errors.fps = "range";
  return errors;
}

// "walk copy", "walk copy 2", ... the first name that no animation uses.
export function copyName(name: string, others: string[]): string {
  const used = new Set(others.map((other) => other.toLowerCase()));
  let candidate = `${name} copy`;
  for (let n = 2; used.has(candidate.toLowerCase()); n++) {
    candidate = `${name} copy ${n}`;
  }
  return candidate;
}
