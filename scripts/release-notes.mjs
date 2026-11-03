// Prints the release notes of a tag: the section of CHANGELOG.md with the
// same version.
//
//   node scripts/release-notes.mjs v0.2.0 > notes.md
//
// It fails when the section is missing or empty, so a release never goes out
// with blank notes.
import fs from "node:fs";
import { pathToFileURL } from "node:url";

/** Returns the text under "## [version]" in a changelog, or throws. */
export function extractNotes(changelog, version) {
  const lines = changelog.split("\n");
  const heading = new RegExp(
    `^## \\[${version.replace(/[.+]/g, "\\$&")}\\]( |$)`,
  );
  const start = lines.findIndex((line) => heading.test(line));
  if (start === -1) {
    throw new Error(
      `CHANGELOG.md has no section "## [${version}]". Move the Unreleased notes under it in the release commit.`,
    );
  }
  let end = lines.findIndex((line, i) => i > start && /^## /.test(line));
  if (end === -1) end = lines.length;
  const body = lines
    .slice(start + 1, end)
    .join("\n")
    .trim();
  if (!body)
    throw new Error(`The CHANGELOG.md section "## [${version}]" is empty.`);
  return body + "\n";
}

function main([tag]) {
  if (!tag || !/^v\d/.test(tag)) {
    console.error("usage: node scripts/release-notes.mjs vMAJOR.MINOR.PATCH");
    return 2;
  }
  try {
    process.stdout.write(
      extractNotes(fs.readFileSync("CHANGELOG.md", "utf8"), tag.slice(1)),
    );
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
