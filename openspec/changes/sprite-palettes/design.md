# Design

## Pointing a sprite to a palette
The model has no operation to change the palette of a sprite. We add
`SetSpritePalette { sprite, palette }`. It fails with `keys_missing` if the
sprite has a pixel key that the new palette does not have, so no pixel is
ever left without a color. Its inverse is the same operation with the old
palette. No pixel is rewritten, so it is small and safe.

## Tools (new file `mcp/tools/palettes.rs`)
- `create_palette` (`Need::Edit`): `name`, optional `colors` `[{key?, hex, name?}]`
  (no `key` = next free key), optional `ramp {base, steps, hue_shift?, name?}`.
  Uses the same color rules as the app (`check_colors`). Name unique ignoring case.
- `set_palette_color` stays as it is. `set_sprite_palette` (`Need::Edit`): `name`, `folder`, `palette`.
- `create_sprite` with `own_palette: true` creates a palette named like the
  sprite and uses it (one history entry). It cannot be used with `palette`.
- `remove_unused_colors` (`Need::Edit`): removes the colors of one palette that
  no sprite uses, one `SetPalette` operation. `.` is never a color.
- `generate_ramp` (`Need::Edit`): adds the tones of `base` to a palette.

## apply_grid palette line
The `palette:` line is checked before anything else, in the tool. Same name as
the sprite palette (ignoring case): fine. Another existing palette:
`palette_mismatch` (names the sprite palette, and says to remove the line, to
call `set_sprite_palette`, or to create the sprite with `own_palette`). A name
that does not exist: `palette_not_found`.

## Legend conflicts
In `Project::grid_plan` (the new core of `grid_operations`): for a legend line
with a `#hex` whose key is in the palette with a DIFFERENT hex:
1. if the palette already has that hex under another key, use that key;
2. else take the next free key (not in the palette, not in the legend), add the
   color with the legend name (a number is added if the name is taken);
3. rewrite every cell of that key in the grid to the new key.
The answer has `remapped` `{"P": "Z"}` and `warnings` (a list of sentences).
Same hex = no warning. `grid_operations` keeps its signature for the app.

## Ramps (pure math, `document/ramp.rs`)
`ramp(base, steps, hue_shift) -> Vec<String>` from dark to light, `steps` 3 to 9,
the base is one of them. In HSL: lighter tones = `(steps-1)/2` (floor), darker
tones = the rest. Tone `k` of `m` on the dark side has lightness
`L * (1 - 0.65 * k/m)`; on the light side `L + (1 - L) * 0.65 * k/m`. Hue moves
`-hue_shift * k/m` degrees on the dark side and `+hue_shift * k/m` on the light
side (default 0; -60 to 60). Saturation stays. Rounding: nearest integer per channel.
Names are `<name>-dark2`, `-dark1`, `-base`, `-light1`...; a tone whose hex is
already in the palette reuses that key and adds nothing.

## UI
The palette panel already reads the palette of the open sprite. We check it with
a sprite that has its own palette and fix only what is broken.
