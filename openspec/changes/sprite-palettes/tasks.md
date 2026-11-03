# Tasks

## 1. Document model

- [x] 1.1 Add the operation `SetSpritePalette` with `keys_missing` and its inverse; verify apply, undo and the error
- [x] 1.2 Add `document/ramp.rs` (`ramp`); verify exact values, hue shift and bad steps

## 2. Palette tools

- [x] 2.1 Add `create_palette` and `set_sprite_palette`; verify colors, auto keys, name taken, undo and the permission
- [x] 2.2 Add `own_palette` to `create_sprite`; verify the name, one history entry and the conflict with `palette`
- [ ] 2.3 Add `remove_unused_colors`; verify removed keys, `.` untouched and undo
- [ ] 2.4 Add `generate_ramp` and the `ramp` option of `create_palette`; verify keys, reuse of an equal hex and palette_full

## 3. Safe grids

- [ ] 3.1 Check the `palette:` line in `apply_grid`; verify same palette, `palette_mismatch` and `palette_not_found`
- [ ] 3.2 Remap legend keys that conflict, with `remapped` and `warnings`; verify different hex, same hex, equal hex under another key and a full palette

## 4. UI and docs

- [ ] 4.1 Check the palette panel with an own palette and fix only what is broken; verify with a test
- [ ] 4.2 Add the tools to the activity log (Rust, `en.json`, `es.json`); verify the summaries
- [ ] 4.3 Update `docs/ai-and-mcp.md`, `docs/text-grid-format.md`, the guides and `SKILL.md`, and `CHANGELOG.md`; verify the docs tests and the skill test
