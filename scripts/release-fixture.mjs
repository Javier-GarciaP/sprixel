// Builds a fake release in a folder, for the tests of the installers. It has
// the same file names and the same checksums.txt as a real release, but the
// "programs" are tiny scripts. Nothing here touches the network.
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";

const sha256 = (file) =>
  createHash("sha256").update(fs.readFileSync(file)).digest("hex");

/**
 * Writes a fake release of `version` into `dir` and returns its file names.
 * `wrongChecksum` is the name of a file whose checksum is made wrong on
 * purpose, to test that installers stop.
 */
export function makeRelease(
  dir,
  { version = "0.2.0", wrongChecksum, registerMenuFails = false } = {},
) {
  fs.mkdirSync(dir, { recursive: true });
  const write = (name, text) => fs.writeFileSync(path.join(dir, name), text);
  const names = [];
  const add = (name, text) => {
    write(name, text);
    names.push(name);
  };

  add("VERSION", `${version}\n`);
  add("LICENSE", "MIT License (fixture)\n");
  add("THIRD-PARTY.md", "Third-party notices (fixture)\n");

  // The command: a shell script that prints the version.
  for (const target of ["linux-x64", "macos-arm64", "macos-x64"]) {
    add(`sprixel-${target}`, `#!/bin/sh\necho ${version}\n`);
  }
  add("sprixel-windows-x64.exe", `fake exe ${version}\n`);

  // The Linux app also answers `--register-menu` like the real one: it notes
  // the call in $HOME/.fake-app-calls and prints a line (or fails, for a test).
  const registerMenu = registerMenuFails
    ? "echo 'fuse: not found' >&2; exit 1"
    : 'echo "menu entry created: fake"; exit 0';
  add(
    "Sprixel-linux-x64.AppImage",
    `#!/bin/sh\nif [ "$1" = "--register-menu" ]; then\n  echo "$@" >> "$HOME/.fake-app-calls"\n  ${registerMenu}\nfi\necho app ${version}\n`,
  );

  // macOS apps are .app folders packed in a .tar.gz.
  for (const arch of ["arm64", "x64"]) {
    const stage = fs.mkdtempSync(path.join(dir, "stage-"));
    const exe = path.join(stage, "Sprixel.app/Contents/MacOS");
    fs.mkdirSync(exe, { recursive: true });
    fs.writeFileSync(path.join(exe, "sprixel"), `app ${version}\n`);
    const name = `Sprixel-macos-${arch}.app.tar.gz`;
    execFileSync("tar", [
      "-czf",
      path.join(dir, name),
      "-C",
      stage,
      "Sprixel.app",
    ]);
    fs.rmSync(stage, { recursive: true });
    names.push(name);
  }

  // Windows app: a zip with Sprixel.exe and the notices.
  const stage = fs.mkdtempSync(path.join(dir, "stage-"));
  fs.writeFileSync(path.join(stage, "Sprixel.exe"), `app ${version}\n`);
  fs.writeFileSync(path.join(stage, "LICENSE"), "MIT License (fixture)\n");
  fs.writeFileSync(path.join(stage, "THIRD-PARTY.md"), "notices\n");
  const zip = path.join(dir, "Sprixel-windows-x64.zip");
  // Windows has tar (bsdtar), which writes zip files; elsewhere use zip.
  if (process.platform === "win32") {
    execFileSync("tar", ["-a", "-c", "-f", zip, "-C", stage, "."]);
  } else {
    execFileSync("zip", ["-qr", zip, "."], { cwd: stage });
  }
  fs.rmSync(stage, { recursive: true });
  names.push("Sprixel-windows-x64.zip");

  const lines = names.map((name) => {
    const hash =
      name === wrongChecksum ? "0".repeat(64) : sha256(path.join(dir, name));
    return `${hash}  ${name}`;
  });
  write("checksums.txt", lines.join("\n") + "\n");
  return names;
}
