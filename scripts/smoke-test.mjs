// Smoke test of an installed Sprixel, run by the release pipeline on a clean
// machine after install.sh or install.ps1:
//
//   node scripts/smoke-test.mjs <version>
//
// It runs `sprixel version` (it must print the release version) and
// `sprixel doctor` (the installation check must be OK: the app is found). The
// other doctor checks fail on a machine with no running app, and that is fine.
import { spawnSync } from "node:child_process";
import os from "node:os";
import path from "node:path";
import process from "node:process";
import { pathToFileURL } from "node:url";

/** Where the installers put the sprixel command for the current user. */
export function installedCommand(
  env = process.env,
  platform = process.platform,
) {
  if (platform === "win32") {
    return path.join(env.LOCALAPPDATA, "Sprixel", "bin", "sprixel.exe");
  }
  return path.join(env.HOME ?? os.homedir(), ".local", "bin", "sprixel");
}

/** Runs the checks and returns the problems found (empty means OK). */
export function smoke(command, expected) {
  const run = (...args) => spawnSync(command, args, { encoding: "utf8" });
  const problems = [];

  const version = run("version");
  if (version.error) return [`cannot run ${command}: ${version.error.message}`];
  if (version.stdout.trim() !== expected) {
    problems.push(
      `sprixel version printed "${version.stdout.trim()}", expected "${expected}"`,
    );
  }

  // doctor exits 1 when any check fails, so only its output is read.
  const doctor = run("doctor", "--json");
  try {
    const checks = JSON.parse(doctor.stdout).checks;
    const installation = checks.find((c) => c.name === "installation");
    if (installation?.status !== "ok") {
      problems.push(
        `doctor: the installation check is ${installation?.status ?? "missing"}: ${installation?.detail ?? ""}`,
      );
    }
  } catch {
    problems.push("sprixel doctor --json did not print JSON");
  }
  return problems;
}

function main([expected]) {
  if (!expected) {
    console.error("usage: node scripts/smoke-test.mjs <version>");
    return 2;
  }
  const command = installedCommand();
  const problems = smoke(command, expected);
  for (const p of problems) console.error(`error: ${p}`);
  if (problems.length === 0)
    console.log(`Smoke test passed: ${command} is ${expected}.`);
  return problems.length === 0 ? 0 : 1;
}

if (
  process.argv[1] &&
  import.meta.url === pathToFileURL(process.argv[1]).href
) {
  process.exitCode = main(process.argv.slice(2));
}
