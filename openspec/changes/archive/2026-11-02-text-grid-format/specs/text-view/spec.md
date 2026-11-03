# Spec Delta

## Purpose

Defines the Text view of the editor, where a person can read, edit, copy and
paste the text grid of a sprite with a live preview.

## ADDED Requirements

### Requirement: Switch views
The editor SHALL have a "Canvas | Text" switch in the same place in both views, and switching MUST keep the same sprite and frame.

#### Scenario: Switch to text
- **WHEN** the user switches to Text while editing frame 2
- **THEN** the Text view shows frame 2 of the same sprite

### Requirement: Show the sprite as text
The Text view SHALL show the palette name, the legend, the current frame number and the rows of the frame in a monospace font, with line numbers, and the frames of the sprite as tabs or chips.

#### Scenario: Open the view
- **WHEN** the Text view opens for the 8x8 goblin
- **THEN** it shows the legend and 8 rows of 8 characters

### Requirement: Live validation and preview
While the user types, the Text view SHALL validate the text and show a live preview of the result, and a status line that says "Valid" with the size and the number of keys, or the first error with its line.

#### Scenario: Type a valid change
- **WHEN** the user changes one character to another valid key
- **THEN** the preview updates and the status stays "Valid"

### Requirement: Errors in the text
Lines with errors SHALL be highlighted with the problem written next to them, the preview MUST keep showing the last valid state, and the Apply button MUST be disabled.

#### Scenario: A short row
- **WHEN** the user deletes a character from a row
- **THEN** that line is highlighted with "7 columns", the status shows the error and Apply is disabled

### Requirement: Apply, copy and paste
The Text view SHALL provide Apply (to write the text into the frame), Copy (the whole grid to the clipboard) and Paste (replace the text from the clipboard, then validate it).

#### Scenario: Paste a grid from an AI
- **WHEN** the user pastes a valid grid and presses Apply
- **THEN** the frame changes in one undoable action and the canvas shows it

### Requirement: Text for the AI is the same
The text shown and copied SHALL be exactly the format that the AI tools read and write, so that a grid copied from the app can be used by an AI and the reverse.

#### Scenario: Copy to an AI
- **WHEN** the user copies the grid and an AI applies it back with `apply_grid`
- **THEN** the sprite is unchanged
