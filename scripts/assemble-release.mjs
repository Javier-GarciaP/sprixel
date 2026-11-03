// Puts the files of every build job together into the folder that is
// published, and writes checksums.txt.
//
//   node scripts/assemble-release.mjs <artifactsDir> <outDir>
//
// <artifactsDir> holds the files of the build jobs (one folder per job, as
// actions/download-artifact writes them, or loose files). The result has the
// bundles and commands for every system, the install scripts, LICENSE,
// THIRD-PARTY.md, VERSION and checksums.txt. It fails when a file is missing,
// so a release is never published with a system left out.
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { TARGETS, builtFiles } from "./release-files.mjs";

const walk = (dir) =>
  fs.readdirSync(dir, { withFileTypes: true }).flatMap((entry) => {
    const full = path.join(dir, entry.name);
    return entry.isDirectory() ? walk(full) : [full];
  });

/** Builds the release folder. `root` is the repository, with the shared files. */
export function assemble(root, artifactsDir, outDir) {
  const built = new Map(
    walk(artifactsDir).map((file) => [path.basename(file), file]),
  );
  const missing = TARGETS.flatMap(builtFiles).filter(
    (name) => !built.has(name),
  );
  if (missing.length > 0) {
    throw new Error(
      `these release files were not built: ${missing.join(", ")}`,
    );
  }

  fs.rmSync(outDir, { recursive: true, force: true });
  fs.mkdirSync(outDir, { recursive: true });
  for (const name of TARGETS.flatMap(builtFiles)) {
    fs.copyFileSync(built.get(name), path.join(outDir, name));
  }
  const shared = {
    "install.sh": "install.sh",
    "install.ps1": "install.ps1",
    LICENSE: "LICENSE",
    "THIRD-PARTY.md": "docs/third-party.md",
  };
  for (const [name, from] of Object.entries(shared)) {
    fs.copyFileSync(path.join(root, from), path.join(outDir, name));
  }
  const { version } = JSON.parse(
    fs.readFileSync(path.join(root, "package.json"), "utf8"),
  );
  fs.writeFileSync(path.join(outDir, "VERSION"), `${version}\n`);

  // sha256sum format: "<hash>  <name>". The installers read this file.
  const names = fs.readdirSync(outDir).sort();
  const lines = names.map((name) => {
    const hash = createHash("sha256")
      .update(fs.readFileSync(path.join(outDir, name)))
      .digest("hex");
    return `${hash}  ${name}`;
  });
  fs.writeFileSync(path.join(outDir, "checksums.txt"), lines.join("\n") + "\n");
  return [...names, "checksums.txt"];
}

function main([artifacts, out]) {
  if (!artifacts || !out) {
    console.error(
      "usage: node scripts/assemble-release.mjs <artifactsDir> <outDir>",
    );
    return 2;
  }
  try {
    for (const name of assemble(process.cwd(), artifacts, out))
      console.log(name);
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
