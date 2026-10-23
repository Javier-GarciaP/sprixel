# Spec Delta

## Purpose

Defines the tools an AI can call, what each one does, and the exact errors it
returns, so that an AI can work reliably and fix its own mistakes.

## ADDED Requirements

### Requirement: Reading tools
The server SHALL provide the read tools `get_project`, `list_sprites`, `get_sprite` (the sprite as a text grid), `get_palette`, `render_preview` (an image of a sprite or frame at a chosen scale) and `open_sprite` (shows a sprite in the editor), and read tools MUST NOT change the project.

#### Scenario: Read a sprite
- **WHEN** an agent calls `get_sprite` for "goblin"
- **THEN** it receives the size, the palette name, the frames as a text grid and the animations

#### Scenario: See the work
- **WHEN** an agent calls `render_preview` for frame 2
- **THEN** it receives an image of that frame

### Requirement: Creating tools
The server SHALL provide `create_folder`, `create_sprite` (name, folder, width, height, palette) and `add_frame` (empty or copied), enforcing the same rules and errors as the app (`name_exists`, `too_big`).

#### Scenario: Create a sprite
- **WHEN** an agent calls `create_sprite` for a 16x16 sprite named "orc" in "Enemigos"
- **THEN** the sprite exists in that folder and appears in the app

### Requirement: Drawing tools
The server SHALL provide `apply_grid` (a text grid, with a frame, an optional position and an option to keep existing pixels; a `legend` line with a hex value adds a color that is not in the palette, in the same atomic action), `set_pixels` (a list of x, y and key) and `draw_shape` (line, rectangle, ellipse or fill with a key), each applied as one atomic action.

#### Scenario: Apply a grid
- **WHEN** an agent calls `apply_grid` with a valid 8x8 grid
- **THEN** the frame changes and the call returns the new revision

#### Scenario: Apply a grid with a new color
- **WHEN** an agent calls `apply_grid` with a grid that uses the key "Q" and a legend line `Q teal #1FB5A8`, and the palette has no "Q"
- **THEN** the frame changes, the palette gets the color `Q` and the call returns the new revision

#### Scenario: Invalid grid
- **WHEN** the grid has a row of the wrong length
- **THEN** the call returns the error `row_length` with the line and nothing changes

### Requirement: Structure tools
The server SHALL provide `delete_frame`, `create_tag` (name, from, to, fps, mode), `set_palette_color` (add or change a color with its key, name and hex) and `delete_sprite`, with the same validation as the app.

#### Scenario: Create an animation
- **WHEN** an agent calls `create_tag` for "walk" frames 1 to 4 at 12 fps
- **THEN** the animation exists and shows in the sprite details

### Requirement: Export tool
The server SHALL provide `export` with the scope, format and options of the app, writing only inside the project folder and respecting the export permission.

#### Scenario: Export a folder
- **WHEN** an agent calls `export` for the folder "Personajes" as a sheet with JSON
- **THEN** the files are written inside the project and the call returns their paths

### Requirement: Undo tool
The server SHALL provide `undo_last_ai_action`, which reverts the latest action only if it is the latest entry in the history, and otherwise fails with `newer_changes`.

#### Scenario: Undo the last action
- **WHEN** an agent calls `undo_last_ai_action` right after its own drawing action
- **THEN** the drawing is reverted

### Requirement: Revisions
Every write tool SHALL return the new revision of the project and accept an optional `base_revision`, and when the project has changed since that revision the call MUST fail with `stale_revision` and change nothing.

#### Scenario: Stale revision
- **WHEN** an agent writes with `base_revision` 10 and the project is at revision 12
- **THEN** the call fails with `stale_revision` and the agent must read again

### Requirement: Exact errors
Every error SHALL have a stable code, a message that says how to fix it, and the details that apply (name, line, row, limit), and the tool descriptions MUST explain the text grid format and the palette keys.

#### Scenario: Unknown key
- **WHEN** an agent writes a pixel with a key that is not in the palette
- **THEN** the error `unknown_key` lists the keys that are valid
