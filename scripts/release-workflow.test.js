import fs from "node:fs";
import { describe, expect, it } from "vitest";

// The workflow cannot run here (it needs GitHub runners), so these tests read
// it as text and check the rules that matter for safety, and that every
// script it calls exists.
const text = fs.readFileSync(".github/workflows/release.yml", "utf8");

describe("release workflow", () => {
  it("starts only on version tags", () => {
    expect(text).toMatch(/on:\n {2}push:\n {4}tags: \["v\*"\]/);
    expect(text).not.toMatch(/pull_request|workflow_dispatch|branches:/);
  });

  it("reads the code only by default", () => {
    expect(text).toMatch(/\npermissions:\n {2}contents: read\n/);
  });

  it("uses only GITHUB_TOKEN as a secret", () => {
    const secrets = [...text.matchAll(/secrets\.(\w+)/g)].map((m) => m[1]);

    expect(secrets.filter((s) => s !== "GITHUB_TOKEN")).toEqual([]);
  });

  it("pins every action to a major version of an official organization", () => {
    const uses = [...text.matchAll(/uses: (\S+)/g)].map((m) => m[1]);

    expect(uses.length).toBeGreaterThan(0);
    for (const use of uses) {
      expect(use, use).toMatch(/^(actions|tauri-apps)\/[\w-]+@v\d+$/);
    }
  });

  it("calls only scripts and files that exist", () => {
    const paths = [
      ...text.matchAll(
        /(?:node |bash |sh |\.\/)((?:scripts\/|install\.)[\w./-]+)/g,
      ),
    ].map((m) => m[1]);

    for (const file of paths) {
      expect(fs.existsSync(file), file).toBe(true);
    }
  });

  it("builds every system of the release", () => {
    for (const row of [
      "ubuntu-latest",
      "macos-latest",
      "macos-15-intel",
      "windows-latest",
    ]) {
      expect(text).toContain(`runner: ${row}`);
    }
  });

  it("runs the tests before the app is built", () => {
    expect(text.indexOf("npm test")).toBeLessThan(text.indexOf("tauri build"));
  });
});
