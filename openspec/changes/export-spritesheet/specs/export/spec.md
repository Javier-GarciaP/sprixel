# Spec Delta

## Purpose

Defines how sprites are exported to files that a game engine can use, and how
the result and any problem are reported to the user.

## ADDED Requirements

### Requirement: Scope
The user SHALL choose to export the whole project, one folder (with its sub-folders) or a selection of sprites, and the Export tab MUST show how many sprites and frames will be exported.

#### Scenario: Export a folder
- **WHEN** the user chooses the folder "Personajes" with 4 sprites and 14 frames
- **THEN** the tab shows "Personajes - 4 sprites" and "14 frames - 2 files"

### Requirement: Sprite sheet and JSON
The "Sheet + JSON" format SHALL write one PNG with all the frames packed in a grid without overlapping, and one JSON file with, for every frame, its name, position and size in the sheet and its duration, and for every animation its name, first and last frame and repeat mode, plus the sheet size and the scale.

#### Scenario: Frames in the JSON
- **WHEN** a folder with 14 frames of 8x8 is exported with spacing 1
- **THEN** the PNG is 71x17 pixels and the JSON lists 14 frames with their positions

### Requirement: Same result every time
Exporting the same project with the same options SHALL produce byte-identical files, with frames ordered by sprite name and frame number.

#### Scenario: Export twice
- **WHEN** the same export is run twice
- **THEN** both runs produce identical files

### Requirement: Single PNG files
The "PNG files" format SHALL write one PNG per frame, named with the naming pattern (default `{sprite}_{animation}_{frame}`).

#### Scenario: Naming
- **WHEN** the sprite "goblin" with animation "walk" is exported as PNG files
- **THEN** files like `goblin_walk_1.png` to `goblin_walk_4.png` are written

### Requirement: GIF
The GIF format SHALL write one animated GIF per animation using its speed and repeat mode, and for a sprite without animations one GIF with all its frames, with transparency preserved.

#### Scenario: Export a walk GIF
- **WHEN** "walk" (12 fps, repeat) is exported as a GIF
- **THEN** the GIF loops with frames about 83 ms long

### Requirement: Options
The user SHALL choose a scale of 1x, 2x, 4x or 8x with exact nearest-neighbor scaling, the spacing between frames from 0 to 16 pixels, the output folder (inside the project, `assets/sprites` by default), and in "More options" trimming of transparent edges, a pivot per sprite and the naming pattern.

#### Scenario: Scale 4x
- **WHEN** a 64x64 sprite is exported at 4x
- **THEN** the image is 256x256 with sharp pixels

### Requirement: Output location
Files SHALL be written only inside the project folder. The output folder MUST NOT be allowed to point outside it, and files with the same names from an earlier export MUST be replaced.

#### Scenario: Path outside the project
- **WHEN** the output folder is set to `../../etc`
- **THEN** the export is refused with a message that the folder must be inside the project

### Requirement: Result
When an export finishes, the app SHALL show a success message with the number of files and the folder and a button to open that folder, and an AI export MUST appear in the activity list.

#### Scenario: Success
- **WHEN** an export of 2 files finishes
- **THEN** a green message says "2 files saved" with an "Open folder" button

### Requirement: Errors
When an export fails (no permission, no space, invalid path), the app SHALL show the reason, keep the options, write no partial files, and offer Retry and Change folder.

#### Scenario: No permission
- **WHEN** the output folder is read-only
- **THEN** a red message says there is no permission to write in that folder and Retry is offered

### Requirement: Nothing to export
When the selected scope has no sprites, the Export button SHALL be disabled and the tab MUST explain what to do.

#### Scenario: Empty project
- **WHEN** the project has no sprites
- **THEN** the steps are dimmed, Export is disabled and "Go to project" is offered
