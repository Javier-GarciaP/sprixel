import { execFileSync, spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { afterEach, beforeEach, describe, expect, it } from "vitest";

// A tiny repository in a temporary folder: a commit on main, and a commit
// that exists only on another branch. The real repository is never touched.
const script = path.resolve("scripts/check-tag-branch.mjs");
let repo;
const git = (...args) =>
  execFileSync("git", args, { cwd: repo, encoding: "utf8" }).trim();
const commit = (name) => {
  fs.writeFileSync(path.join(repo, name), name);
  git("add", name);
  git("-c", "user.name=t", "-c", "user.email=t@t", "commit", "-q", "-m", name);
  return git("rev-parse", "HEAD");
};
const check = (...args) =>
  spawnSync("node", [script, ...args], { cwd: repo, encoding: "utf8" });

let onMain, offMain;
beforeEach(() => {
  repo = fs.mkdtempSync(path.join(os.tmpdir(), "sprixel-branch-"));
  git("init", "-q", "-b", "main");
  onMain = commit("a");
  git("switch", "-q", "-c", "feature/x");
  offMain = commit("b");
  git("switch", "-q", "main");
});
afterEach(() => fs.rmSync(repo, { recursive: true, force: true }));

describe("check-tag-branch", () => {
  it("passes for a commit on main", () => {
    expect(check(onMain, "main").status).toBe(0);
  });

  it("passes for the tip of main and for older commits of main", () => {
    const tip = commit("c");

    expect(check(tip, "main").status).toBe(0);
    expect(check(onMain, "main").status).toBe(0);
  });

  it("fails for a commit that is not on main, and says releases come from main", () => {
    const r = check(offMain, "main");

    expect(r.status).toBe(1);
    expect(r.stderr).toContain("Releases come from main");
  });

  it("fails clearly when main cannot be found", () => {
    const r = check(onMain, "origin/main");

    expect(r.status).toBe(1);
    expect(r.stderr).toContain("cannot find origin/main");
  });
});
