// Collects the build output of one system into the files of a release.
//
//   node scripts/stage-release.mjs <os> <arch> <outDir>
//
// Run it after `tauri build` and `npm run build:cli`. It reads:
//   Linux    src-tauri/target/release/bundle/appimage/*.AppImage
//   macOS    src-tauri/target/release/bundle/macos/Sprixel.app
//   Windows  src-tauri/target/release/sprixel.exe
// and bin/sprixel (bin/sprixel.exe on Windows), and writes the release file
// names (see release-files.mjs) into <outDir>.
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { appName, cliName } from "./release-files.mjs";

const OSES = ["linux", "macos", "windows"];

function find(dir, test, what) {
  const found = fs.existsSync(dir) ? fs.readdirSync(dir).find(test) : undefined;
  if (!found)
    throw new Error(`${what} was not found in ${dir}. Did the build finish?`);
  return path.join(dir, found);
}

/** Copies the build output of `target` into `outDir`. `root` is the repository. */
export function stage(root, target, outDir) {
  const release = path.join(root, "src-tauri/target/release");
  const exe = target.os === "windows" ? "sprixel.exe" : "sprixel";
  const command = path.join(root, "bin", exe);
  if (!fs.existsSync(command)) {
    throw new Error(`${command} was not found. Run npm run build:cli first.`);
  }
  fs.mkdirSync(outDir, { recursive: true });
  fs.copyFileSync(command, path.join(outDir, cliName(target)));

  const app = path.join(outDir, appName(target));
  if (target.os === "linux") {
    const image = find(
      path.join(release, "bundle/appimage"),
      (f) => f.endsWith(".AppImage"),
      "The AppImage",
    );
    fs.copyFileSync(image, app);
  } else if (target.os === "macos") {
    const bundle = find(
      path.join(release, "bundle/macos"),
      (f) => f === "Sprixel.app",
      "Sprixel.app",
    );
    execFileSync("tar", [
      "-czf",
      app,
      "-C",
      path.dirname(bundle),
      "Sprixel.app",
    ]);
  } else {
    // The zip holds Sprixel.exe (the build calls it sprixel.exe).
    const work = fs.mkdtempSync(path.join(outDir, "zip-"));
    fs.copyFileSync(
      path.join(release, "sprixel.exe"),
      path.join(work, "Sprixel.exe"),
    );
    // Windows has tar (bsdtar), which writes zip files; elsewhere use zip.
    if (process.platform === "win32") {
      execFileSync("tar", ["-a", "-c", "-f", app, "-C", work, "Sprixel.exe"]);
    } else {
      execFileSync("zip", ["-q", "-j", app, path.join(work, "Sprixel.exe")]);
    }
    fs.rmSync(work, { recursive: true });
  }
  if (target.os !== "windows")
    fs.chmodSync(path.join(outDir, cliName(target)), 0o755);
  return [cliName(target), appName(target)];
}

function main([os, arch, outDir]) {
  const target = { os, arch };
  if (!OSES.includes(os) || !["x64", "arm64"].includes(arch) || !outDir) {
    console.error(
      "usage: node scripts/stage-release.mjs <linux|macos|windows> <x64|arm64> <outDir>",
    );
    return 2;
  }
  try {
    for (const name of stage(process.cwd(), target, outDir))
      console.log(`staged ${name}`);
    return 0;
  } catch (error) {
    console.error(`error: ${error.message}`);
    return 1;
  }
}

if (
  process.argv[1] &&
  import.meta.url === pathToFileURL(process.argv[1]).href
) {
  process.exitCode = main(process.argv.slice(2));
}
