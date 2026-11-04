import { describe, expect, it } from "vitest";
import { copyName, isValid, validateNew, validateTag } from "./validate";

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

describe("validateNew", () => {
  const draft = { name: "walk", count: "4", fps: "12" };

  // A good draft has no errors; the name is compared ignoring case.
  it("accepts a free name, frames and speed", () => {
    expect(validateNew(draft, ["idle"])).toEqual({});
    expect(validateNew({ ...draft, name: "IDLE" }, ["idle"]).name).toBe(
      "taken",
    );
  });

  // An empty name, 0 or too many frames and a speed outside 1-60 are refused.
  it("refuses an empty name and numbers out of range", () => {
    expect(validateNew({ ...draft, name: " " }, []).name).toBe("empty");
    expect(validateNew({ ...draft, count: "0" }, []).count).toBe("range");
    expect(validateNew({ ...draft, count: "101" }, []).count).toBe("range");
    expect(validateNew({ ...draft, fps: "61" }, []).fps).toBe("range");
  });
});

describe("copyName", () => {
  // The copy of "walk" is "walk copy"; if that exists too, a number follows.
  it("finds the first free name", () => {
    expect(copyName("walk", [])).toBe("walk copy");
    expect(copyName("walk", ["Walk Copy"])).toBe("walk copy 2");
    expect(copyName("walk", ["walk copy", "walk copy 2"])).toBe("walk copy 3");
  });
});
