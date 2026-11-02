import { describe, expect, it } from "vitest";
import {
  changedKeys,
  draftOf,
  keyError,
  nameError,
  newColor,
  nextFreeKey,
  toEdits,
  toHex,
  toRgb,
} from "./draft";

const entry = (key: string, name: string, hex = "#112233", used = false) => ({
  key,
  name,
  hex,
  used,
});
const draft = draftOf([entry("R", "red"), entry("B", "blue")]);

describe("key rules", () => {
  // Checks the "Reserved key" scenario: "." and the space are refused.
  it("refuses the dot and the space as reserved", () => {
    expect(keyError(draft, 0, ".")).toBe("reserved");
    expect(keyError(draft, 0, " ")).toBe("reserved");
  });

  // Checks that a key is one printable ASCII character, case sensitive.
  it("accepts one printable character and refuses the rest", () => {
    expect(keyError(draft, 0, "")).toBe("bad");
    expect(keyError(draft, 0, "HH")).toBe("bad");
    expect(keyError(draft, 0, "é")).toBe("bad");
    expect(keyError(draft, 0, "r")).toBeNull();
    expect(keyError(draft, 0, "#")).toBeNull();
  });

  // Checks the "Duplicate key" scenario: the key of another color is refused,
  // but a color can keep its own key.
  it("refuses a key that another color has", () => {
    expect(keyError(draft, 0, "B")).toBe("duplicate");
    expect(keyError(draft, 0, "R")).toBeNull();
  });
});

describe("name rules", () => {
  // Checks that a name is not empty and not used by another color (letter
  // case does not count).
  it("needs a name that no other color has", () => {
    expect(nameError(draft, 0, "  ")).toBe("empty");
    expect(nameError(draft, 0, "BLUE")).toBe("duplicate");
    expect(nameError(draft, 0, "RED")).toBeNull();
  });
});

describe("new colors", () => {
  // Checks the order of the generated keys: letters first, skipping used ones.
  it("takes the next free key", () => {
    expect(nextFreeKey(draft)).toBe("A");
    const letters = draftOf(
      [..."ABCDEFGHIJKLMNOPQRSTUVWXYZ"].map((k) => entry(k, `c${k}`)),
    );
    expect(nextFreeKey(letters)).toBe("a");
  });

  // Checks that the 94th color cannot be added.
  it("gives nothing when the palette is full", () => {
    let full = draftOf([]);
    for (let i = 0; i < 93; i++) {
      full = [...full, { ...newColor(full)!, name: `c${i}` }];
    }
    expect(full).toHaveLength(93);
    expect(newColor(full)).toBeNull();
  });

  // Checks that two new colors get different names and ids.
  it("gives each new color its own name", () => {
    const first = newColor(draft)!;
    const second = newColor([...draft, first])!;
    expect(first.name).not.toBe(second.name);
    expect(first.id).not.toBe(second.id);
  });
});

describe("what is sent on Done", () => {
  // Checks that only a color whose key changed is a changed key (the warning).
  it("lists the keys that changed", () => {
    const edited = draft.map((c) => (c.id === 0 ? { ...c, key: "H" } : c));
    expect(changedKeys(edited)).toEqual(["R"]);
    expect(changedKeys(draft)).toEqual([]);
  });

  // Checks that the original key is kept as `from` and a new color has none.
  it("sends the original key of each color", () => {
    const edited = [
      ...draft.map((c) => (c.id === 0 ? { ...c, key: "H" } : c)),
      newColor(draft)!,
    ];
    expect(toEdits(edited).map((e) => [e.from, e.key])).toEqual([
      ["R", "H"],
      ["B", "B"],
      [null, "A"],
    ]);
  });

  // Checks the conversion between #RRGGBB and the three sliders.
  it("converts hex and RGB both ways", () => {
    expect(toRgb("#F04545")).toEqual([240, 69, 69]);
    expect(toHex([240, 69, 69])).toBe("#F04545");
    expect(toHex([0, 0, 0])).toBe("#000000");
  });
});
