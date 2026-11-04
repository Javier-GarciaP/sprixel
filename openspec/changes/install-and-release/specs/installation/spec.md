# Spec Delta

## Purpose

Defines how a person installs, updates and removes Sprixel with commands, on
Linux, Windows and macOS, safely and without administrator rights.

## ADDED Requirements

### Requirement: One command install
The install scripts SHALL install the app and the `sprixel` command for the current user from the latest GitHub Release (or a version given with `--version`), without administrator rights, and print the next steps (`sprixel doctor` and `sprixel mcp add <client>`).

#### Scenario: Install on Linux
- **WHEN** the user runs the documented `curl ... | sh` command
- **THEN** the app and `sprixel` are installed and the output shows the next steps

#### Scenario: Install on Windows
- **WHEN** the user runs the documented PowerShell command
- **THEN** the app and `sprixel` are installed and `sprixel` is available in a new terminal

### Requirement: Detect the system
The scripts SHALL detect the operating system and the CPU type, choose the right files, and stop with a clear message when the system is not supported.

#### Scenario: Unsupported system
- **WHEN** the script runs on an unsupported system
- **THEN** it stops and lists the supported systems

### Requirement: Verify downloads
The scripts SHALL download the checksums file of the release, verify the SHA-256 of every downloaded file, and stop without installing anything when a check fails.

#### Scenario: Corrupted download
- **WHEN** a downloaded file does not match its checksum
- **THEN** nothing is installed and the script says why

### Requirement: Locations and PATH
The scripts SHALL install into user folders (for example `~/.local` on Linux and macOS and `%LOCALAPPDATA%` on Windows), add the `sprixel` command to the user PATH when needed, and tell the user if a new terminal is required.

#### Scenario: PATH update
- **WHEN** the install folder is not in the PATH
- **THEN** the script adds it for the user and says to open a new terminal

### Requirement: Safe to repeat
Running the installer again SHALL upgrade or repair the installation and MUST keep the user's projects, settings, tokens and approved clients.

#### Scenario: Reinstall
- **WHEN** the installer is run on an existing installation
- **THEN** the version is updated and settings and projects are untouched

### Requirement: Update
`sprixel update` SHALL check for the latest release, install it with the same checks as the installer, and say which version it moved from and to, or say that it is up to date.

#### Scenario: Up to date
- **WHEN** the latest version is already installed
- **THEN** the command says so and changes nothing

### Requirement: Uninstall
`sprixel uninstall` SHALL remove the app, the command, the PATH entry and the registrations in AI tools, ask before doing it, and keep the user's projects and ask separately whether to remove the local data folder.

#### Scenario: Uninstall keeps projects
- **WHEN** the user confirms the uninstall
- **THEN** the program files are removed and the project folders are untouched

### Requirement: License notice
The installers SHALL show the license notice (MIT License, open source) before installing, and every release bundle MUST include the `LICENSE` file and the third-party notices from `docs/third-party.md`, which the fonts (SIL Open Font License) require.

#### Scenario: Notice shown
- **WHEN** the installer starts
- **THEN** it prints that the software is released under the MIT License and where to read the license and the third-party notices
