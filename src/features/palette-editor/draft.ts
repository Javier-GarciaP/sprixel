import type { ColorEdit, PaletteEntry, Replacement } from "./types";

export const MAX_COLORS = 93;

// The order the core gives keys to new colors: A to Z, a to z, 0 to 9, then
// the other printable characters (not the space and not the dot).
const KEY_ORDER =
  "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789" +
  "!\"#$%&'()*+,-/:;<=>?@[\\]^_`{|}~";

// One color of the copy the dialog works on. `from` is the key it had when
// the dialog opened (null for a color added here); `id` never changes.
export type DraftColor = {
  id: number;
  from: string | null;
  key: string;
  name: string;
  hex: string;
  used: boolean;
};

export function draftOf(colors: PaletteEntry[]): DraftColor[] {
  return colors.map((color, id) => ({ id, from: color.key, ...color }));
}

export function nextFreeKey(draft: DraftColor[]): string | null {
  const taken = new Set(draft.map((c) => c.key));
  return [...KEY_ORDER].find((k) => !taken.has(k)) ?? null;
}

export type KeyError = "reserved" | "bad" | "duplicate";

// The rules of the core, to show the error as soon as the user types.
export function keyError(
  draft: DraftColor[],
  id: number,
  key: string,
): KeyError | null {
  if (key === "." || key === " ") return "reserved";
  if (!/^[!-~]$/.test(key)) return "bad";
  return draft.some((c) => c.id !== id && c.key === key) ? "duplicate" : null;
}

export type NameError = "empty" | "duplicate";

export function nameError(
  draft: DraftColor[],
  id: number,
  name: string,
): NameError | null {
  if (name.trim() === "") return "empty";
  const lower = name.trim().toLowerCase();
  return draft.some((c) => c.id !== id && c.name.toLowerCase() === lower)
    ? "duplicate"
    : null;
}

export const isHex = (hex: string) => /^#[0-9a-f]{6}$/i.test(hex);

export function toRgb(hex: string): [number, number, number] {
  return [1, 3, 5].map((i) => parseInt(hex.slice(i, i + 2), 16)) as [
    number,
    number,
    number,
  ];
}

export function toHex(rgb: [number, number, number]): string {
  return `#${rgb.map((n) => n.toString(16).padStart(2, "0")).join("")}`.toUpperCase();
}

// A new color: the next free key, a gray, and a name nobody uses.
export function newColor(draft: DraftColor[]): DraftColor | null {
  const key = nextFreeKey(draft);
  if (key === null) return null;
  const hex = "#808080";
  let name = hex.slice(1);
  for (
    let n = 2;
    draft.some((c) => c.name.toLowerCase() === name.toLowerCase());
    n++
  ) {
    name = `${hex.slice(1)} ${n}`;
  }
  return {
    id: Math.max(-1, ...draft.map((c) => c.id)) + 1,
    from: null,
    key,
    name,
    hex,
    used: false,
  };
}

// The keys that changed: the text grids that use the old key stop being valid.
export function changedKeys(draft: DraftColor[]): string[] {
  return draft
    .filter((c) => c.from !== null && c.from !== c.key)
    .map((c) => c.from as string);
}

export function toEdits(draft: DraftColor[]): ColorEdit[] {
  return draft.map(({ from, key, name, hex }) => ({
    from,
    key,
    name: name.trim(),
    hex,
  }));
}

export type Removed = { key: string; with: string | null };

// The replacements to send: only for removed colors that sprites use.
export function toReplacements(removed: Removed[]): Replacement[] {
  return removed.flatMap((r) =>
    r.with === null ? [] : [{ key: r.key, with: r.with }],
  );
}
