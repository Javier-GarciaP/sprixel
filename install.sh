#!/bin/sh
# Sprixel installer for Linux and macOS.
#
#   curl -fsSL https://github.com/Javier-GarciaP/sprixel/releases/latest/download/install.sh | sh
#   ... | sh -s -- --version 0.2.0      (a specific version)
#   ... | sh -s -- --yes --no-menu --connect claude      (no questions)
#
# It installs for the current user only (no sudo). It downloads the files of a
# GitHub Release, checks their SHA-256 and only then copies them:
#   Linux:  ~/.local/bin/sprixel and ~/.local/bin/sprixel-app (and its menu entry)
#   macOS:  ~/.local/bin/sprixel and ~/Applications/Sprixel.app
# Your projects and the folder ~/.sprixel (settings, tokens) are never touched,
# so running it again upgrades or repairs the installation.
#
# On a terminal it shows a summary, asks you to confirm and asks two questions
# (the menu entry, and connecting an AI). With no terminal, or with --yes, it
# asks nothing and uses the default answers.
#
# Options: --version X.Y.Z, --yes (-y), --no-menu, --connect claude|opencode|both|none.
# Variables: SPRIXEL_YES=1, SPRIXEL_MENU=0, SPRIXEL_CONNECT=..., NO_COLOR.
# For tests only: SPRIXEL_BASE_URL replaces the download folder (for example
# file:///tmp/release), SPRIXEL_OS and SPRIXEL_ARCH replace `uname`, and
# SPRIXEL_TTY replaces /dev/tty (a file with the answers; empty means no terminal).
set -eu

REPO="Javier-GarciaP/sprixel"
SOURCE="https://github.com/$REPO"

say() { printf '%s\n' "$*"; }
fail() { printf 'error: %s\n' "$*" >&2; exit 1; }

VERSION=""
YES="${SPRIXEL_YES:-}"
MENU="${SPRIXEL_MENU:-}"
CONNECT="${SPRIXEL_CONNECT:-}"
while [ $# -gt 0 ]; do
  case "$1" in
    --version) [ $# -ge 2 ] || fail "--version needs a value, for example --version 0.2.0"
               VERSION="${2#v}"; shift 2 ;;
    --yes|-y) YES=1; shift ;;
    --no-menu) MENU=0; shift ;;
    --connect) [ $# -ge 2 ] || fail "--connect needs a value: claude, opencode, both or none"
               CONNECT="$2"; shift 2 ;;
    -h|--help) say "usage: install.sh [--version X.Y.Z] [--yes] [--no-menu] [--connect claude|opencode|both|none]"; exit 0 ;;
    *) fail "unknown option: $1 (usage: install.sh [--version X.Y.Z] [--yes] [--no-menu] [--connect claude|opencode|both|none])" ;;
  esac
done
case "$CONNECT" in
  ""|claude|opencode|both|none) ;;
  *) fail "--connect must be claude, opencode, both or none (not \"$CONNECT\")." ;;
esac

# Colors only on a real terminal, and never when NO_COLOR is set.
BOLD=""; RESET=""
if [ -t 1 ] && [ -z "${NO_COLOR:-}" ] && [ "${TERM:-dumb}" != dumb ]; then
  BOLD="$(printf '\033[1m')"; RESET="$(printf '\033[0m')"
fi

say "${BOLD}Sprixel installer${RESET}"
say "Sprixel is open source software released under the MIT License."
say "Source code: $SOURCE"
say "The license and the third-party notices are installed in ~/.local/share/sprixel/"
say ""

# Every path below starts with $HOME. An empty one would point at the root of the
# system, so stop before anything is copied or removed.
[ -n "${HOME:-}" ] && [ -d "$HOME" ] || fail "HOME must be set to your home folder. Nothing was installed."

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
BIN="$HOME/.local/bin"
SHARE="$HOME/.local/share/sprixel"

# 3. Is there a terminal to ask on? `curl ... | sh` gives the script to the shell
# through its input, so the answers are read from the terminal itself (fd 3).
INTERACTIVE=""
if [ -z "$YES" ]; then
  TTY_FILE="${SPRIXEL_TTY-/dev/tty}"
  if [ -n "$TTY_FILE" ] && ( exec 3< "$TTY_FILE" ) 2>/dev/null; then
    exec 3< "$TTY_FILE"
    INTERACTIVE=1
  fi
fi

