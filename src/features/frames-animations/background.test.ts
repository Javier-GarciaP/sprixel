import { describe, expect, it } from "vitest";
import { backgroundStyle, nextBackground } from "./background";

describe("preview background", () => {
  // Checks that the checker is a pattern and the other two are flat colors,
  // one bright and one dark.
  it("has a checkerboard, a light and a dark background", () => {
    expect(backgroundStyle("checker").backgroundImage).toContain(
      "conic-gradient",
    );
    expect(backgroundStyle("light")).toEqual({ backgroundColor: "#F4F4F5" });
    expect(backgroundStyle("dark")).toEqual({ backgroundColor: "#16161A" });
  });

  // Checks the cycle used by the key: checker, light, dark, checker.
  it("goes through the three backgrounds", () => {
    expect(nextBackground("checker")).toBe("light");
    expect(nextBackground("light")).toBe("dark");
    expect(nextBackground("dark")).toBe("checker");
  });
});
