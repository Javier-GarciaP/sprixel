import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import en from "../src/shared/lib/i18n/en.json";
import { commands } from "../src/app/commands/registry.ts";
import { formatShortcut } from "../src/app/commands/shortcut.ts";

const usage = readFileSync("docs/usage.md", "utf8");
const section = usage.slice(
  usage.indexOf("## Keyboard shortcuts"),
  usage.indexOf("## The `sprixel` command"),
);

describe("docs/usage.md", () => {
  // Checks that every shortcut of the app is in the shortcuts section of the
  // guide, with the same name and the same keys (Ctrl form).
  it("lists every shortcut of the command registry", () => {
    for (const command of commands.filter((c) => c.shortcut)) {
      const row = `| ${en[command.labelKey]} | \`${formatShortcut(command.shortcut, "other")}\` |`;
      expect(section, row).toContain(row);
    }
  });

  // Checks that the guide has no shortcut row that the app does not have.
  it("has no shortcut that the app does not have", () => {
    const rows = section
      .split("\n")
      .filter((l) => /^\| .+ \| `.+` \|$/.test(l));
    const expected = commands
      .filter((c) => c.shortcut)
      .map(
        (c) =>
          `| ${en[c.labelKey]} | \`${formatShortcut(c.shortcut, "other")}\` |`,
      );
    expect(rows.sort()).toEqual(expected.sort());
  });
});
