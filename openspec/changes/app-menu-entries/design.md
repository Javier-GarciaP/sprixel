# Design

## Context

- The install scripts copy the files and stop there. `install.sh` has no menu step, and `install.ps1` creates no shortcut (see `install.sh` and `install.ps1`).
- The Linux app is an AppImage kept as `~/.local/bin/sprixel-app`. The Windows app is `%LOCALAPPDATA%\Sprixel\Sprixel.exe`. On macOS it is `~/Applications/Sprixel.app` (a bundle with the icon).
- The release publishes the AppImage, a `.tar.gz` with the macOS app and a `.zip` with the Windows exe. It does not publish `.deb`, `.rpm`, `.msi` or `.dmg`.
- The icons already exist in `src-tauri/icons/` (`.png`, `.icns`, `.ico`). Tauri puts the `.ico` inside the Windows exe.
- The guide has a "download from GitHub" method where no script runs (see `docs/guide.md`). Any solution that lives only in the scripts misses that method.
- The install layout is in one place for the command: `LayoutFor` in `cli/internal/release/release.go`, used by `update` and `uninstall`.

## Goals / Non-Goals

**Goals:**
- One place that knows how to write the entry, so the script install, the manual download and `sprixel update` all give the same result.
- The entry exists right after the install, before the first launch.
- No new dependency and no new file in the release.
- Everything can be tested with a fake home folder.

**Non-Goals:**
- Native installers (`.deb`, `.rpm`, `.msi`, `.dmg`) and signing.
- A settings screen for the entry. The only opt-out is the file `~/.sprixel/no-menu-entry` (decision 8); `sprixel uninstall` removes the entry.
- Entries for other users or for the whole system.
- Registering a file type or a URL scheme.

## Decisions

**1. The app writes its own entry (not the scripts).**
The app knows its own path and carries its own icon, so it can register itself however it got there. The scripts only ask it to do so.
- *Alternative: the scripts write the files.* Simple, but the manual download would have no entry, and Linux would need the icon as a new release file.
- *Alternative: publish native installers.* They give the entry for free, but they add three release formats, and the `.rpm` and `.msi` bundlers already fail on pre-release versions. Too much for this goal.

**2. `--register-menu` registers and exits.**
`main` reads the argument before it builds the window or starts the AI server. It registers, prints one line and exits with 0, or prints the problem and exits with 0 too (see decision 6). The install scripts and `sprixel update` call it right after copying the app. The app also registers at start when the entry is missing or points elsewhere, in a background thread so it never delays the window.
- A Linux AppImage needs FUSE to run at all. If the scripts cannot run it, they warn and go on; the entry is then created the first time the person opens the app.

**3. Linux entry.**
- File `~/.local/share/applications/sprixel.desktop` (`Name=Sprixel`, `Exec` with the quoted path of the AppImage, `Icon=sprixel`, `Categories=Graphics;`, `Terminal=false`) and the icon in `~/.local/share/icons/hicolor/256x256/apps/sprixel.png`, embedded in the app with `include_bytes!`.
- The path comes from the `APPIMAGE` variable that an AppImage sets, so a dev run (`npm run tauri dev`) never registers: it has no `APPIMAGE`. A path with a newline or a quote that cannot be escaped is refused.
- "Already correct" means the `Exec` line of the existing file equals the one that would be written.
- After writing, run `update-desktop-database` and `gtk-update-icon-cache` if they exist, ignoring errors.

**4. Windows shortcut.**
- File `%APPDATA%\Microsoft\Windows\Start Menu\Programs\Sprixel.lnk`, created with the `WScript.Shell` object from a hidden `powershell.exe`, with the exe as target and as icon source. No dependency.
- Reading a `.lnk` is awkward, so the app keeps a marker file `%LOCALAPPDATA%\Sprixel\menu-entry.txt` with the exe path it registered. "Already correct" means the shortcut exists and the marker equals the current exe path.
- Only release builds register at start (`cfg!(not(debug_assertions))`); `--register-menu` always works.

**5. macOS: verify, do not build.**
The bundle in `~/Applications` is indexed by Spotlight and Launchpad. The task is a manual check; if the icon is missing, that is a bug of the bundle, fixed in its own change.

**6. Never fail the install or the app.**
Every step that writes the entry returns a result that the caller turns into one warning line. Nothing in this path can stop the window from opening or make an install script exit with an error.

**7. `update` and `uninstall` use the layout.**
`Layout` gets the list of files the app may create (the `.desktop` and the icon on Linux; the shortcut and the marker on Windows). `uninstall` removes them with the files it already removes. `update` calls `--register-menu` after it replaces the app, with a time limit.

**8. Opt out with a marker file.**
At start the app checks `~/.sprixel/no-menu-entry` first and does nothing when it exists. `--register-menu` removes the file and registers. The installer creates the file when the person says no (see `installer-experience`). Windows and Linux read the same file, in the user's home folder.

## Risks / Trade-offs

- **The app writes outside its own folder.** It is the user's own menu folder and it is documented, but it is a new behavior of a program that only opened a window before. Mitigated by writing only when missing or different, and by `sprixel uninstall`.
- **PowerShell at start (Windows).** About a third of a second, so it runs in a background thread and only when the shortcut is missing or the marker differs. Some antivirus tools dislike scripts started by an app; this is unverified.
- **Dock grouping on Linux.** The window may not match the entry (no `StartupWMClass`), so the dock could show a second icon. Verify on a real Linux desktop (X11 and Wayland) and add `StartupWMClass` if needed.
- **Only Linux can be tried by hand here.** Windows is covered by unit tests of the generated script and a Windows-only test in CI; macOS has no code. Both stay "not verified on a real machine" in the changelog.
- **The fix reaches users only with a new release.** `install.sh` and `install.ps1` come from the release, so people on 0.1.0 need `sprixel update` or the installer again.

## Assumption (to confirm)

- "The normal installer downloaded from GitHub" is read as the manual download of the files of the Release (method 2 of the guide). If the wish is real installers (`.deb`, `.msi`), that is a different change, and this design would still serve as the fallback for the AppImage.