# ask_yes "question" default(Y|N): returns 0 for yes. Enter, or no more lines,
# gives the default.
ask_yes() {
  while :; do
    printf '%s ' "$1"
    # No more lines (the terminal ended): the cursor is still on the question.
    if IFS= read -r ANSWER <&3; then :; else ANSWER=""; say ""; fi
    [ -n "$ANSWER" ] || ANSWER="$2"
    case "$ANSWER" in
      [Yy]|[Yy][Ee][Ss]) return 0 ;;
      [Nn]|[Nn][Oo]) return 1 ;;
      *) say "Please answer y or n." ;;
    esac
  done
}

# 4. The summary: what will be installed, and where.
say "${BOLD}This will install:${RESET}"
say "  Version   ${VERSION:-the latest}"
say "  System    $OS $ARCH"
if [ "$OS" = linux ]; then
  say "  App       ~/.local/bin/sprixel-app"
else
  say "  App       ~/Applications/Sprixel.app"
fi
say "  Command   ~/.local/bin/sprixel"
say "  Notices   ~/.local/share/sprixel/"
say ""

if [ -n "$INTERACTIVE" ]; then
  ask_yes "Install Sprixel? [Y/n]" Y || { say "Nothing was installed."; exit 0; }
  say ""
fi

# 5. The options. A flag or a variable answers the question before it is asked.
# The menu entry exists on Linux only: macOS uses the app bundle itself.
if [ "$OS" = linux ] && [ -z "$MENU" ]; then
  MENU=1
  if [ -n "$INTERACTIVE" ]; then
    ask_yes "Add Sprixel to your applications menu? [Y/n]" Y || MENU=0
  fi
fi

# The AI tools that this computer has. With none, the question is not asked.
AI_ASKED=""
if [ -n "$INTERACTIVE" ] && [ -z "$CONNECT" ]; then
  HAVE_CLAUDE=""; HAVE_OPENCODE=""
  if command -v claude >/dev/null 2>&1; then HAVE_CLAUDE=1; fi
  if command -v opencode >/dev/null 2>&1; then HAVE_OPENCODE=1; fi
  if [ -n "$HAVE_CLAUDE$HAVE_OPENCODE" ]; then
    set -- none
    [ -z "$HAVE_CLAUDE" ] || set -- "$@" claude
    [ -z "$HAVE_OPENCODE" ] || set -- "$@" opencode
    [ -z "$HAVE_CLAUDE" ] || [ -z "$HAVE_OPENCODE" ] || set -- "$@" both
    COUNT=$#
    say "Connect an AI tool to Sprixel now?"
    N=0
    for CHOICE in "$@"; do
      N=$((N + 1))
      case "$CHOICE" in
        none) LABEL="No, I will do it later" ;;
        claude) LABEL="Claude Code" ;;
        opencode) LABEL="OpenCode" ;;
        both) LABEL="Both" ;;
      esac
      say "  $N) $LABEL"
    done
    while :; do
      printf 'Choose 1-%s [1]: ' "$COUNT"
      if IFS= read -r ANSWER <&3; then :; else ANSWER=""; say ""; fi
      [ -n "$ANSWER" ] || ANSWER=1
      case "$ANSWER" in
        *[!0-9]*) say "Please type a number from 1 to $COUNT." ;;
        *)
          if [ "${#ANSWER}" -le 2 ] && [ "$ANSWER" -ge 1 ] && [ "$ANSWER" -le "$COUNT" ]; then
            N=0
            for CHOICE in "$@"; do
              N=$((N + 1))
              if [ "$N" = "$ANSWER" ]; then CONNECT="$CHOICE"; fi
            done
            break
          fi
          say "Please type a number from 1 to $COUNT." ;;
      esac
    done
    AI_ASKED=1
    say ""
  fi
fi
[ -n "$CONNECT" ] || CONNECT=none

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
trap 'exit 1' INT TERM

# Only https is accepted, also after a redirect. file:// is allowed only for the
# tests, through SPRIXEL_BASE_URL.
PROTO="=https"
[ -z "${SPRIXEL_BASE_URL:-}" ] || PROTO="=https,file"

