// The version of Sprixel lives in package.json. This script keeps every other
// file that holds it in sync. Run it from the repository root:
//
//   node scripts/bump-version.mjs 0.2.0   set the version everywhere
//   node scripts/bump-version.mjs --check fail if the files disagree
//   node scripts/bump-version.mjs --tag v0.2.0
//                                         fail if the tag and the code differ
//
// Files: package.json (the source), package-lock.json, src-tauri/Cargo.toml,
// src-tauri/Cargo.lock and the default of the Go command. Tauri reads
// package.json by itself ("version": "../package.json").
import fs from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";

const SEMVER = /^\d+\.\d+\.\d+(-(alpha|beta|rc)\.\d+)?$/;

// Each file says how to find its version (get) and how to replace it (set).
// The patterns match the one line that holds the version, nothing else.
const lineFile = (file, pattern, build) => ({
  file,
  get: (text) => text.match(pattern)?.[1],
  set: (text, v) => text.replace(pattern, build(v)),
});

const jsonFile = (file, get, set) => ({
  file,
  get: (text) => get(JSON.parse(text)),
  // JSON.stringify keeps the 2-space layout these files already use.
  set: (text, v) => {
    const data = JSON.parse(text);
    set(data, v);
    return JSON.stringify(data, null, 2) + "\n";
  },
});

const TARGETS = [
  jsonFile(
    "package.json",
    (d) => d.version,
    (d, v) => (d.version = v),
  ),
  jsonFile(
    "package-lock.json",
    (d) => (d.version === d.packages[""].version ? d.version : undefined),
    (d, v) => {
      d.version = v;
      d.packages[""].version = v;
    },
  ),
  lineFile(
    "src-tauri/Cargo.toml",
    /^version = "([^"]+)"$/m,
    (v) => `version = "${v}"`,
  ),
  {
    file: "src-tauri/Cargo.lock",
    // The "sprixel" package is the line after its name (LF or CRLF).
    get: (text) => text.match(/name = "sprixel"\r?\nversion = "([^"]+)"/)?.[1],
    set: (text, v) =>
      text.replace(/(name = "sprixel"\r?\nversion = ")[^"]+(")/, `$1${v}$2`),
  },
  lineFile(
    "cli/internal/version/version.go",
    /^var Version = "([^"]+)"$/m,
    (v) => `var Version = "${v}"`,
  ),
];
const read = (root, file) => fs.readFileSync(path.join(root, file), "utf8");

/** Returns the version of every file: { file, version }[] (undefined if unreadable). */
export function readVersions(root) {
  return TARGETS.map((t) => ({
    file: t.file,
    version: t.get(read(root, t.file)),
  }));
}

/** Returns the problems found (an empty list means every file matches). */
export function check(root) {
  const source = JSON.parse(read(root, "package.json")).version;
  const problems = readVersions(root)
    .filter((r) => r.version !== source)
    .map(
      (r) => `${r.file} has ${r.version ?? "no version"}, expected ${source}`,
    );
  const tauri = JSON.parse(read(root, "src-tauri/tauri.conf.json")).version;
  if (tauri !== "../package.json") {
    problems.push(
      `src-tauri/tauri.conf.json must read the version from "../package.json", it has "${tauri}"`,
    );
  }
  return problems;
}

/** Writes the version in every file. Nothing is written if the version is invalid. */
export function bump(root, version) {
  if (!SEMVER.test(version)) {
    throw new Error(
      `"${version}" is not a version: use MAJOR.MINOR.PATCH, for example 0.2.0 or 0.3.0-alpha.1`,
    );
  }
  for (const t of TARGETS) {
    fs.writeFileSync(
      path.join(root, t.file),
      t.set(read(root, t.file), version),
    );
  }
}

/** Returns the problem when the tag is badly formed or is not "v" plus the code version. */
export function checkTag(root, tag) {
  if (!/^v/.test(tag) || !SEMVER.test(tag.slice(1))) {
    return `the tag ${tag} is not in the form vMAJOR.MINOR.PATCH (or -alpha.N, -beta.N, -rc.N)`;
  }
  const version = JSON.parse(read(root, "package.json")).version;
  if (tag !== `v${version}`) {
    return `the tag is ${tag} but the code version is ${version} (the tag must be v${version})`;
  }
}

function main(args) {
  const root = process.cwd();
  if (args[0] === "--tag" && args.length === 2) {
    const problems = [checkTag(root, args[1]), ...check(root)].filter(Boolean);
    for (const p of problems) console.error(`error: ${p}`);
    if (problems.length === 0) console.log(`Tag ${args[1]} matches the code.`);
    return problems.length === 0 ? 0 : 1;
  }
  const [arg] = args;
  if (!arg || args.length > 1) {
    console.error(
      "usage: node scripts/bump-version.mjs <MAJOR.MINOR.PATCH> | --check | --tag vX.Y.Z",
    );
    return 2;
  }
  if (arg !== "--check") {
    try {
      bump(root, arg);
    } catch (error) {
      console.error(`error: ${error.message}`);
      return 2;
    }
  }
  const problems = check(root);
  for (const p of problems) console.error(`error: ${p}`);
  if (problems.length > 0) return 1;
  const version = JSON.parse(read(root, "package.json")).version;
  console.log(`All version files match: ${version}`);
  return 0;
}

if (
  process.argv[1] &&
  import.meta.url === pathToFileURL(process.argv[1]).href
) {
  process.exitCode = main(process.argv.slice(2));
}
