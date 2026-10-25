# sprite-management Specification

## Purpose
Defines how a person creates and organizes sprites and folders, and how deleting
is made safe without making everyday work slow.

## Requirements

### Requirement: New sprite
The New sprite dialog SHALL ask for a name, a folder, a size and, optionally, a palette. When no palette is chosen, the sprite MUST use the palette `main` of the project, which the app creates when it does not exist and which grows with the colors that are used. The size MUST be chosen from the presets 8x8, 16x16, 32x32 and 64x64 or typed as any width and height from 1 to 512, and typing a size MUST clear the selected preset.

#### Scenario: Custom size
- **WHEN** the user types 120 and 30 as width and height and presses "Create sprite"
- **THEN** a sprite of 120x30 is created in the chosen folder and opens in the editor

#### Scenario: Preset fills the fields
- **WHEN** the user selects the preset 16x16
- **THEN** the width and height fields show 16 and 16

#### Scenario: No palette chosen
- **WHEN** the user creates a sprite with "No palette" in a project that has no palettes
- **THEN** the project gets a palette `main` and the sprite uses it

### Requirement: Large size warning
When the width or height is 128 or more, the New sprite dialog SHALL show a warning that drawing with an AI is less accurate and the editor can be slower, and MUST still allow the sprite to be created.

#### Scenario: A 256x256 sprite
- **WHEN** the user types 256 and 256
- **THEN** the warning appears and "Create sprite" stays enabled

### Requirement: Name validation
Creating or renaming a sprite or a folder SHALL show an error next to the field when the name is empty, invalid or already used in that folder, and the confirm button MUST stay disabled until the name is valid.

#### Scenario: Name already exists
- **WHEN** the user types "Enemigos" in New folder and that folder exists
- **THEN** the field turns red with the message "A folder with that name already exists" and the create button is disabled

### Requirement: New folder
A folder SHALL be created from the Project tab inside the current folder or the project root, chosen in the dialog.

#### Scenario: Create a folder
- **WHEN** the user creates the folder "Enemigos" in the root
- **THEN** it appears in the tree and a matching folder is created in the project's `sprites/` folder

### Requirement: Rename
Sprites and folders SHALL be renamed with the menu, the F2 key or by double-clicking the name, with the same validation as creation, and renaming a sprite MUST keep its content and history.

#### Scenario: Rename with F2
- **WHEN** a sprite is selected and the user presses F2, types "orc" and confirms
- **THEN** the sprite is shown as "orc" and its file is renamed

### Requirement: Duplicate
Duplicating a sprite SHALL create a copy in the same folder with a unique name ("goblin copy", "goblin copy 2"...) and all frames and animations.

#### Scenario: Duplicate twice
- **WHEN** the user duplicates "goblin" twice
- **THEN** "goblin copy" and "goblin copy 2" exist with the same content

### Requirement: Move
Sprites SHALL be moved to another folder from the "Move to folder" menu (which can also create a folder) or by dragging them onto a folder, and moving MUST fail with a clear message if the name already exists in the destination.

#### Scenario: Move to a folder
- **WHEN** the user moves "slime" to "Enemigos" from the menu
- **THEN** the sprite appears in "Enemigos" and no longer in the old folder

### Requirement: Delete a sprite
Deleting one sprite SHALL NOT ask for confirmation. It MUST move the sprite to the trash and show a message "<name> moved to the trash" with an Undo action for 8 seconds, and Ctrl+Z MUST also restore it.

#### Scenario: Delete and undo
- **WHEN** the user presses Delete on the sprite "goblin" and then presses "Undo" in the message
- **THEN** "goblin" is back in its folder with all its content

### Requirement: Delete folders and groups
Deleting a folder or several sprites at once SHALL ask for confirmation, with the focus on "Cancel", listing what will be moved to the trash.

#### Scenario: Delete a folder
- **WHEN** the user deletes the folder "Personajes" that has 4 sprites
- **THEN** a dialog asks "Delete the folder 'Personajes'?" and explains that it and its 4 sprites go to the trash

### Requirement: Multiple selection
The user SHALL select several sprites with the checkboxes or with Ctrl and Shift clicks, see an action bar with the count, Move, Delete and Cancel selection, and see the combined details on the right.

#### Scenario: Select three sprites
- **WHEN** the user checks three sprites
- **THEN** the bar shows "3 selected" with Move and Delete, and the details panel lists the three names and their total frames

### Requirement: Ways to start an action
Rename, duplicate, move and delete SHALL be available from the "..." button on a card, the right-click menu, the details panel buttons and the keyboard (F2, Ctrl+D, Delete).

#### Scenario: Right-click
- **WHEN** the user right-clicks a sprite card
- **THEN** a menu opens next to it with Open in editor, Rename, Duplicate, Move to folder and Delete