# 6. Download everything and check it BEFORE installing anything. On a terminal
# curl draws a progress bar; otherwise each file is one plain line.
TOTAL=6
COUNTED=0
if [ -t 2 ] && [ "${TERM:-dumb}" != dumb ]; then BAR=1; else BAR=""; fi
get() {
  COUNTED=$((COUNTED + 1))
  if [ -n "$BAR" ]; then
    say "[$COUNTED/$TOTAL] $1"
    curl --proto "$PROTO" --proto-redir "=https" -fL --progress-bar "$BASE/$1" -o "$TMP/$1" || fail "cannot download $BASE/$1 (is the version right, and are you online?)"
  else
    printf '[%s/%s] %s ... ' "$COUNTED" "$TOTAL" "$1"
    curl --proto "$PROTO" --proto-redir "=https" -fsSL "$BASE/$1" -o "$TMP/$1" || { say ""; fail "cannot download $BASE/$1 (is the version right, and are you online?)"; }
    say "done"
  fi
}

say "Downloading Sprixel ($OS $ARCH)..."
get checksums.txt
for name in VERSION "$CLI" "$APP" LICENSE THIRD-PARTY.md; do
  get "$name"
done
for name in VERSION "$CLI" "$APP" LICENSE THIRD-PARTY.md; do
  want="$(awk -v n="$name" '$2 == n { print $1 }' "$TMP/checksums.txt")"
  [ -n "$want" ] || fail "checksums.txt has no checksum for $name. Nothing was installed."
  [ "$(sha "$TMP/$name")" = "$want" ] || fail "the checksum of $name does not match. The download is damaged or was changed. Nothing was installed."
done
say "Checksums are correct."

# 7. Install into the user folders.
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

# 8. Make sure `sprixel` is on the PATH of new terminals.
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

# 9. Linux: the applications menu. A "no" is kept in ~/.sprixel/no-menu-entry,
# which the app reads, so it does not add the entry by itself later. A "yes"
# removes an old "no". A problem here is a warning, never an error.
MENU_RESULT=""
if [ "$OS" = linux ]; then
  say ""
  if [ "$MENU" = 0 ]; then
    mkdir -p "$HOME/.sprixel" && : > "$HOME/.sprixel/no-menu-entry"
    say "Skipped the applications menu entry (you can add it later with: ~/.local/bin/sprixel-app --register-menu)."
    MENU_RESULT="no"
  else
    rm -f "$HOME/.sprixel/no-menu-entry"
    say "Adding Sprixel to the applications menu..."
    if command -v timeout >/dev/null 2>&1; then
      run_limited() { timeout 30 "$@"; }
    else
      run_limited() { "$@"; }
    fi
    if MENU_OUT="$(run_limited "$BIN/sprixel-app" --register-menu 2>&1 </dev/null)"; then
      say "$(printf '%s\n' "$MENU_OUT" | tail -n 1)"
      MENU_RESULT="yes"
    else
      say "Sprixel could not be added to the applications menu. It will try again the first"
      say "time you open the app, or you can run: ~/.local/bin/sprixel-app --register-menu"
      MENU_RESULT="not done"
    fi
  fi
fi

# 10. Connect the AI tools that were chosen. The command is run by its full
# path, because the PATH of this terminal has not changed yet.
CONNECTED=""
connect_one() {
  case "$1" in claude) NAME="Claude Code" ;; *) NAME="OpenCode" ;; esac
  say ""
  say "Connecting $NAME..."
  if "$BIN/sprixel" mcp add "$1" </dev/null; then
    CONNECTED="${CONNECTED:+$CONNECTED, }$NAME"
  else
    say "warning: could not connect $NAME. You can try again later with: sprixel mcp add $1"
  fi
}
case "$CONNECT" in
  claude) connect_one claude ;;
  opencode) connect_one opencode ;;
  both) connect_one claude; connect_one opencode ;;
esac

# 11. The summary and the next steps.
say ""
say "Sprixel $INSTALLED is installed."
say ""
say "  Command   ~/.local/bin/sprixel"
if [ "$OS" = linux ]; then say "  App       ~/.local/bin/sprixel-app"; else say "  App       ~/Applications/Sprixel.app"; fi
[ -z "$MENU_RESULT" ] || say "  Choices   menu entry: $MENU_RESULT"
say "  AI tools: ${CONNECTED:-none connected}"
[ -z "$NEW_TERMINAL" ] || { say ""; say "Open a new terminal so that the 'sprixel' command is found."; }
say ""
say "Next steps:"
say "  sprixel doctor            check that everything works"
if [ -z "$CONNECTED" ]; then
  say "  sprixel mcp add claude    let Claude Code use Sprixel (or: opencode)"
fi
if [ "$OS" = macos ]; then
  say ""
  say "The app is not signed yet. If macOS blocks it the first time, right-click"
  say "Sprixel in ~/Applications, choose Open, and confirm."
fi
