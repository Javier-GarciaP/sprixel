# Tasks

## 1. Linux entry in the app

- [x] 1.1 Write tests for the text of the `.desktop` file (name, quoted `Exec`, `Icon`, category) and for refusing a path with a newline, then implement it as a pure function
- [x] 1.2 Write tests with a temporary home folder for creating the entry and the icon, for "already correct writes nothing", and for "app moved rewrites it with no duplicate", then implement the registration
- [x] 1.3 Embed the icon in the app and verify the written PNG is the Sprixel icon
- [x] 1.4 Make a run without `APPIMAGE` (a dev run) never register at start, and verify it with a test

## 2. The `--register-menu` argument

- [x] 2.1 Write tests for reading the argument, then make `main` register, print one line and exit with 0 before it builds the window or starts the AI server
- [x] 2.2 Register at start in a background thread when the entry is missing or different, and verify the window opens even if the write fails

## 3. Windows shortcut

- [x] 3.1 Write tests for the PowerShell text that creates the shortcut (target, icon, name) and for the marker file logic, then implement them
- [x] 3.2 Create the shortcut in the user's Start Menu from a hidden PowerShell, only in release builds at start, and verify with a Windows-only test in CI
- [x] 3.3 Verify that `--register-menu` always rewrites the shortcut and the marker

## 4. Install scripts

- [x] 4.1 Make `install.sh` run `sprixel-app --register-menu` after copying the Linux app, with a warning (not an error) when it cannot, and verify with the existing script tests and a fake home
- [x] 4.2 Make `install.ps1` run `Sprixel.exe --register-menu` after copying the app, with the same warning rule
- [x] 4.3 Verify that the macOS path of `install.sh` adds no menu step

## 5. The `sprixel` command

- [x] 5.1 Add the menu files to `Layout` and verify `LayoutFor` for each system
- [x] 5.2 Make `sprixel uninstall` remove them and verify with a fake home that projects and `~/.sprixel` stay
- [x] 5.3 Make `sprixel update` call `--register-menu` after it replaces the app, with a time limit, and verify the warning path when the call fails

## 6. Documentation

- [x] 6.1 Update "Where the installer puts things" in `docs/usage.md` with the menu files, and fix every "open Sprixel from the applications menu" sentence in the README, `README.es.md` and both guides
- [x] 6.2 Add to the manual download steps of both guides how the entry appears (start the app once, or run it with `--register-menu`), and verify `install-docs.test.js` still passes
- [x] 6.3 Write the changelog entry, including what is not verified on Windows and macOS

## 7. Check by hand and release

- [ ] 7.1 On a real Linux desktop, check the entry and the icon after `install.sh`, after a manual download and after `sprixel update`, the dock grouping (X11 and Wayland, adding `StartupWMClass` if needed) and `sprixel uninstall`
- [ ] 7.2 Check on macOS that Spotlight and Launchpad show Sprixel with its icon, and write down the result
- [ ] 7.3 Run `openspec validate --all --strict`, then prepare release 0.1.1 following the release steps
