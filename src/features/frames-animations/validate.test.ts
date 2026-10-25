import { describe, expect, it } from "vitest";
import { isValid, validateTag } from "./validate";

const draft = { name: "walk", from: "1", to: "4", fps: "12" };

describe("validateTag", () => {
  // Checks the "Create an animation" input: walk, frames 1 to 4 at 12 fps is
  // valid on a sprite with 7 frames.
  it("accepts a good animation", () => {
    expect(isValid(validateTag(draft, 7, []))).toBe(true);
  });

  // Checks the "Invalid range" scenario: an end frame below the start frame is
  // an error on the end field.
  it("refuses an end frame below the start frame", () => {
    expect(validateTag({ ...draft, from: "4", to: "2" }, 7, [])).toEqual({
      to: "order",
    });
  });

  // Checks the frame limits: from 1 to the number of frames, whole numbers.
  it("keeps the frames inside the sprite", () => {
    const errors = validateTag({ ...draft, from: "0", to: "9" }, 7, []);
    expect(errors).toEqual({ from: "range", to: "range" });
    expect(validateTag({ ...draft, to: "" }, 7, []).to).toBe("range");
    expect(validateTag({ ...draft, from: "1.5" }, 7, []).from).toBe("range");
  });

  // Checks the speed limits, 1 to 60 fps.
  it("keeps the speed between 1 and 60 fps", () => {
    expect(validateTag({ ...draft, fps: "0" }, 7, []).fps).toBe("range");
    expect(validateTag({ ...draft, fps: "61" }, 7, []).fps).toBe("range");
    expect(isValid(validateTag({ ...draft, fps: "60" }, 7, []))).toBe(true);
    expect(isValid(validateTag({ ...draft, fps: "1" }, 7, []))).toBe(true);
  });

  // Checks the name: needed, and not used by another animation (ignoring the
  // letter case).
  it("needs a name that no other animation has", () => {
    expect(validateTag({ ...draft, name: "  " }, 7, []).name).toBe("empty");
    expect(validateTag(draft, 7, ["Walk"]).name).toBe("taken");
    expect(validateTag(draft, 7, ["idle"]).name).toBeUndefined();
  });
});
