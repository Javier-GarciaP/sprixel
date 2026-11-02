# Spec Delta

## Purpose

Defines how palettes work: each color has a name, a one-letter key and a hex
value, and changes to a palette stay consistent in every sprite that uses it.
Palettes are not required to draw: a palette grows by itself with the colors
that are used, and the one-letter keys are the language an AI draws with.

## ADDED Requirements

### Requirement: Color definition
A palette SHALL contain up to 93 colors and MAY be empty. Each color MUST have a unique name, a unique one-character key that is a printable ASCII character other than a space and the dot (case sensitive), and a hex value.

#### Scenario: Reserved key
- **WHEN** the user tries to use "." or a space as a key
- **THEN** the editor refuses it and explains that "." means transparent

#### Scenario: Duplicate key
- **WHEN** the user types a key that another color already uses
- **THEN** the field shows an error and the change is not applied

### Requirement: A sprite always has a palette
A sprite SHALL always have a palette. When the project has no palette, the core MUST create one called `main` the first time a color is needed.

#### Scenario: First color of a new project
- **WHEN** a person draws the first pixel with `#1F2A44` in a sprite created with "No palette"
- **THEN** the project has a palette `main` with one color, key `A`, name `1F2A44`, and the pixel uses `A`

### Requirement: A palette that grows
When a color that is not in the palette is used, the core MUST add it with the next free key in this order: `A` to `Z`, `a` to `z`, `0` to `9`, then the other printable ASCII characters except the space and the dot, and the name of an added color MUST be its hex value. When the palette has 93 colors, adding a color MUST fail with the error `palette_full` and nothing changes.

#### Scenario: Next free key
- **WHEN** the palette already uses `A` to `Z` and a new color is added
- **THEN** the new color gets the key `a`

#### Scenario: Palette is full
- **WHEN** the palette has 93 colors and another color is added
- **THEN** the error `palette_full` is returned and the palette and the sprite are unchanged

### Requirement: Palette panel
The editor SHALL show the palette colors in the right panel, select the main color on click and the secondary color on right-click, show a transparent entry, and offer a "Show keys" switch that writes each key on its color.

#### Scenario: Show keys
- **WHEN** the user turns "Show keys" on
- **THEN** every color shows its letter

### Requirement: Edit a color
The palette editor SHALL let the user change the name, key and color (hex or RGB sliders) of the selected color, and a change of color value MUST update all sprites that use it immediately.

#### Scenario: Change a color
- **WHEN** the user changes the red from #F04545 to #D03030
- **THEN** every sprite that uses that key shows the new color

### Requirement: Change a key safely
Changing the key of a color SHALL update the stored pixels of every sprite that uses the palette in one undoable action, and the editor MUST warn the user that text grids written with the old key will become invalid.

#### Scenario: Rename the key R to H
- **WHEN** the user changes the key of "red" from R to H
- **THEN** the pixels of every sprite using it keep their look and a warning explains that grids using "R" will stop being valid

### Requirement: Add and remove colors
The user SHALL add colors up to the limit, and remove a color only if no sprite uses it, or after choosing a replacement color for the pixels that use it.

#### Scenario: Remove a used color
- **WHEN** the user removes a color that sprites use
- **THEN** the editor asks for a replacement and applies it to all those pixels

### Requirement: Create and import palettes
The user SHALL create a palette empty or from a built-in preset, and import colors from `.gpl` (GIMP) and `.hex` (one hex value per line) files, giving every imported color a generated key and name that the user can edit.

#### Scenario: Create from a preset
- **WHEN** the user creates a palette from the preset "Pocket RPG 16" or "Farm 32"
- **THEN** a new palette with 16 or 32 colors is created, each with a unique key and a name, and the preset names contain no trademarks

#### Scenario: Import a hex file
- **WHEN** the user imports a `.hex` file with 16 lines
- **THEN** a new palette with 16 colors is created, each with a unique key

### Requirement: Cancel and apply
The palette editor SHALL have Cancel and Done buttons: Cancel MUST discard all changes made in the dialog and Done MUST apply them as one undoable action.

#### Scenario: Cancel
- **WHEN** the user changes two colors and presses Cancel
- **THEN** the palette and the sprites are unchanged

### Requirement: Editing while an agent works
While an agent is writing to a sprite that uses the palette, key changes SHALL be blocked with a message, and color value changes MAY be applied.

#### Scenario: Agent is drawing
- **WHEN** an agent is writing and the user tries to change a key
- **THEN** the editor says to wait or stop the agent first
