import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

const css = readFileSync("src/app/index.css", "utf8");

// Returns the color variables declared inside the block with this selector.
function tokensOf(selector: string): string[] {
  const start = css.indexOf(`${selector} {`);
  const end = css.indexOf("}", start);
  const block = css.slice(start, end);
  return [...block.matchAll(/(--color-[\w-]+):/g)].map((m) => m[1]).sort();
}

describe("design tokens", () => {
  // Checks that the light theme defines every color the dark theme defines,
  // so no token is left without a value in one theme.
  it("has the same color tokens in dark and light", () => {
    const dark = tokensOf('[data-theme="dark"]');
    const light = tokensOf('[data-theme="light"]');

    expect(dark.length).toBeGreaterThan(5);
    expect(light).toEqual(dark);
  });

  // Checks that the three design fonts are mapped and bundled offline
  // (the CSS imports the local @fontsource packages, not a web URL).
  it("maps the three fonts and bundles them", () => {
    expect(css).toContain('--font-title: "Space Grotesk"');
    expect(css).toContain('--font-sans: "Inter"');
    expect(css).toContain('--font-mono: "DM Mono"');
    expect(css).toContain("@fontsource/space-grotesk");
    expect(css).toContain("@fontsource/inter");
    expect(css).toContain("@fontsource/dm-mono");
    expect(css).not.toContain("http");
  });
});
