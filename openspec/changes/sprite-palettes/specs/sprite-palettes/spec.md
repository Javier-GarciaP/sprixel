# Spec Delta

## Purpose

Defines own palettes for sprites and the rules that stop an agent from losing colors.

## ADDED Requirements

### Requirement: Create a palette
The tool `create_palette` SHALL create a palette with a unique name (ignoring case) and optional colors `{key, hex, name}`. A color without `key` MUST get the next free key. It MAY also add a ramp.

#### Scenario: Palette with colors
- **WHEN** an agent creates the palette `skin` with two colors
- **THEN** the palette exists with those colors and one history entry is added

#### Scenario: Name taken
- **WHEN** the name exists, in any letter case
- **THEN** the call fails with `name_exists` and nothing changes

### Requirement: Own palette
`create_sprite` with `own_palette: true` SHALL create a palette named like the sprite and use it. `set_sprite_palette` SHALL point a sprite to an existing palette and MUST fail with `keys_missing` if the sprite has pixels with a key the palette does not have.

#### Scenario: Sprite with own palette
- **WHEN** `create_sprite` is called with `own_palette: true`
- **THEN** the sprite uses a new palette with its name, and one undo removes both

#### Scenario: Change palette
- **WHEN** `set_sprite_palette` points a sprite whose pixels all exist in the new palette
- **THEN** the sprite uses the new palette and undo gives back the old one

#### Scenario: Missing keys
- **WHEN** the sprite uses a key that the new palette does not have
- **THEN** the call fails with `keys_missing` and nothing changes

### Requirement: Grid palette line
`apply_grid` SHALL NOT ignore a `palette:` line. A palette that exists and is not the palette of the sprite MUST give `palette_mismatch`; a palette that does not exist MUST give `palette_not_found`.

#### Scenario: Other palette
- **WHEN** the grid says `palette: skin` and the sprite uses `main`
- **THEN** the call fails with `palette_mismatch`, the message names `main` and says how to change it

#### Scenario: Unknown palette
- **WHEN** the grid names a palette that does not exist
- **THEN** the call fails with `palette_not_found`

### Requirement: Legend conflicts
When a legend line defines a key that the palette has with a different hex, `apply_grid` MUST NOT keep the old color silently. It SHALL use a free key (or the key that already has that hex), rewrite the pixels of that key and return `remapped` and `warnings`. The same hex gives no warning.

#### Scenario: Different color
- **WHEN** the palette has `P` as `#FF0000` and the legend says `P skin #FFCC99`
- **THEN** the pixels use a new key, the old `P` stays `#FF0000`, and `remapped` and `warnings` say it

#### Scenario: Same color
- **WHEN** the legend repeats `P` with the same hex
- **THEN** there is no remap and no warning

### Requirement: Remove unused colors
`remove_unused_colors` SHALL remove from one palette the colors that no sprite uses and return their keys. It MUST NOT touch `.`.

#### Scenario: Clean
- **WHEN** a palette has 3 colors and sprites use only one
- **THEN** the other two are removed and returned, and undo brings them back

### Requirement: Color ramps
`generate_ramp` SHALL add `steps` (3 to 9) tones of a base color to a palette, from dark to light, with an optional `hue_shift`, and return the keys. The math MUST be exact and the same every time.

#### Scenario: Three tones
- **WHEN** the base is `#FF0000` and steps is 3
- **THEN** the tones are `#590000`, `#FF0000` and `#FFA6A6`

#### Scenario: Bad steps
- **WHEN** steps is 2 or 10
- **THEN** the call fails with `bad_argument`

### Requirement: Palette panel
The palette panel SHALL show the palette of the open sprite, also when it is the sprite's own palette.

#### Scenario: Own palette
- **WHEN** a sprite uses a palette created for it
- **THEN** the panel lists the colors of that palette
