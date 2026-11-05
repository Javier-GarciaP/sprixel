# Spec Delta

## Purpose
Sprixel appears in the applications menu of Linux, Windows and macOS with its icon, however it was installed, and the entry is kept correct and removed with the app.

## ADDED Requirements

### Requirement: Linux menu entry
On Linux, Sprixel SHALL have an entry named "Sprixel" in the applications menu of the current user, in the Graphics category, that starts the installed app and shows the Sprixel icon. The entry and the icon MUST be written only inside the user's own folders, with no administrator rights.

#### Scenario: Entry after the install script
- **WHEN** a person installs Sprixel on Linux with `install.sh`
- **THEN** "Sprixel" with its icon is in the applications menu without starting the app first

#### Scenario: Entry starts the app
- **WHEN** the person chooses "Sprixel" in the menu
- **THEN** the same app that `sprixel start` opens is started

### Requirement: Windows Start Menu shortcut
On Windows, Sprixel SHALL have a shortcut named "Sprixel" in the Start Menu of the current user that starts the installed app and shows the Sprixel icon. The shortcut MUST be written only in the user's own Start Menu folder, with no administrator rights.

#### Scenario: Shortcut after the install script
- **WHEN** a person installs Sprixel on Windows with `install.ps1`
- **THEN** "Sprixel" with its icon is in the Start Menu without starting the app first

### Requirement: macOS app bundle
On macOS, the app bundle `Sprixel.app` in `~/Applications` SHALL be the menu entry, with the Sprixel icon. Sprixel MUST NOT create any extra file for it.

#### Scenario: App shows in Launchpad and Spotlight
- **WHEN** a person installs Sprixel on macOS with `install.sh`
- **THEN** Sprixel is found by name in Spotlight and shows its icon in Launchpad

### Requirement: Same result for every way of installing
The menu entry SHALL exist whether Sprixel was installed with the install script or by downloading the files from the GitHub Release and putting them in place by hand. When the app starts and its entry is missing, the app MUST create it.

#### Scenario: Manual download
- **WHEN** a person puts the Linux or Windows app in place by hand and starts it once
- **THEN** the entry exists afterwards, with the icon

#### Scenario: Entry already correct
- **WHEN** the app starts and its entry already points to this app
- **THEN** nothing is written again

### Requirement: Register without opening a window
The app SHALL accept the argument `--register-menu`. With it, the app creates or repairs its menu entry, prints one line saying what it did, and exits without opening a window or starting the AI server. The install scripts and `sprixel update` MUST use it right after they copy the app.

#### Scenario: Register only
- **WHEN** `--register-menu` is given to the app
- **THEN** the entry is created, no window opens and the exit code is 0

#### Scenario: Update refreshes the entry
- **WHEN** `sprixel update` replaces the app
- **THEN** the menu entry is created or repaired in the same run

### Requirement: Opt out
While the file `~/.sprixel/no-menu-entry` exists, the app MUST NOT create or rewrite its menu entry when it starts. The argument `--register-menu` is an explicit request: it SHALL create the entry and remove that file.

#### Scenario: The person said no in the installer
- **WHEN** the file `~/.sprixel/no-menu-entry` exists and the app starts
- **THEN** no menu entry is created

#### Scenario: The person asks for it later
- **WHEN** `--register-menu` is given while that file exists
- **THEN** the entry is created and the file is removed

### Requirement: The entry follows the app
The entry SHALL always point to the app that registered it. If the app is in another place than the one the entry names, registering MUST rewrite the entry, and there MUST be one entry only, never a duplicate.

#### Scenario: App moved
- **WHEN** the app was moved to another folder and starts
- **THEN** the entry points to the new place and no second entry exists

### Requirement: A failure never blocks
If the entry cannot be written (a read-only home folder, a missing PowerShell, an app that cannot start in a build step), the install and the app MUST continue. The person gets one short message that says what was not done and how to retry, and the exit code of the install is not changed.

#### Scenario: Read-only menu folder
- **WHEN** the menu folder is not writable
- **THEN** the app starts normally and the install script ends successfully with a warning

### Requirement: Uninstall removes the entry
`sprixel uninstall` SHALL remove the menu entry and the icon that Sprixel created. It MUST NOT touch the projects of the person or the local data folder `~/.sprixel`, except as it already does today.

#### Scenario: Uninstall
- **WHEN** a person runs `sprixel uninstall` and confirms
- **THEN** "Sprixel" is gone from the menu and the person's projects are untouched
