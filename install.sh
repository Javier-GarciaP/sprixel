#!/bin/sh
# Sprixel installer for Linux and macOS.
#
#   curl -fsSL https://github.com/Javier-GarciaP/sprixel/releases/latest/download/install.sh | sh
#   ... | sh -s -- --version 0.2.0      (a specific version)
#
# It installs for the current user only (no sudo). It downloads the files of a
# GitHub Release, checks their SHA-256 and only then copies them:
#   Linux:  ~/.local/bin/sprixel and ~/.local/bin/sprixel-app
#   macOS:  ~/.local/bin/sprixel and ~/Applications/Sprixel.app
# Your projects and the folder ~/.sprixel (settings, tokens) are never touched,
# so running it again upgrades or repairs the installation.
#
# SPRIXEL_BASE_URL (tests only) replaces the download folder, for example
# file:///tmp/release. SPRIXEL_OS and SPRIXEL_ARCH (tests only) replace `uname`.
set -eu

REPO="Javier-GarciaP/sprixel"
SOURCE="https://github.com/$REPO"

say() { printf '%s\n' "$*"; }
fail() { printf 'error: %s\n' "$*" >&2; exit 1; }

VERSION=""
while [ $# -gt 0 ]; do
  case "$1" in
    --version) [ $# -ge 2 ] || fail "--version needs a value, for example --version 0.2.0"
               VERSION="${2#v}"; shift 2 ;;
    -h|--help) say "usage: install.sh [--version X.Y.Z]"; exit 0 ;;
    *) fail "unknown option: $1 (usage: install.sh [--version X.Y.Z])" ;;
  esac
done

say "Sprixel is open source software released under the MIT License."
say "Source code: $SOURCE"
say "The license and the third-party notices are installed in ~/.local/share/sprixel/"
say ""

# 1. Which system is this?
case "${SPRIXEL_OS:-$(uname -s)}" in
  Linux)  OS=linux ;;
  Darwin) OS=macos ;;
  *) OS="" ;;
esac
case "${SPRIXEL_ARCH:-$(uname -m)}" in
  x86_64|amd64)  ARCH=x64 ;;
  arm64|aarch64) ARCH=arm64 ;;
  *) ARCH="" ;;
esac
case "$OS-$ARCH" in
  linux-x64|macos-x64|macos-arm64) ;;
  *) fail "this system is not supported by install.sh ($(uname -s) $(uname -m)).
Supported: Linux x64, macOS Apple silicon and macOS Intel. For Windows use install.ps1." ;;
esac

command -v curl >/dev/null 2>&1 || fail "curl is needed to download Sprixel. Install curl and try again."
if command -v sha256sum >/dev/null 2>&1; then
  sha() { sha256sum "$1" | cut -d ' ' -f 1; }
elif command -v shasum >/dev/null 2>&1; then
  sha() { shasum -a 256 "$1" | cut -d ' ' -f 1; }
else
  fail "sha256sum or shasum is needed to check the download."
fi

# 2. Where do the files come from?
if [ -n "${SPRIXEL_BASE_URL:-}" ]; then
  BASE="$SPRIXEL_BASE_URL"
elif [ -n "$VERSION" ]; then
  BASE="$SOURCE/releases/download/v$VERSION"
else
  BASE="$SOURCE/releases/latest/download"
fi

CLI="sprixel-$OS-$ARCH"
if [ "$OS" = linux ]; then APP="Sprixel-linux-$ARCH.AppImage"; else APP="Sprixel-macos-$ARCH.app.tar.gz"; fi

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
trap 'exit 1' INT TERM

# 3. Download everything and check it BEFORE installing anything.
get() { curl -fsSL "$BASE/$1" -o "$TMP/$1" || fail "cannot download $BASE/$1 (is the version right, and are you online?)"; }

say "Downloading Sprixel ($OS $ARCH)..."
get checksums.txt
for name in VERSION "$CLI" "$APP" LICENSE THIRD-PARTY.md; do
  get "$name"
  want="$(awk -v n="$name" '$2 == n { print $1 }' "$TMP/checksums.txt")"
  [ -n "$want" ] || fail "checksums.txt has no checksum for $name. Nothing was installed."
  [ "$(sha "$TMP/$name")" = "$want" ] || fail "the checksum of $name does not match. The download is damaged or was changed. Nothing was installed."
done
say "Checksums are correct."

# 4. Install into the user folders.
BIN="$HOME/.local/bin"
SHARE="$HOME/.local/share/sprixel"
mkdir -p "$BIN" "$SHARE"

# Copy to a temporary name, then rename: a running program is replaced safely.
put() { cp "$1" "$2.new" && chmod 755 "$2.new" && mv -f "$2.new" "$2"; }

put "$TMP/$CLI" "$BIN/sprixel"
if [ "$OS" = linux ]; then
  put "$TMP/$APP" "$BIN/sprixel-app"
else
  mkdir -p "$HOME/Applications"
  mkdir "$TMP/app" && tar -xzf "$TMP/$APP" -C "$TMP/app"
  [ -d "$TMP/app/Sprixel.app" ] || fail "the app archive has no Sprixel.app. Nothing else was changed."
  rm -rf "$HOME/Applications/Sprixel.app"
  mv "$TMP/app/Sprixel.app" "$HOME/Applications/Sprixel.app"
fi
cp "$TMP/LICENSE" "$SHARE/LICENSE"
cp "$TMP/THIRD-PARTY.md" "$SHARE/THIRD-PARTY.md"
INSTALLED="$(cat "$TMP/VERSION")"

# 5. Make sure `sprixel` is on the PATH of new terminals.
NEW_TERMINAL=""
case ":$PATH:" in
  *":$BIN:"*) ;;
  *)
    case "${SHELL:-}" in
      */zsh) PROFILE="$HOME/.zprofile" ;;
      */bash) if [ "$OS" = macos ]; then PROFILE="$HOME/.bash_profile"; else PROFILE="$HOME/.profile"; fi ;;
      *) PROFILE="$HOME/.profile" ;;
    esac
    # The marker lets `sprixel uninstall` remove exactly this line.
    if ! grep -qs "# Added by the Sprixel installer" "$PROFILE"; then
      printf '\n# Added by the Sprixel installer\nexport PATH="$HOME/.local/bin:$PATH"\n' >> "$PROFILE"
    fi
    NEW_TERMINAL="yes"
    say "Added $BIN to your PATH in $PROFILE."
    ;;
esac

say ""
say "Sprixel $INSTALLED is installed."
[ -z "$NEW_TERMINAL" ] || say "Open a new terminal so that the 'sprixel' command is found."
say ""
say "Next steps:"
say "  sprixel doctor            check that everything works"
say "  sprixel mcp add claude    let Claude Code use Sprixel (or: opencode)"
if [ "$OS" = macos ]; then
  say ""
  say "The app is not signed yet. If macOS blocks it the first time, right-click"
  say "Sprixel in ~/Applications, choose Open, and confirm."
fi
