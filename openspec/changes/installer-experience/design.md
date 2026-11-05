# Design

## Context

- `install.sh` (POSIX `sh`) and `install.ps1` (PowerShell 5.1 or newer) download five files with `curl` and `Invoke-WebRequest`, check them, copy them and print a short result. They print no progress.
- The usual command is `curl … | sh` and `irm … | iex`, so the script arrives through the standard input of the shell. A `read` in the script would read the script, not the keyboard.
- The release smoke tests run `sh release/install.sh` and `install.ps1` on clean runners with no terminal. They must keep working with no change.
- The scripts already have test hooks for tests only (`SPRIXEL_BASE_URL`, `SPRIXEL_OS`, `SPRIXEL_ARCH`, `SPRIXEL_SKIP_PATH`) and tests in `scripts/install-sh.test.js` and `scripts/install-ps1.test.js`.
- See proposal.md for the motivation. The menu entry question needs `app-menu-entries`.

## Goals / Non-Goals

**Goals:**
- A clear summary, a confirmation, two questions and a visible progress, with plain `sh` and PowerShell.
- No change for runs with no terminal.
- Everything testable without a real terminal.

**Non-Goals:**
- Arrow-key menus, a separate installer program, a graphical installer.
- Translating the installer (it is in English like the rest of the scripts).
- Changing what is installed or where.

## Decisions

**1. Plain scripts, plain questions.**
Questions use typed answers (`Y/n`, a number). Arrow menus in `sh` need terminal modes and key sequences that differ between terminals, and a separate installer program adds a release file and a big change. Both were offered and not chosen.

**2. Questions read the terminal, not the script (sh).**
A function `ask` reads from `/dev/tty`. A terminal exists only if `/dev/tty` can be opened for reading and writing; if not, the script runs in no-terminal mode. For tests only, `SPRIXEL_TTY=<file>` replaces `/dev/tty`, the same style as the other `SPRIXEL_*` hooks.

**3. One mode switch.**
`INTERACTIVE` is true only with a terminal and without `--yes` or `SPRIXEL_YES=1`. Every question and every confirmation is behind it, so the no-terminal path is the code of today with prints added.

**4. Progress with curl itself (sh).**
On a terminal, `curl --progress-bar` shows a bar with a percentage. It keeps the `--proto` and `--proto-redir` flags that force https. The script prints `[2/5] name` before each download. Not on a terminal, it prints `[2/5] name ... done` and curl runs silent with `-sS`, so a log has one line per file.

**5. Progress on Windows.**
`curl.exe` comes with Windows 10 (1803) and 11, and shows the same bar. The script uses it when `Get-Command curl.exe` finds it and falls back to `Invoke-WebRequest` with `$ProgressPreference = 'SilentlyContinue'` (the default bar makes `Invoke-WebRequest` very slow in PowerShell 5.1) and the plain `[2/5] name ... done` lines.

**6. Flags and variables.**

| Meaning | `install.sh` | PowerShell |
| --- | --- | --- |
| No questions | `--yes`, `-y` or `SPRIXEL_YES=1` | `SPRIXEL_YES=1` |
| No menu entry | `--no-menu` or `SPRIXEL_MENU=0` | `SPRIXEL_MENU=0` |
| Connect an AI | `--connect claude\|opencode\|both\|none` or `SPRIXEL_CONNECT` | `SPRIXEL_CONNECT` |

`irm … | iex` cannot take arguments, so PowerShell uses variables only.

**7. Connecting an AI.**
The script runs the installed command by its full path (`$BIN/sprixel mcp add claude`), because the PATH of the current terminal has not changed yet. It offers a client only when `command -v claude` or `command -v opencode` (`Get-Command` on Windows) finds it. A failure is a warning, not an install failure.

**8. Colors.**
Only when the output is a terminal (`[ -t 1 ]`, `[Console]::IsOutputRedirected` false) and `NO_COLOR` is empty. Only bold and a green check; no cursor movement apart from what curl does.

**9. The menu question and the app.**
The answer "no" must survive: if the app registers its entry at start (see `app-menu-entries`), a "no" would be undone at the first launch. It is kept in the file `~/.sprixel/no-menu-entry` (see the resolved question below).

## Risks / Trade-offs

- **No terminal found by mistake** (for example some containers or CI shells with a fake terminal): the script falls back to no questions, which is safe, but the person sees no questions. `--yes` and the flags remain the way to be explicit.
- **A prompt that hangs a script:** a script run with a terminal that nobody watches would wait for an answer. The confirmation has a clear default and the text says how to skip it (`--yes`).
- **`curl --progress-bar` output differs a little between curl versions.** Accepted; it only affects looks.
- **Windows without `curl.exe`** (older systems) gets no bar, only lines.
- **Two scripts to keep equal** (`sh` and PowerShell). Mitigated by the same text and the same table of options, checked by the tests of each.

## Resolved questions

- **A "no" to the menu entry.** The installer creates the empty file `~/.sprixel/no-menu-entry` when the person says no or passes `--no-menu`. The app does not register its entry at start while that file exists, and `--register-menu` removes it (see `app-menu-entries`, decision 8). Approved by the person.
