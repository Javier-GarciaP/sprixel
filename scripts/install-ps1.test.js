import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import process from "node:process";
import { pathToFileURL } from "node:url";
import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { makeRelease } from "./release-fixture.mjs";

const script = path.resolve("install.ps1");
const text = fs.readFileSync(script, "utf8");

// PowerShell is not installed on every machine (the CI runners of the three
// systems have it). The real runs below are skipped when `pwsh` is missing;
// the static checks always run.
const pwsh = ["pwsh", "powershell"].find(
  (name) => spawnSync(name, ["-NoProfile", "-Command", "1"]).status === 0,
);

describe("install.ps1 (static checks)", () => {
  it("verifies the SHA-256 of every file before it installs", () => {
    expect(text).toContain("Get-FileHash -Algorithm SHA256");
    expect(text.indexOf("does not match")).toBeLessThan(
      text.indexOf("# 4. Install"),
    );
  });

  it("prints the MIT notice and the source URL first", () => {
    expect(text.indexOf("MIT License")).toBeLessThan(
      text.indexOf("Downloading"),
    );
    expect(text).toContain("Source code: $source");
    expect(text).toContain("Javier-GarciaP/sprixel");
  });

  it("installs only for the user and never asks for administrator rights", () => {
    expect(text).toContain("$env:LOCALAPPDATA");
    expect(text).toContain('"User"');
    expect(text).not.toMatch(/RunAs|"Machine"|HKLM|ProgramFiles/i);
  });

  it("does not exit when run with iex (it would close the terminal)", () => {
    expect(text).toContain("$MyInvocation.MyCommand.Path");
  });

  it("uses the same file names as the release", () => {
    expect(text).toContain("sprixel-windows-x64.exe");
    expect(text).toContain("Sprixel-windows-x64.zip");
  });
});

// PowerShell starts slowly on a busy runner (one run took 6.5 s): 5 s is too short.
describe.skipIf(!pwsh)(
  "install.ps1 (run with PowerShell)",
  { timeout: 30000 },
  () => {
    let root, local, release;
    beforeEach(() => {
      root = fs.mkdtempSync(path.join(os.tmpdir(), "sprixel-ps1-"));
      local = path.join(root, "local");
      release = path.join(root, "release");
      fs.mkdirSync(local);
    });
    afterEach(() => fs.rmSync(root, { recursive: true, force: true }));

    const install = (env = {}) =>
      spawnSync(pwsh, ["-NoProfile", "-File", script], {
        encoding: "utf8",
        env: {
          ...process.env,
          LOCALAPPDATA: local,
          SPRIXEL_BASE_URL: pathToFileURL(release).href,
          SPRIXEL_SKIP_PATH: "1",
          // The script reads the CPU from a Windows variable: say it is x64, so
          // the same run works on Linux and macOS runners too.
          SPRIXEL_ARCH: "AMD64",
          ...env,
        },
      });

    it("installs the app, the command and the notices", () => {
      makeRelease(release);
      const r = install();

      expect(r.status, r.stderr + r.stdout).toBe(0);
      expect(fs.existsSync(path.join(local, "Sprixel/bin/sprixel.exe"))).toBe(
        true,
      );
      expect(fs.existsSync(path.join(local, "Sprixel/Sprixel.exe"))).toBe(true);
      expect(fs.existsSync(path.join(local, "Sprixel/LICENSE"))).toBe(true);
    });

    it("stops and installs nothing when a checksum is wrong", () => {
      makeRelease(release, { wrongChecksum: "Sprixel-windows-x64.zip" });
      const r = install();

      expect(r.status).toBe(1);
      expect(fs.existsSync(path.join(local, "Sprixel"))).toBe(false);
    });

    it("reinstalling upgrades and keeps the user's projects and settings", () => {
      makeRelease(release, { version: "0.2.0" });
      install();
      const project = path.join(root, "home/Sprites/hero.txt");
      const token = path.join(root, "home/.sprixel/token");
      fs.mkdirSync(path.dirname(project), { recursive: true });
      fs.mkdirSync(path.dirname(token), { recursive: true });
      fs.writeFileSync(project, "pixels\n");
      fs.writeFileSync(token, "secret\n");

      makeRelease(release, { version: "0.3.0" });
      const r = install({ USERPROFILE: path.join(root, "home") });

      expect(r.status, r.stderr + r.stdout).toBe(0);
      expect(r.stdout).toContain("Sprixel 0.3.0 is installed");
      expect(fs.readFileSync(project, "utf8")).toBe("pixels\n");
      expect(fs.readFileSync(token, "utf8")).toBe("secret\n");
    });

    it("stops on an unsupported CPU", () => {
      makeRelease(release);
      const r = install({ SPRIXEL_ARCH: "ARM64" });

      expect(r.status).toBe(1);
      expect(fs.existsSync(path.join(local, "Sprixel"))).toBe(false);
    });
  },
);
