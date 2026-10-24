import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import en from "../src/shared/lib/i18n/en.json";
import { commands } from "../src/app/commands/registry.ts";
import { formatShortcut } from "../src/app/commands/shortcut.ts";
import { tools } from "../src/features/canvas-editor/tools/tools.ts";
import {
  MAX_ZOOM,
  MIN_ZOOM,
} from "../src/features/canvas-editor/view/viewMath.ts";
import { GRID_MIN_ZOOM } from "../src/features/canvas-editor/render/renderer.ts";
import {
  DEFAULT_OPACITY,
  MAX_OPACITY,
  MIN_OPACITY,
} from "../src/features/canvas-editor/components/OnionPanel.tsx";

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

const draw = usage.slice(
  usage.indexOf("## Draw"),
  usage.indexOf("## Frames and animations"),
);

describe("docs/usage.md, Draw", () => {
  // Checks that the table of tools has every tool of the editor with its key
  // (the guide may add a few words after the name, like "Bucket (fill)").
  it("lists every tool with its key", () => {
    for (const tool of tools) {
      const label = en[tool.labelKey];
      const row = new RegExp(
        "^\\| " + label + "( \\(.*\\))? \\| `" + tool.key + "` \\|$",
        "m",
      );
      expect(draw, label).toMatch(row);
    }
    const rows = draw.split("\n").filter((l) => /^\| .+ \| `.+` \|$/.test(l));
    expect(rows).toHaveLength(tools.length);
  });

  // Checks the numbers of the guide against the numbers of the code: the zoom
  // range, the zoom where the grid starts and the onion skin opacity.
  it("gives the zoom and onion skin limits of the app", () => {
    expect(draw).toContain(`from ${MIN_ZOOM * 100}% to ${MAX_ZOOM * 100}%`);
    expect(draw).toContain(`from ${GRID_MIN_ZOOM * 100}% zoom`);
    expect(usage).toContain(`(${MIN_OPACITY}% to ${MAX_OPACITY}%,`);
    expect(usage).toContain(`${DEFAULT_OPACITY}% at the start`);
  });

  // Checks that the texts the guide quotes are the real texts of the app.
  it("quotes the real messages", () => {
    expect(draw).toContain(`"${en["colors.paletteFull"]}"`);
    expect(draw).toContain(
      `"${en["marker.drawing"].replace("{agent}", "Claude Code").replace("{frame}", "2")}"`,
    );
    expect(draw).toContain(`**${en["marker.stop"]}**`);
  });
});
