import fs from "node:fs";
import { describe, expect, it } from "vitest";
import { SHARED_FILES } from "./release-files.mjs";

// The install commands in the documentation must match the real scripts. If a
// script changes (a flag, a folder, a file name), these tests point at the
// page to fix.
const read = (file) => fs.readFileSync(file, "utf8");
const readme = read("README.md");
// Prettier may wrap a line of the docs, so spaces and line breaks are equal.
const usage = read("docs/usage.md").replace(/\s+/g, " ");
const contributing = read("CONTRIBUTING.md");
const sh = read("install.sh");
const ps1 = read("install.ps1");
const updateGo = read("cli/internal/command/update.go");
const uninstallGo = read("cli/internal/command/uninstall.go");

describe("install documentation", () => {
  it("the README commands download scripts that the release publishes", () => {
    const urls = [
      ...readme.matchAll(/releases\/latest\/download\/([\w.-]+)/g),
    ].map((m) => m[1]);

    expect(urls.length).toBeGreaterThan(0);
    for (const name of urls) {
      expect(SHARED_FILES, name).toContain(name);
    }
  });

  it("the README repository URL is the one the scripts use", () => {
    expect(sh).toContain('REPO="Javier-GarciaP/sprixel"');
    expect(ps1).toContain('$repo = "Javier-GarciaP/sprixel"');
    expect(readme).toContain(
      "github.com/Javier-GarciaP/sprixel/releases/latest/download/install.sh",
    );
  });

  it("the documented way to pick a version exists in both scripts", () => {
    expect(readme).toContain("sh -s -- --version 0.2.0");
    expect(sh).toContain("--version)");
    expect(readme).toContain('$env:SPRIXEL_VERSION = "0.2.0"');
    expect(ps1).toContain("$env:SPRIXEL_VERSION");
  });

  it("the documented folders are the ones the scripts and the command use", () => {
    expect(usage).toContain("~/.local/bin/sprixel-app");
    expect(sh).toContain('"$BIN/sprixel-app"');
    expect(usage).toContain("~/Applications/Sprixel.app");
    expect(sh).toContain('"$HOME/Applications/Sprixel.app"');
    expect(usage).toContain("%LOCALAPPDATA%\\Sprixel\\bin\\sprixel.exe");
    expect(ps1).toContain('Join-Path $home_ "bin"');
    expect(read("cli/internal/release/release.go")).toContain(
      '"bin", "sprixel.exe"',
    );
  });

  it("the documented update and uninstall flags are implemented", () => {
    expect(usage).toContain("sprixel update --check");
    expect(updateGo).toContain('"--check"');
    for (const flag of ["--yes", "--remove-data"]) {
      expect(usage, flag).toContain(flag);
      expect(uninstallGo, flag).toContain(`"${flag}"`);
    }
  });

  it("the PATH marker in the docs is the one install.sh writes", () => {
    expect(usage).toContain("# Added by the Sprixel installer");
    expect(sh).toContain("# Added by the Sprixel installer");
    expect(read("cli/internal/release/uninstall.go")).toContain(
      "# Added by the Sprixel installer",
    );
  });

  it("the release steps mention the bump script and the files of a release", () => {
    expect(contributing).toContain("node scripts/bump-version.mjs");
    for (const name of [
      "install.sh",
      "install.ps1",
      "checksums.txt",
      "THIRD-PARTY.md",
    ]) {
      expect(contributing, name).toContain(name);
    }
  });

  it("the README says the first release is not published until it is", () => {
    // Remove this note and this test in task 5.2, when 0.1.0 is published.
    expect(readme).toContain("is not published yet");
  });
});
