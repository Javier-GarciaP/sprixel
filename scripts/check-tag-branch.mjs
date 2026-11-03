// Fails when a commit is not on main. The release pipeline runs it on the
// commit of the tag: releases come from main only (see CONTRIBUTING.md).
//
//   node scripts/check-tag-branch.mjs [commit] [mainRef]
//
// commit defaults to HEAD and mainRef to origin/main.
import { spawnSync } from "node:child_process";
import { pathToFileURL } from "node:url";

/** Returns true when `commit` is main itself or an ancestor of main. */
export function isOnMain(commit, mainRef) {
  const git = (...args) => spawnSync("git", args, { encoding: "utf8" });
  if (
    git("rev-parse", "--verify", "--quiet", `${mainRef}^{commit}`).status !== 0
  ) {
    throw new Error(
      `cannot find ${mainRef}. Fetch the main branch first (checkout with fetch-depth: 0).`,
    );
  }
  // Exit code 0 means "is an ancestor"; 1 means "is not".
  return git("merge-base", "--is-ancestor", commit, mainRef).status === 0;
}

function main([commit = "HEAD", mainRef = "origin/main"]) {
  try {
    if (isOnMain(commit, mainRef)) {
      console.log(`${commit} is on ${mainRef}.`);
      return 0;
    }
    console.error(
      `error: ${commit} is not on ${mainRef}. Releases come from main: merge the release pull request first, then tag the merge commit.`,
    );
    return 1;
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
