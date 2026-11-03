import { execFileSync, spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import process from "node:process";
import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { assemble } from "./assemble-release.mjs";
import { TARGETS, allFiles, builtFiles } from "./release-files.mjs";

// Fake build output: one folder per job, like actions/download-artifact.
let work, artifacts, out;
beforeEach(() => {
  work = fs.mkdtempSync(path.join(os.tmpdir(), "sprixel-assemble-"));
  artifacts = path.join(work, "artifacts");
  out = path.join(work, "release");
  for (const target of TARGETS) {
    const dir = path.join(artifacts, `release-${target.os}-${target.arch}`);
    fs.mkdirSync(dir, { recursive: true });
    for (const name of builtFiles(target)) {
      fs.writeFileSync(path.join(dir, name), `#!/bin/sh\necho ${name}\n`);
    }
  }
});
afterEach(() => fs.rmSync(work, { recursive: true, force: true }));

describe("assemble-release", () => {
  it("the release lists the files for the three systems and the scripts", () => {
    assemble(process.cwd(), artifacts, out);
    const names = fs.readdirSync(out);

    for (const name of [...allFiles(), "checksums.txt"]) {
      expect(names, name).toContain(name);
    }
    expect(names).toHaveLength(allFiles().length + 1);
  });

  it("VERSION holds the version of package.json", () => {
    assemble(process.cwd(), artifacts, out);
    const { version } = JSON.parse(fs.readFileSync("package.json", "utf8"));

    expect(fs.readFileSync(path.join(out, "VERSION"), "utf8")).toBe(
      `${version}\n`,
    );
  });

  it("checksums.txt is checked by sha256sum", () => {
    assemble(process.cwd(), artifacts, out);

    const check = spawnSync("sha256sum", ["-c", "checksums.txt"], {
      cwd: out,
      encoding: "utf8",
    });
    expect(check.status, check.stdout + check.stderr).toBe(0);
  });

  it("checksums.txt lists every file but itself", () => {
    assemble(process.cwd(), artifacts, out);
    const listed = fs
      .readFileSync(path.join(out, "checksums.txt"), "utf8")
      .trim()
      .split("\n")
      .map((l) => l.split("  ")[1]);

    expect(listed.sort()).toEqual(allFiles().sort());
  });

  it("fails and names the file when a system was not built", () => {
    fs.rmSync(path.join(artifacts, "release-windows-x64"), { recursive: true });

    expect(() => assemble(process.cwd(), artifacts, out)).toThrow(
      /Sprixel-windows-x64\.zip/,
    );
    expect(fs.existsSync(path.join(out, "checksums.txt"))).toBe(false);
  });

  // The installer and the pipeline must agree on names and checksums: install
  // from the assembled folder, with a temporary HOME.
  it("install.sh installs the assembled release", () => {
    assemble(process.cwd(), artifacts, out);
    const home = path.join(work, "home");
    fs.mkdirSync(home);

    const r = spawnSync("sh", [path.resolve("install.sh")], {
      encoding: "utf8",
      env: {
        PATH: process.env.PATH,
        HOME: home,
        SHELL: "/bin/bash",
        SPRIXEL_BASE_URL: `file://${out}`,
        SPRIXEL_OS: "Linux",
        SPRIXEL_ARCH: "x86_64",
      },
    });

    expect(r.status, r.stderr).toBe(0);
    expect(
      execFileSync(path.join(home, ".local/bin/sprixel"), { encoding: "utf8" }),
    ).toContain("sprixel-linux-x64");
  });
});
