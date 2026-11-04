import fs from "node:fs";
import js from "@eslint/js";
import tseslint from "typescript-eslint";

// Every folder in src/features is a feature. Read them from disk so the list
// never goes out of date.
const features = fs
  .readdirSync("src/features", { withFileTypes: true })
  .filter((entry) => entry.isDirectory())
  .map((entry) => entry.name);

// Relative paths can climb a few folders: "../x", "../../x", ...
const climbs = [1, 2, 3, 4, 5].map((depth) => "../".repeat(depth));

const message = (text) => ({ message: text });

// A feature may only use `shared/` or the index of another feature.
const featureRules = features.map((feature) => {
  const others = features.filter((other) => other !== feature);
  return {
    files: [`src/features/${feature}/**/*.{ts,tsx}`],
    rules: {
      "no-restricted-imports": [
        "error",
        {
          patterns: others.map((other) => ({
            group: [
              `@/features/${other}/**`,
              ...climbs.map((up) => `${up}${other}/**`),
            ],
            ...message(
              `Do not import inside the "${other}" feature. Use "@/features/${other}" (its index.ts).`,
            ),
          })),
        },
      ],
    },
  };
});

export default tseslint.config(
  { ignores: ["dist", "src-tauri", "cli", "node_modules"] },
  js.configs.recommended,
  ...tseslint.configs.recommended,
  ...featureRules,
  {
    // Node scripts and config files run in Node, not in the browser.
    files: ["scripts/**/*.mjs", "*.js"],
    languageOptions: {
      globals: { process: "readonly", console: "readonly" },
    },
  },
  {
    // `shared/` must not depend on any feature.
    files: ["src/shared/**/*.{ts,tsx}"],
    rules: {
      "no-restricted-imports": [
        "error",
        {
          patterns: [
            {
              group: [
                "@/features/**",
                ...climbs.map((up) => `${up}features/**`),
              ],
              ...message("`shared/` must not import from `features/`."),
            },
          ],
        },
      ],
    },
  },
);
