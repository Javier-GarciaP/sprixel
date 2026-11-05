import process from "node:process";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { makeRelease } from "./release-fixture.mjs";

// Every test runs install.sh with a temporary HOME and a fake release in a
// local folder (file:// URL). Nothing touches the real machine or the network.
const script = path.resolve("install.sh");
let root, home, release;

beforeEach(() => {
  root = fs.mkdtempSync(path.join(os.tmpdir(), "sprixel-install-"));
  home = path.join(root, "home");
  release = path.join(root, "release");
  fs.mkdirSync(home);
});
afterEach(() => fs.rmSync(root, { recursive: true, force: true }));

function install(args = [], env = {}) {
  return spawnSync("sh", [script, ...args], {
    encoding: "utf8",
    env: {
      PATH: process.env.PATH,
      HOME: home,
      SHELL: "/bin/bash",
      SPRIXEL_BASE_URL: `file://${release}`,
      SPRIXEL_OS: "Linux",
      SPRIXEL_ARCH: "x86_64",
      ...env,
    },
  });
}
const file = (...p) => path.join(home, ...p);

// install.sh is for Linux and macOS; Windows has install.ps1.
describe.skipIf(process.platform === "win32")("install.sh", () => {
  it("installs the command, the app and the notices for the user", () => {
    makeRelease(release);
    const r = install();

    expect(r.status, r.stderr).toBe(0);
    expect(fs.existsSync(file(".local/bin/sprixel"))).toBe(true);
    expect(fs.existsSync(file(".local/bin/sprixel-app"))).toBe(true);
    expect(fs.statSync(file(".local/bin/sprixel")).mode & 0o100).toBeTruthy();
    expect(fs.existsSync(file(".local/share/sprixel/LICENSE"))).toBe(true);
    expect(fs.existsSync(file(".local/share/sprixel/THIRD-PARTY.md"))).toBe(
      true,
    );
  });

  // Checks the scenario "Entry after the install script": the script asks the
  // new app to put itself in the applications menu, before the first launch.
  it("asks the Linux app to register its menu entry", () => {
    makeRelease(release);
    const r = install();

    expect(r.status, r.stderr).toBe(0);
    expect(fs.readFileSync(file(".fake-app-calls"), "utf8")).toBe(
      "--register-menu\n",
    );
    expect(r.stdout).toContain("menu entry created: fake");
  });

  // Checks the scenario "Read-only menu folder": a failure to register is a
  // warning that says how to retry, and the install still ends well.
  it("still installs when the app cannot register its entry", () => {
    makeRelease(release, { registerMenuFails: true });
    const r = install();

    expect(r.status, r.stderr).toBe(0);
    expect(r.stdout).toContain("could not be added to the applications menu");
    expect(r.stdout).toContain("sprixel-app --register-menu");
    expect(fs.existsSync(file(".local/bin/sprixel-app"))).toBe(true);
  });

  // Checks the macOS rule: the app bundle is the menu entry, so the script
  // adds no step for it.
  it("adds no menu step on macOS", () => {
    makeRelease(release);
    const r = install([], { SPRIXEL_OS: "Darwin", SPRIXEL_ARCH: "arm64" });

    expect(r.status, r.stderr).toBe(0);
    expect(r.stdout).not.toContain("applications menu");
    expect(fs.existsSync(file(".fake-app-calls"))).toBe(false);
  });

  it("prints the license notice and the source URL before installing", () => {
    makeRelease(release);
    const out = install().stdout;

    expect(out).toContain("MIT License");
    expect(out).toContain("https://github.com/Javier-GarciaP/sprixel");
    expect(out.indexOf("MIT License")).toBeLessThan(out.indexOf("Downloading"));
  });

  it("prints the next steps", () => {
    makeRelease(release);
    const out = install().stdout;

    expect(out).toContain("sprixel doctor");
    expect(out).toContain("sprixel mcp add");
    expect(out).toContain("Sprixel 0.2.0 is installed");
  });

  it("stops and installs nothing when a checksum is wrong", () => {
    makeRelease(release, { wrongChecksum: "Sprixel-linux-x64.AppImage" });
    const r = install();

    expect(r.status).toBe(1);
    expect(r.stderr).toContain("checksum of Sprixel-linux-x64.AppImage");
    expect(fs.existsSync(file(".local"))).toBe(false);
    expect(fs.existsSync(file(".profile"))).toBe(false);
  });

  it("stops when a file is changed after the checksums were made", () => {
    makeRelease(release);
    fs.appendFileSync(path.join(release, "sprixel-linux-x64"), "evil\n");
    const r = install();

    expect(r.status).toBe(1);
    expect(fs.existsSync(file(".local"))).toBe(false);
  });

  it("stops when checksums.txt does not list a file", () => {
    makeRelease(release);
    const sums = path.join(release, "checksums.txt");
    const lines = fs.readFileSync(sums, "utf8").split("\n");
    fs.writeFileSync(
      sums,
      lines.filter((l) => !l.endsWith("  LICENSE")).join("\n"),
    );
    const r = install();

    expect(r.status).toBe(1);
    expect(r.stderr).toContain("no checksum for LICENSE");
    expect(fs.existsSync(file(".local"))).toBe(false);
  });

  it("stops with the supported systems on an unsupported one", () => {
    makeRelease(release);
    for (const env of [
      { SPRIXEL_OS: "FreeBSD" },
      { SPRIXEL_ARCH: "riscv64" },
    ]) {
      const r = install([], env);

      expect(r.status).toBe(1);
      expect(r.stderr).toContain("not supported");
      expect(r.stderr).toContain("Linux x64");
      expect(fs.existsSync(file(".local"))).toBe(false);
    }
  });

  it("adds the install folder to the PATH once and says to open a new terminal", () => {
    makeRelease(release);
    const first = install([], { PATH: "/usr/bin:/bin" });
    const second = install([], { PATH: "/usr/bin:/bin" });
    const profile = fs.readFileSync(file(".profile"), "utf8");

    expect(first.stdout).toContain("Open a new terminal");
    expect(profile).toContain('export PATH="$HOME/.local/bin:$PATH"');
    expect(profile.match(/Added by the Sprixel installer/g)).toHaveLength(1);
    expect(second.status).toBe(0);
  });

  it("does not touch the profile when the folder is already in the PATH", () => {
    makeRelease(release);
    const r = install([], {
      PATH: `${file(".local/bin")}:${process.env.PATH}`,
    });

    expect(r.status, r.stderr).toBe(0);
    expect(fs.existsSync(file(".profile"))).toBe(false);
    expect(r.stdout).not.toContain("Open a new terminal");
  });

  it("uses .zprofile for zsh", () => {
    makeRelease(release);
    install([], { PATH: "/usr/bin:/bin", SHELL: "/usr/bin/zsh" });

    expect(fs.existsSync(file(".zprofile"))).toBe(true);
    expect(fs.existsSync(file(".profile"))).toBe(false);
  });

  it("reinstalling upgrades the version and keeps projects, settings and tokens", () => {
    makeRelease(release, { version: "0.2.0" });
    install();
    const keep = {
      ".sprixel/token": "secret-token\n",
      ".sprixel/settings.json": '{"theme":"dark"}\n',
      ".sprixel/clients.json": '{"approved":["claude"]}\n',
      "Sprites/hero/sprixel.json": '{"name":"hero"}\n',
    };
    for (const [name, text] of Object.entries(keep)) {
      fs.mkdirSync(path.dirname(file(name)), { recursive: true });
      fs.writeFileSync(file(name), text);
    }

    makeRelease(release, { version: "0.3.0" });
    const r = install();

    expect(r.status, r.stderr).toBe(0);
    expect(r.stdout).toContain("Sprixel 0.3.0 is installed");
    expect(fs.readFileSync(file(".local/bin/sprixel"), "utf8")).toContain(
      "0.3.0",
    );
    for (const [name, text] of Object.entries(keep)) {
      expect(fs.readFileSync(file(name), "utf8")).toBe(text);
    }
  });

  it("reinstalling changes nothing outside the install folders", () => {
    makeRelease(release);
    install();
    // Things the user owns: projects, data folder, a profile with other lines.
    fs.mkdirSync(file(".sprixel"), { recursive: true });
    fs.writeFileSync(file(".sprixel/token"), "t\n");
    fs.mkdirSync(file("Documents/game"), { recursive: true });
    fs.writeFileSync(file("Documents/game/hero.txt"), "pixels\n");
    fs.writeFileSync(file(".profile"), "alias ll='ls -l'\n");
    const mine = [".sprixel/token", "Documents/game/hero.txt", ".profile"];
    const snapshot = () => mine.map((n) => fs.readFileSync(file(n), "utf8"));
    const before = snapshot();

    makeRelease(release, { version: "0.3.0" });
    const r = install([], { PATH: "/usr/bin:/bin" });

    expect(r.status, r.stderr).toBe(0);
    // Only the installer's own two lines were added to the profile.
    const [token, hero, profile] = snapshot();
    expect([token, hero]).toEqual(before.slice(0, 2));
    expect(profile.startsWith(before[2])).toBe(true);
  });

  it("repairs an installation whose command was deleted", () => {
    makeRelease(release);
    install();
    fs.rmSync(file(".local/bin/sprixel"));
    const r = install();

    expect(r.status, r.stderr).toBe(0);
    expect(fs.existsSync(file(".local/bin/sprixel"))).toBe(true);
  });

  it("a failed reinstall leaves the working installation as it was", () => {
    makeRelease(release, { version: "0.2.0" });
    install();
    makeRelease(release, {
      version: "0.3.0",
      wrongChecksum: "sprixel-linux-x64",
    });
    const r = install();

    expect(r.status).toBe(1);
    expect(fs.readFileSync(file(".local/bin/sprixel"), "utf8")).toContain(
      "0.2.0",
    );
  });

  // An empty HOME would turn "$HOME/Applications" into "/Applications": the
  // script must stop before it copies or removes anything.
  it("stops when HOME is empty or is not a folder", () => {
    makeRelease(release);
    for (const bad of ["", path.join(root, "missing")]) {
      const r = install([], { HOME: bad });
      expect(r.status).not.toBe(0);
      expect(r.stderr).toContain("HOME");
    }
  });

  it("--version needs a value and unknown options fail", () => {
    expect(install(["--version"]).status).toBe(1);
    expect(install(["--nope"]).stderr).toContain("unknown option");
  });

  it("installs the app on macOS under ~/Applications and replaces the old one", () => {
    makeRelease(release);
    const mac = {
      SPRIXEL_OS: "Darwin",
      SPRIXEL_ARCH: "arm64",
      SHELL: "/bin/zsh",
    };
    const r = install([], mac);
    const exe = file("Applications/Sprixel.app/Contents/MacOS/sprixel");

    expect(r.status, r.stderr).toBe(0);
    expect(fs.existsSync(exe)).toBe(true);
    expect(fs.existsSync(file(".local/bin/sprixel"))).toBe(true);
    expect(r.stdout).toContain("not signed");

    makeRelease(release, { version: "0.3.0" });
    install([], mac);
    expect(fs.readFileSync(exe, "utf8")).toContain("0.3.0");
  });
});
