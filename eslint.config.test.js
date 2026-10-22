import { ESLint } from "eslint";
import { describe, expect, it } from "vitest";

// Lints a piece of code as if it lived in `filePath` and returns the messages.
async function lint(code, filePath) {
  const eslint = new ESLint();
  const [result] = await eslint.lintText(code, { filePath });
  return result.messages;
}

describe("import boundaries", () => {
  it("rejects a feature importing inside another feature", async () => {
    const messages = await lint(
      'import { Tool } from "@/features/palette-editor/components/Tool";\nexport const x = Tool;\n',
      "src/features/canvas-editor/components/Canvas.tsx",
    );

    expect(messages.map((m) => m.ruleId)).toContain("no-restricted-imports");
  });

  it("rejects a relative import into another feature", async () => {
    const messages = await lint(
      'import { Tool } from "../../palette-editor/components/Tool";\nexport const x = Tool;\n',
      "src/features/canvas-editor/components/Canvas.tsx",
    );

    expect(messages.map((m) => m.ruleId)).toContain("no-restricted-imports");
  });

  it("allows a feature to use the index of another feature", async () => {
    const messages = await lint(
      'import { Tool } from "@/features/palette-editor";\nexport const x = Tool;\n',
      "src/features/canvas-editor/components/Canvas.tsx",
    );

    expect(messages).toEqual([]);
  });

  it("allows a feature to import its own files", async () => {
    const messages = await lint(
      'import { Tool } from "../hooks/useTool";\nexport const x = Tool;\n',
      "src/features/canvas-editor/components/Canvas.tsx",
    );

    expect(messages).toEqual([]);
  });

  it("rejects shared importing from features", async () => {
    const messages = await lint(
      'import { Tool } from "@/features/palette-editor";\nexport const x = Tool;\n',
      "src/shared/lib/colors.ts",
    );

    expect(messages.map((m) => m.ruleId)).toContain("no-restricted-imports");
  });
});
