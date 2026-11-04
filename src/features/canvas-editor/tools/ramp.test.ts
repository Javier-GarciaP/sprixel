import { describe, expect, it } from "vitest";
import { darkToLight, darkestKey, lightness } from "./ramp";

const palette = [
  { key: "L", hex: "#FFFFFF" },
  { key: "B", hex: "#3366CC" },
  { key: "K", hex: "#000000" },
];

describe("shading ramp helpers", () => {
  // Checks the lightness of black and white and that a blue is in between.
  it("measures lightness", () => {
    expect(lightness("#000000")).toBe(0);
    expect(lightness("#FFFFFF")).toBeCloseTo(255, 0);
    expect(lightness("#3366CC")).toBeGreaterThan(0);
    expect(lightness("#3366CC")).toBeLessThan(255);
  });

  // Checks that the chosen keys come back dark to light, whatever the order
  // they were picked in, and that the palette is not changed.
  it("sorts keys from dark to light", () => {
    expect(darkToLight(palette, ["L", "K", "B"])).toEqual(["K", "B", "L"]);
    expect(palette.map((c) => c.key)).toEqual(["L", "B", "K"]);
  });

  // Checks the default outline color: the darkest, or none for an empty palette.
  it("finds the darkest key", () => {
    expect(darkestKey(palette)).toBe("K");
    expect(darkestKey([])).toBeNull();
  });
});
