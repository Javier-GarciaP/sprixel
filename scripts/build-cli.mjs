// Builds the `sprixel` command into bin/ with the version from package.json.
import { execFileSync } from "node:child_process";
import fs from "node:fs";

const { version } = JSON.parse(fs.readFileSync("package.json", "utf8"));
const exe = process.platform === "win32" ? "sprixel.exe" : "sprixel";
const flag = `-X github.com/Javier-GarciaP/sprixel/cli/internal/version.Version=${version}`;

execFileSync(
  "go",
  [
    "build",
    "-C",
    "cli",
    "-ldflags",
    flag,
    "-o",
    `../bin/${exe}`,
    "./cmd/sprixel",
  ],
  { stdio: "inherit" },
);
console.log(`Built bin/${exe} (version ${version})`);
