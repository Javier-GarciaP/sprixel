# Proposal

## Why

After installing Sprixel 0.1.0, the app does not show up in the applications menu on Linux and Windows, so the person has to start it from a terminal (`sprixel start`). On macOS the app already sits in `~/Applications`. A desktop app should appear in the menu with its icon, however it was installed: with the install script or by downloading the files from GitHub.

## What Changes

- On Linux, Sprixel gets a menu entry (a `.desktop` file) and its icon, for the current user only.
- On Windows, Sprixel gets a Start Menu shortcut that uses the icon of `Sprixel.exe`.
- On macOS, the app in `~/Applications` is the menu entry. This change only checks that it shows its icon and adds no files.
- The app can register its own entry. It does it when it starts, if the entry is missing or points to another place, and it can also do it on request without opening a window (`--register-menu`). So both ways of installing give the same result.
- The install scripts and `sprixel update` ask the app to register the entry right after copying it, so the entry exists before the first launch.
- `sprixel uninstall` removes the entry and the icon.
- A person can opt out: while the file `~/.sprixel/no-menu-entry` exists, the app does not register its entry by itself. The installer creates that file when the person says no to the menu entry (see `installer-experience`).
- The README and the guides stop saying "open Sprixel from your applications menu" for every system, and describe the real behavior.

## Capabilities

### New Capabilities
- `app-menu-entries`: how Sprixel appears in the applications menu of each system, how the entry is created, kept up to date and removed.

### Modified Capabilities

## Impact

- Rust app (`src-tauri/`): reads a `--register-menu` argument before it opens a window, and registers the entry at start. No new dependency: the icon is embedded in the app and Windows uses the PowerShell that comes with the system.
- `install.sh` and `install.ps1`: one extra best-effort step after copying the app.
- Go command (`cli/internal/release`, `update`, `uninstall`): the layout gets the paths of the entry and the icon; update refreshes them and uninstall removes them.
- Tests for each part (with a fake home folder), and the usage guide, the guides and the README.
- Needs a new release (0.1.1), because the install scripts are downloaded from the release. Someone who already has 0.1.0 gets the entry by running `sprixel update` or the installer again.
- Out of scope: native `.deb`, `.rpm`, `.msi` or `.dmg` installers, signing, and a settings screen for the entry (the file `no-menu-entry` is the only opt-out).
