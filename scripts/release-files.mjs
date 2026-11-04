// The names of the files of a release. They must match cli/internal/release,
// install.sh and install.ps1 (a test checks the installers against a release
// made with these names).

/** The systems we build for. */
export const TARGETS = [
  { os: "linux", arch: "x64" },
  { os: "macos", arch: "arm64" },
  { os: "macos", arch: "x64" },
  { os: "windows", arch: "x64" },
];

/** File name of the sprixel command for a target. */
export const cliName = ({ os, arch }) =>
  os === "windows" ? "sprixel-windows-x64.exe" : `sprixel-${os}-${arch}`;

/** File name of the app bundle for a target. */
export const appName = ({ os, arch }) =>
  ({
    linux: `Sprixel-linux-${arch}.AppImage`,
    macos: `Sprixel-macos-${arch}.app.tar.gz`,
    windows: "Sprixel-windows-x64.zip",
  })[os];

/** Files that every build job produces for its target. */
export const builtFiles = (target) => [cliName(target), appName(target)];

/** Files that are the same for every system. */
export const SHARED_FILES = [
  "install.sh",
  "install.ps1",
  "LICENSE",
  "THIRD-PARTY.md",
  "VERSION",
];

/** Everything a complete release holds, apart from checksums.txt. */
export const allFiles = () => [...TARGETS.flatMap(builtFiles), ...SHARED_FILES];
