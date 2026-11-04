# project-storage Specification

## Purpose

Defines how a project is stored as plain text files, saved automatically and
safely, and kept working across app versions and outside changes.

## Requirements

### Requirement: Project folder format
A project SHALL be stored as a folder with a `sprixel.json` manifest (name and `formatVersion`), a `palettes/` folder with one `*.palette.json` per palette, a `sprites/` folder whose sub-folders mirror the project folders with one `*.sprite.json` per sprite, and a `.sprixel/` folder for internal data.

#### Scenario: Create a project
- **WHEN** a project named "Dungeon Kit" is created in `~/Proyectos`
- **THEN** the folder `~/Proyectos/Dungeon Kit` is created with a valid `sprixel.json`, a `palettes/` folder and a `sprites/` folder

### Requirement: Readable sprite files
A sprite file SHALL be JSON text that stores each frame as an array of row strings of palette keys, together with its size, palette name, frame durations, layers and animation tags, so that the files are readable and produce clean diffs in Git.

#### Scenario: Change one pixel
- **WHEN** one pixel of a sprite changes and the project is saved
- **THEN** the sprite file differs from the previous one only in the affected row

### Requirement: Automatic saving
Changes SHALL be saved automatically within one second after the last change, without user action, and the interface MUST show whether the project is saved, saving or has a save error.

#### Scenario: Draw and wait
- **WHEN** the user finishes a stroke and waits one second
- **THEN** the sprite file on disk contains the stroke and the status shows "Saved"

### Requirement: Safe writes
Files SHALL be written atomically (to a temporary file that then replaces the old one), so that a crash or power loss never leaves a half-written file.

#### Scenario: App closes while saving
- **WHEN** the app process is killed during a save
- **THEN** the sprite file on disk is either the full old version or the full new version

### Requirement: Save errors
When a file cannot be saved, the app SHALL keep the changes in memory, show the reason (for example no space or no permission), allow the user to retry and MUST NOT discard the unsaved data.

#### Scenario: Disk full
- **WHEN** a save fails because the disk is full
- **THEN** the status shows "Error saving" with the reason and a Retry action, and the changes remain in the editor

### Requirement: Trash
Deleting a sprite or folder SHALL move its files to `.sprixel/trash/` inside the project instead of erasing them, and undoing the deletion MUST restore them to their original place.

#### Scenario: Delete and undo
- **WHEN** a sprite is deleted and the user presses Undo
- **THEN** the sprite file is back in its original folder with the same content

### Requirement: Changes outside the app
The app SHALL detect when a project file changes on disk while it is open. If the app has no unsaved changes it MUST reload the file; if it has unsaved changes it MUST show a notice offering to reload or keep the app version.

#### Scenario: Git pull while the app is open
- **WHEN** a sprite file changes on disk and the app has no pending changes for it
- **THEN** the sprite is reloaded and shown updated

### Requirement: Format versions and migration
Every file SHALL carry a `formatVersion`. Opening a project with an older version MUST migrate it after making a backup in `.sprixel/backups/`, and opening a project with a newer version MUST be refused with a message that asks the user to update the app.

#### Scenario: Open an older project
- **WHEN** a project with `formatVersion` lower than the current one is opened
- **THEN** a backup is made, the project is migrated and it opens normally

#### Scenario: Open a newer project
- **WHEN** a project with a higher `formatVersion` is opened
- **THEN** it is not modified and a message explains that a newer Sprixel is needed

### Requirement: Damaged files
If one sprite or palette file is invalid, the project SHALL still open, the damaged file MUST be listed with its path and the reason, and the other files MUST load normally.

#### Scenario: One broken sprite
- **WHEN** a sprite file contains invalid JSON
- **THEN** the project opens, the sprite is marked as damaged and a message shows the file path and the error

### Requirement: Project size
Opening a project with 500 sprites SHALL list all of them within 2 seconds by loading pixel data only when a sprite is needed.

#### Scenario: Large project
- **WHEN** a project with 500 sprites of 32x32 is opened
- **THEN** the project browser lists all sprites in 2 seconds or less
