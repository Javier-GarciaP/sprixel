# Spec Delta

## Purpose

Defines how the Project tab shows the contents of a project so that a person can
find and select any sprite quickly.

## ADDED Requirements

### Requirement: Folders tree
The Project tab SHALL show a tree with the folders and the sprites of the project, with the number of items next to each folder and the number of frames next to each sprite, and let the user expand and collapse folders.

#### Scenario: Browse folders
- **WHEN** the user selects the folder "Personajes"
- **THEN** the tree highlights it and the middle area lists its sprites

### Requirement: Sprite cards
The middle area SHALL show the sprites of the selected folder as cards with a thumbnail of the first frame, the name, the size and the number of frames, and a "New sprite" card at the end.

#### Scenario: Card information
- **WHEN** a folder has a sprite "goblin" of 8x8 with 4 frames
- **THEN** its card shows the thumbnail, "goblin" and "8x8 - 4 frames"

### Requirement: Sprite details
Selecting a sprite SHALL show its details on the right: a large preview, size, frames, palette, the list of its animations (name, frames, speed, repeat mode) and who edited it last, with the actions Open in editor, Duplicate and Delete.

#### Scenario: Last edited by an AI
- **WHEN** the last operation on a sprite was made by an AI client
- **THEN** the details show "Edited by" with the client name and the time

### Requirement: Open a sprite
Double-clicking a card or pressing "Open in editor" SHALL open the sprite in the Editor tab.

#### Scenario: Double-click
- **WHEN** the user double-clicks the card "goblin"
- **THEN** the Editor tab opens with "goblin"

### Requirement: Search
The search field SHALL filter the sprites and folders of the whole project by name while the user types, ignoring case and accents.

#### Scenario: Search by name
- **WHEN** the user types "gob"
- **THEN** only the sprites and folders whose name contains "gob" are listed

### Requirement: Palettes list
The folders panel SHALL list the palettes of the project below the folders, with a button to create a palette and a click that opens the palette editor.

#### Scenario: Open a palette
- **WHEN** the user clicks the palette "dungeon16"
- **THEN** the palette editor opens for it

### Requirement: Empty states
When the project has no sprites, the Project tab SHALL show an empty message with one main action, "New sprite", and MUST NOT show the details panel.

#### Scenario: New project
- **WHEN** a project has no folders and no sprites
- **THEN** the tree shows "No folders yet" and the middle area shows the empty message with "New sprite"
