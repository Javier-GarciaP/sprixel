import { describe, expect, it } from "vitest";
import { Draft } from "./draft";

const make = () => {
  const pixels = new Uint8Array(16).fill(46);
  return { pixels, draft: new Draft(pixels, 4, 4) };
};

describe("Draft", () => {
  // Checks that set changes the pixel, remembers it, and lists it.
  it("lists the pixels that changed", () => {
    const { pixels, draft } = make();
    draft.set(1, 2, 82);
    expect(pixels[2 * 4 + 1]).toBe(82);
    expect(draft.changes()).toEqual([{ x: 1, y: 2, code: 82 }]);
  });

  // Checks the clipping rule: pixels outside the sprite are ignored.
  it("ignores pixels outside the sprite", () => {
    const { pixels, draft } = make();
    draft.set(-1, 0, 82);
    draft.set(4, 0, 82);
    draft.set(0, 4, 82);
    expect(pixels.every((c) => c === 46)).toBe(true);
    expect(draft.changes()).toEqual([]);
  });

  // Checks that a pixel painted back to its first value is not a change, so
  // a stroke never sends pixels that did not change.
  it("does not list a pixel that came back to its first value", () => {
    const { draft } = make();
    draft.set(0, 0, 82);
    draft.set(0, 0, 46);
    expect(draft.changes()).toEqual([]);
  });

  // Checks revert: the pixels return to the first value and the draft can be
  // used again (this is how a shape preview follows the pointer).
  it("reverts to the first values and can be used again", () => {
    const { pixels, draft } = make();
    draft.set(0, 0, 82);
    draft.set(1, 0, 82);
    draft.revert();
    expect(pixels.every((c) => c === 46)).toBe(true);
    draft.set(2, 0, 66);
    expect(draft.changes()).toEqual([{ x: 2, y: 0, code: 66 }]);
  });

  // Checks that the dirty area covers every touched pixel, also after a
  // revert, and is empty after it is taken.
  it("tracks the area to repaint", () => {
    const { draft } = make();
    expect(draft.takeDirty()).toBeNull();
    draft.set(1, 1, 82);
    draft.set(3, 2, 82);
    expect(draft.takeDirty()).toEqual({ x: 1, y: 1, width: 3, height: 2 });
    expect(draft.takeDirty()).toBeNull();
    draft.revert();
    expect(draft.takeDirty()).toEqual({ x: 1, y: 1, width: 3, height: 2 });
  });

  // Checks close: what is on the canvas becomes the new start.
  it("starts again from the current pixels after close", () => {
    const { draft } = make();
    draft.set(0, 0, 82);
    draft.close();
    draft.set(0, 0, 66);
    expect(draft.changes()).toEqual([{ x: 0, y: 0, code: 66 }]);
    draft.revert();
    expect(draft.changes()).toEqual([]);
  });
});
