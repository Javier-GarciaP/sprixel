# Spec Delta

## Purpose
The install scripts of Sprixel show what they will do, ask before they do it, report the progress of each download and still work with no terminal.

## ADDED Requirements

### Requirement: Summary before any download
The install scripts SHALL print, before they download anything, a welcome, the version, the system, the files that will be installed and the folders where they go, and the MIT license notice that they print today.

#### Scenario: Summary on a terminal
- **WHEN** a person runs the install command on a terminal
- **THEN** the screen shows the version, the system and the install folders before any download starts

### Requirement: Confirmation
On a terminal, the install scripts SHALL ask for confirmation after the summary, with yes as the default answer. If the person says no, nothing MUST be downloaded or changed and the exit code MUST be 0.

#### Scenario: The person declines
- **WHEN** the person answers no to the confirmation
- **THEN** the script says that nothing was installed and ends with exit code 0

#### Scenario: The person accepts
- **WHEN** the person presses Enter on the confirmation
- **THEN** the install goes on

### Requirement: Visible download progress
Each download SHALL show the name of the file, a progress bar with the percentage and a counter of the files ("2 of 5"). The progress MUST update while the file arrives, so a slow download does not look frozen.

#### Scenario: Big file
- **WHEN** the app file of about 80 MB is downloading
- **THEN** the bar moves and the percentage grows until the file is complete

#### Scenario: Checks still come first
- **WHEN** all the files are downloaded
- **THEN** the script checks every SHA-256 before it copies anything, and a wrong checksum installs nothing

### Requirement: Options
On a terminal, after the confirmation, the install scripts SHALL ask whether to create the applications menu entry, when the `app-menu-entries` capability exists, and whether to connect an AI now. The AI question SHALL offer only Claude Code and OpenCode when the computer has them, plus "none". The default answers are: create the entry, and connect none.

#### Scenario: Connect Claude Code
- **WHEN** the person chooses Claude Code and the install ends
- **THEN** Sprixel is registered in Claude Code, as `sprixel mcp add claude` does

#### Scenario: No AI tool found
- **WHEN** neither Claude Code nor OpenCode is on the computer
- **THEN** the question is not asked and the end of the install shows how to connect later

#### Scenario: Connecting fails
- **WHEN** registering the AI tool fails
- **THEN** the install still succeeds, with one warning that says how to retry

### Requirement: No terminal, no questions
When there is no terminal to ask on (the script runs in a pipeline with no keyboard, or in a test), or the person passes `--yes` or sets `SPRIXEL_YES=1`, the install scripts MUST NOT ask anything and MUST behave as the previous version: install with the default answers, create the menu entry, connect no AI. The exit codes and the error messages MUST NOT change.

#### Scenario: Release smoke test
- **WHEN** the release pipeline runs the script with no terminal
- **THEN** it installs without any question, exactly as before

#### Scenario: Flags for a script
- **WHEN** a script runs with `--yes --no-menu --connect claude`
- **THEN** it installs, skips the menu entry, registers Claude Code and asks nothing

### Requirement: Works from the usual one-line commands
The questions SHALL work when the script is run with `curl … | sh` and with `irm … | iex`, where the script itself comes through the input of the shell. The answers MUST be read from the terminal of the person, not from the script.

#### Scenario: One-line command
- **WHEN** a person runs the one-line command on a terminal
- **THEN** the questions appear and the typed answers are used

### Requirement: Plain output when it is not a terminal
Colors, bars made with cursor movement and other control codes SHALL be used only when the output is a terminal and `NO_COLOR` is not set. Otherwise the output MUST be plain text, one line per step, readable in a log.

#### Scenario: Output to a file
- **WHEN** the output of the script is saved to a file
- **THEN** the file has no control codes and the progress is shown as plain lines

### Requirement: Final summary
When the install ends, the scripts SHALL print what was installed and where, which choices were made, and the next steps (`sprixel doctor`, how to connect an AI, how to start the app). An error MUST keep its message today and end with a non-zero exit code.

#### Scenario: Successful install
- **WHEN** the install ends well
- **THEN** the summary lists the installed version, the folders and the next steps
