import { spawnSync } from "node:child_process";
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

  // Only the job that creates the release may write to the repository.
  it("gives contents: write to the release job only", () => {
    expect(text.match(/contents: write/g)).toHaveLength(1);
    const release = text.slice(text.indexOf("\n  release:"));
    expect(release).toContain("contents: write");
  });

  it("creates the release only after every build job passed", () => {
    const release = text.slice(text.indexOf("\n  release:"));

    expect(release).toMatch(/needs: \[verify, build, smoke\]/);
  });

  it("marks a tag with a suffix as a pre-release", () => {
    const snippet = text.match(/case "\$TAG" in .*? esac/)[0];
    const flag = (tag) =>
      spawnSync("sh", ["-c", `TAG=${tag}; ${snippet}; echo "[$PRERELEASE]"`], {
        encoding: "utf8",
      }).stdout.trim();

    expect(flag("v0.3.0-alpha.1")).toBe("[--prerelease]");
    expect(flag("v1.0.0-rc.2")).toBe("[--prerelease]");
    expect(flag("v1.0.0")).toBe("[]");
  });

  it("takes the notes from the changelog and checks the tag exists", () => {
    expect(text).toContain('node scripts/release-notes.mjs "$TAG"');
    expect(text).toContain("--verify-tag");
  });

  // Scenarios "Tag on another branch" and "Wrong tag": the checks run first and
  // nothing is built if they fail.
  it("checks the branch and the tag before anything is built", () => {
    const verify = text.slice(
      text.indexOf("\n  verify:"),
      text.indexOf("\n  build:"),
    );

    expect(verify).toContain("scripts/check-tag-branch.mjs");
    expect(verify).toContain("scripts/bump-version.mjs --tag");
    expect(verify).toContain("fetch-depth: 0");
    const build = text.slice(
      text.indexOf("\n  build:"),
      text.indexOf("\n  smoke:"),
    );
    expect(build).toContain("needs: verify");
  });

  // Scenario "Smoke test": each system installs and runs sprixel version.
  it("installs the release on every system and runs the smoke test", () => {
    const smoke = text.slice(
      text.indexOf("\n  smoke:"),
      text.indexOf("\n  release:"),
    );

    expect(smoke).toContain("install.sh");
    expect(smoke).toContain("install.ps1");
    expect(smoke).toContain("scripts/smoke-test.mjs");
    for (const runner of [
      "ubuntu-latest",
      "macos-latest",
      "macos-15-intel",
      "windows-latest",
    ]) {
      expect(smoke).toContain(`runner: ${runner}`);
    }
  });
});
