# Proposal

## Why

The install command prints four lines, then downloads about 80 MB with no sign of life. In the first real test it took 4 minutes and looked frozen. The person also cannot see what will be installed, cannot say no, and has to run more commands afterwards to connect an AI. The first contact with Sprixel should feel like the installers people already know (for example the one of Astro): a clear summary, a confirmation, a few choices and a visible progress.

## What Changes

- `install.sh` and `install.ps1` show a welcome and a summary (version, system, what is installed and where), then ask for confirmation before they download anything.
- Each download shows its name, a progress bar with a percentage and a counter ("2 of 5").
- Two questions are asked after the confirmation: create the applications menu entry (see `app-menu-entries`) and connect an AI now (none, Claude Code, OpenCode or both, offering only the tools found on the computer).
- When there is no terminal to ask, or the person passes `--yes` (or `SPRIXEL_YES=1`), nothing is asked and the install behaves as today, so the release smoke tests and any script keep working.
- Options for scripts: `--yes`, `--no-menu`, `--connect claude|opencode|both|none`, with equivalent environment variables for PowerShell.
- The end shows a summary of what was installed and the next steps. Colors and cursor codes are used only on a real terminal and never when `NO_COLOR` is set.
- The checks of today stay: user-only install, https only, SHA-256 before anything is copied, nothing installed on a failure.

## Capabilities

### New Capabilities
- `installer-experience`: what the install scripts show and ask, how they report progress, and how they behave with and without a terminal.

### Modified Capabilities

## Impact

- `install.sh` and `install.ps1` (the scripts that ship in the release). No new file, no dependency and no extra program.
- The script tests (`scripts/install-sh.test.js`, `scripts/install-ps1.test.js`) and the install documentation (README, `README.es.md`, both guides, `docs/usage.md`).
- The release smoke tests must stay unchanged: they run the scripts with no terminal.
- Depends on `app-menu-entries` only for the menu question. Until that change is built the question is not shown.
- Needs a new release, because the scripts are downloaded from the release.
- Out of scope: arrow-key menus, a separate installer program, a graphical installer and native `.exe`, `.msi` or `.dmg` installers.
