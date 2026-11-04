import { describe, expect, it } from "vitest";
import { anchorAt, effectiveAnchors, sameKey } from "./anchors";

const feet = { name: "feet", x: 1, y: 7, frame: null };
const feet2 = { name: "Feet", x: 2, y: 7, frame: 2 };
const hand = { name: "hand", x: 5, y: 3, frame: 3 };

describe("anchor helpers", () => {
  // Checks the rule of the core: the anchor of a frame replaces the anchor of
  // the whole sprite with the same name (ignoring case), and the anchors of
  // other frames are not shown.
  it("lets a frame anchor win over the sprite anchor", () => {
    expect(effectiveAnchors([feet, feet2, hand], 2)).toEqual([feet2]);
    expect(effectiveAnchors([feet, feet2, hand], 1)).toEqual([feet]);
    expect(effectiveAnchors([feet, feet2, hand], 3)).toEqual([feet, hand]);
  });

  // Checks finding the anchor under a pixel, and the key comparison (name
  // without case, same frame).
  it("finds an anchor by pixel and compares keys", () => {
    expect(anchorAt([feet, hand], 5, 3)).toBe(hand);
    expect(anchorAt([feet, hand], 0, 0)).toBeUndefined();
    expect(
      sameKey({ name: "feet", frame: null }, { name: "FEET", frame: null }),
    ).toBe(true);
    expect(
      sameKey({ name: "feet", frame: null }, { name: "feet", frame: 2 }),
    ).toBe(false);
  });
});
