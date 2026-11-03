import process from "node:process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { installedCommand, smoke } from "./smoke-test.mjs";

// A fake `sprixel` (a shell script) that prints what the real one would.
let dir;
beforeEach(() => {
  dir = fs.mkdtempSync(path.join(os.tmpdir(), "sprixel-smoke-"));
});
afterEach(() => fs.rmSync(dir, { recursive: true, force: true }));

function fake({ version = "0.2.0", installation = "ok" } = {}) {
  const command = path.join(dir, "sprixel");
  const doctor = JSON.stringify({
    ok: false,
    checks: [
      { name: "installation", status: installation, detail: "the app is at x" },
      { name: "app", status: "fail", detail: "the app is not running" },
    ],
  });
  fs.writeFileSync(
    command,
    `#!/bin/sh\ncase "$1" in version) echo ${version};; doctor) echo '${doctor}'; exit 1;; esac\n`,
    { mode: 0o755 },
  );
  return command;
}

describe.skipIf(process.platform === "win32")("smoke-test", () => {
  it("passes when the version matches and the app is found", () => {
    expect(smoke(fake(), "0.2.0")).toEqual([]);
  });

  it("fails when sprixel version shows another version", () => {
    const problems = smoke(fake({ version: "0.1.0" }), "0.2.0");

    expect(problems).toHaveLength(1);
    expect(problems[0]).toContain('expected "0.2.0"');
  });

  it("fails when doctor does not find the app", () => {
    const problems = smoke(fake({ installation: "fail" }), "0.2.0");

    expect(problems[0]).toContain("installation check is fail");
  });

  it("fails when the command is not installed", () => {
    expect(smoke(path.join(dir, "missing"), "0.2.0")[0]).toContain(
      "cannot run",
    );
  });
});

describe("installedCommand", () => {
  it("is in ~/.local/bin on Linux and macOS", () => {
    expect(installedCommand({ HOME: "/h" }, "linux")).toBe(
      path.join("/h", ".local", "bin", "sprixel"),
    );
  });

  it("is in %LOCALAPPDATA%\\Sprixel\\bin on Windows", () => {
    expect(installedCommand({ LOCALAPPDATA: "C:\\L" }, "win32")).toBe(
      path.join("C:\\L", "Sprixel", "bin", "sprixel.exe"),
    );
  });
});
