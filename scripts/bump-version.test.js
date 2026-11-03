import { execFileSync, spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { afterEach, beforeEach, describe, expect, it } from "vitest";

// Every file that holds the version, relative to the repository root.
const FILES = [
  "package.json",
  "package-lock.json",
  "src-tauri/tauri.conf.json",
  "src-tauri/Cargo.toml",
  "src-tauri/Cargo.lock",
  "cli/internal/version/version.go",
];
const script = path.resolve("scripts/bump-version.mjs");

// The tests never touch the real repository: they work on a copy of the
// files that hold the version, in a temporary folder.
let copy;
beforeEach(() => {
  copy = fs.mkdtempSync(path.join(os.tmpdir(), "sprixel-bump-"));
  for (const file of FILES) {
    fs.mkdirSync(path.dirname(path.join(copy, file)), { recursive: true });
    fs.copyFileSync(file, path.join(copy, file));
  }
});
afterEach(() => fs.rmSync(copy, { recursive: true, force: true }));

const run = (...args) =>
  spawnSync("node", [script, ...args], { cwd: copy, encoding: "utf8" });
const read = (file) => fs.readFileSync(path.join(copy, file), "utf8");

describe("bump-version", () => {
  it("the real repository passes the check", () => {
    const real = execFileSync("node", [script, "--check"], {
      encoding: "utf8",
    });

    expect(real).toContain("match");
  });

  it("bumps every version file to 0.2.0 and the check passes", () => {
    const bump = run("0.2.0");
    expect(bump.status).toBe(0);

    expect(JSON.parse(read("package.json")).version).toBe("0.2.0");
    const lock = JSON.parse(read("package-lock.json"));
    expect(lock.version).toBe("0.2.0");
    expect(lock.packages[""].version).toBe("0.2.0");
    expect(read("src-tauri/Cargo.toml")).toMatch(/^version = "0\.2\.0"$/m);
    expect(read("src-tauri/Cargo.lock")).toMatch(
      /name = "sprixel"\nversion = "0\.2\.0"/,
    );
    expect(read("cli/internal/version/version.go")).toContain(
      'Version = "0.2.0"',
    );
    expect(JSON.parse(read("src-tauri/tauri.conf.json")).version).toBe(
      "../package.json",
    );
    expect(run("--check").status).toBe(0);
  });

  it("changes only the version lines", () => {
    const before = read("src-tauri/Cargo.toml");
    run("0.2.0");

    expect(read("src-tauri/Cargo.toml")).toBe(
      before.replace('version = "0.1.0"', 'version = "0.2.0"'),
    );
  });

  it("accepts a pre-release version", () => {
    expect(run("0.3.0-alpha.1").status).toBe(0);
    expect(JSON.parse(read("package.json")).version).toBe("0.3.0-alpha.1");
  });

  it("rejects a version that is not semver and changes nothing", () => {
    const before = read("package.json");
    const bad = run("1.2");

    expect(bad.status).toBe(2);
    expect(bad.stderr).toContain("MAJOR.MINOR.PATCH");
    expect(read("package.json")).toBe(before);
  });

  it("the check fails when one file disagrees", () => {
    const toml = read("src-tauri/Cargo.toml").replace("0.1.0", "0.9.9");
    fs.writeFileSync(path.join(copy, "src-tauri/Cargo.toml"), toml);
    const check = run("--check");

    expect(check.status).toBe(1);
    expect(check.stderr).toContain("src-tauri/Cargo.toml");
  });

  // Checks the scenario "Wrong tag": the tag says one version and the code
  // another, so the release pipeline must stop before building.
  it("--tag passes when the tag is v plus the code version", () => {
    expect(run("--tag", "v0.1.0").status).toBe(0);
  });

  it("--tag fails when the tag and the code version differ", () => {
    const wrong = run("--tag", "v1.2.0");

    expect(wrong.status).toBe(1);
    expect(wrong.stderr).toContain("v1.2.0");
    expect(wrong.stderr).toContain("0.1.0");
  });

  it("--tag fails when the tag is not in the vMAJOR.MINOR.PATCH form", () => {
    expect(run("--tag", "0.1.0").status).toBe(1);
    expect(run("--tag", "release-1").status).toBe(1);
  });

  it("--tag also fails when the version files disagree", () => {
    const toml = read("src-tauri/Cargo.toml").replace("0.1.0", "0.9.9");
    fs.writeFileSync(path.join(copy, "src-tauri/Cargo.toml"), toml);

    expect(run("--tag", "v0.1.0").status).toBe(1);
  });

  it("--tag fails when the tag has a bad suffix, even if the code agrees", () => {
    const pkg = JSON.parse(read("package.json"));
    pkg.version = "0.1.0-foo";
    fs.writeFileSync(path.join(copy, "package.json"), JSON.stringify(pkg));

    const bad = run("--tag", "v0.1.0-foo");

    expect(bad.status).toBe(1);
    expect(bad.stderr).toContain("vMAJOR.MINOR.PATCH");
  });
});
