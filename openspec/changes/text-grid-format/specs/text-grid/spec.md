# Spec Delta

## Purpose

Defines the text grid format used to read and write sprites as plain text, and
how mistakes are reported so they can be corrected automatically.

## ADDED Requirements

### Requirement: Format
A text grid SHALL consist of an optional `palette: <name>` line, an optional `legend` block and one or more `frame <n>` blocks, each followed by rows of characters where every character is the key of a palette color and `.` is transparent. Keywords MUST be English, blank lines ignored, and lines starting with `#` treated as comments.

#### Scenario: Parse the example
- **WHEN** the 8x8 goblin grid from the documentation is parsed with its palette
- **THEN** it gives one frame of 8 rows of 8 pixels with the expected colors

### Requirement: Round trip
Writing a sprite as a text grid and parsing it again SHALL give the same pixels, for any sprite up to 512x512.

#### Scenario: Round trip
- **WHEN** a sprite is written to text and parsed back
- **THEN** every pixel and the size are identical

### Requirement: Size rules
All rows of a frame MUST have the same length, the number of rows MUST match the height, the size MUST match the sprite (or the region), and the grid MUST NOT exceed 512x512.

#### Scenario: Short row
- **WHEN** row 7 of an 8x8 frame has 7 characters
- **THEN** the error `row_length` is returned with the message "line 15 (row 7 of frame 1): 7 columns, expected 8"

### Requirement: Key rules
Every character other than `.` MUST be a key of the sprite's palette, case sensitive.

#### Scenario: Unknown key
- **WHEN** a row contains the character "X" and the palette has no key "X"
- **THEN** the error `unknown_key` is returned with the line, row and column

### Requirement: Error reporting
Every validation error SHALL include a code (`row_length`, `row_count`, `unknown_key`, `no_frame`, `bad_frame`, `too_big`, `region_out`), a human readable message and the line, row and column when they apply, and when a grid is invalid NOTHING MUST be applied.

#### Scenario: Several errors
- **WHEN** a grid has two bad rows
- **THEN** all errors are reported together and the sprite stays unchanged

### Requirement: Frame blocks
Frames SHALL be numbered from 1. A grid that writes into a frame number that does not exist MUST fail with `bad_frame`, and frames not mentioned MUST be left unchanged.

#### Scenario: Frame 5 of 4
- **WHEN** a grid writes `frame 5` on a sprite with 4 frames
- **THEN** the error `bad_frame` says the sprite has 4 frames

### Requirement: Region writing
A grid SHALL be writable at an `x`,`y` position (default 0,0), only if the whole region fits inside the sprite, with an option to keep existing pixels where the grid has transparent pixels.

#### Scenario: Region does not fit
- **WHEN** an 8x8 grid is written at (10, 2) on a 16x8 sprite
- **THEN** the error `region_out` is returned and nothing changes

#### Scenario: Keep existing pixels
- **WHEN** a region is written with the keep-existing option
- **THEN** the "." pixels of the grid leave the existing pixels unchanged

### Requirement: Applying is one action
Applying a valid grid SHALL be one undoable history entry, whoever applies it.

#### Scenario: Undo an applied grid
- **WHEN** a grid is applied and the user presses undo
- **THEN** the frame returns to how it was before in one step
