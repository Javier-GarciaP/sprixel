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

  // Checks the scenario "Shortcut after the install script": the script asks
  // the new app to make its Start Menu shortcut, after it is copied, waits for
  // it with a limit, and treats a problem as a warning.
  it("asks the app to register its Start Menu shortcut after copying it", () => {
    expect(text).toContain("--register-menu");
    expect(text.indexOf("--register-menu")).toBeGreaterThan(
      text.indexOf('Put-File (Join-Path $unpacked "Sprixel.exe")'),
    );
    expect(text).toContain("WaitForExit(30000)");
    expect(text).toContain("could not be added to the Start Menu");
  });

  // Checks the "Confirmation" and "No terminal, no questions" requirements:
  // the questions are behind a terminal check, and SPRIXEL_YES skips them.
  it("asks only on a terminal and SPRIXEL_YES skips the questions", () => {
    expect(text).toContain("Install Sprixel? [Y/n]");
    expect(text).toContain("[Console]::IsInputRedirected");
    expect(text).toContain("$env:SPRIXEL_YES");
    expect(text).toContain("Nothing was installed.");
  });

  // Checks "Visible download progress": curl.exe draws the bar, and without a
  // terminal each file is one plain counted line.
  it("shows the progress of each download", () => {
    expect(text).toContain("curl.exe");
    expect(text).toContain("--progress-bar");
    expect(text).toContain("/$total]");
    expect(text).toContain('$ProgressPreference = "SilentlyContinue"');
  });

  // Checks that the answer "no" is kept in the file that the app reads.
  it("keeps a no to the Start Menu in .sprixel\\no-menu-entry", () => {
    expect(text).toContain("no-menu-entry");
    expect(text).toContain("$env:SPRIXEL_MENU");
  });

  it("connects the chosen AI tools by the full path of the command", () => {
    expect(text).toContain('(Join-Path $bin "sprixel.exe") mcp add');
    expect(text).toContain("could not connect");
    expect(text).toContain("$env:SPRIXEL_CONNECT");
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
      fs.mkdirSync(path.join(root, "home"));
    });
    afterEach(() => fs.rmSync(root, { recursive: true, force: true }));

    const install = (env = {}) =>
      spawnSync(pwsh, ["-NoProfile", "-File", script], {
        encoding: "utf8",
        env: {
          ...process.env,
          LOCALAPPDATA: local,
          // A home folder of the test: the script writes and removes a file in
          // it, and the real one must never be touched.
          USERPROFILE: path.join(root, "home"),
          HOME: path.join(root, "home"),
          // No terminal unless a test gives one.
          SPRIXEL_TTY: "none",
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

    // The fake Sprixel.exe of the fixture cannot run, like an app that cannot
    // start: the shortcut step fails, and the install must still end well.
    it("still installs when the app cannot register its shortcut", () => {
      makeRelease(release);
      const r = install();

      expect(r.status, r.stderr + r.stdout).toBe(0);
      expect(r.stdout).toContain("could not be added to the Start Menu");
      expect(fs.existsSync(path.join(local, "Sprixel/Sprixel.exe"))).toBe(true);
    });

    // A terminal for a test: the answers the person would type.
    const answers = (...lines) => {
      const tty = path.join(root, "tty.txt");
      fs.writeFileSync(tty, lines.join("\n") + "\n");
      return { SPRIXEL_TTY: tty };
    };
    const noMenuFile = () =>
      path.join(root, "home", ".sprixel", "no-menu-entry");

    it("prints the summary before it downloads anything", () => {
      makeRelease(release);
      const out = install().stdout;

      expect(out).toContain("This will install:");
      expect(out).toContain("System    windows x64");
      expect(out.indexOf("System    windows x64")).toBeLessThan(
        out.indexOf("[1/6]"),
      );
    });

    it("asks nothing when there is no terminal", () => {
      makeRelease(release);
      const r = install();

      expect(r.status, r.stderr + r.stdout).toBe(0);
      expect(r.stdout).not.toContain("[Y/n]");
    });

    it("asks for confirmation on a terminal and Enter accepts", () => {
      makeRelease(release);
      const r = install(answers("", ""));

      expect(r.status, r.stderr + r.stdout).toBe(0);
      expect(r.stdout).toContain("Install Sprixel? [Y/n]");
      expect(fs.existsSync(path.join(local, "Sprixel/Sprixel.exe"))).toBe(true);
    });

    it("installs nothing when the answer is no", () => {
      makeRelease(release);
      const r = install(answers("n"));

      expect(r.status, r.stderr + r.stdout).toBe(0);
      expect(r.stdout).toContain("Nothing was installed.");
      expect(fs.existsSync(path.join(local, "Sprixel"))).toBe(false);
    });

    it("SPRIXEL_YES skips the questions even on a terminal", () => {
      makeRelease(release);
      const r = install({ ...answers("n", "n"), SPRIXEL_YES: "1" });

      expect(r.status, r.stderr + r.stdout).toBe(0);
      expect(fs.existsSync(path.join(local, "Sprixel/Sprixel.exe"))).toBe(true);
    });

    it("prints one counted line for each file when it is not a terminal", () => {
      makeRelease(release);
      const out = install().stdout;

      expect(out).toContain("[1/6] checksums.txt ... done");
      expect(out).toContain("[6/6] THIRD-PARTY.md ... done");
    });

    it("keeps a no to the Start Menu in no-menu-entry", () => {
      makeRelease(release);
      const r = install({ SPRIXEL_MENU: "0" });

      expect(r.status, r.stderr + r.stdout).toBe(0);
      expect(fs.existsSync(noMenuFile())).toBe(true);
    });

    it("an install that says yes removes an old no", () => {
      makeRelease(release);
      fs.mkdirSync(path.dirname(noMenuFile()), { recursive: true });
      fs.writeFileSync(noMenuFile(), "");

      install();

      expect(fs.existsSync(noMenuFile())).toBe(false);
    });

    // The fake command of the fixture cannot run on Windows (it is text), so a
    // chosen AI tool fails to connect: that must be a warning only.
    it("a failing AI connection is a warning and the install succeeds", () => {
      makeRelease(release);
      const r = install({ SPRIXEL_CONNECT: "claude" });

      expect(r.status, r.stderr + r.stdout).toBe(0);
      expect(r.stdout).toContain("could not connect Claude Code");
    });

    it("rejects a SPRIXEL_CONNECT value it does not know", () => {
      makeRelease(release);
      const r = install({ SPRIXEL_CONNECT: "vim" });

      expect(r.status).toBe(1);
      expect(r.stdout + r.stderr).toContain("SPRIXEL_CONNECT must be");
      expect(fs.existsSync(path.join(local, "Sprixel"))).toBe(false);
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
