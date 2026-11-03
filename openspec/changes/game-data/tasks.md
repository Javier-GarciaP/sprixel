# Tasks

## 1. Frame durations

- [x] 1.1 Add `set_frame_duration` and the durations in `get_sprite`; verify range, undo, replay of the answer and that the animation preview and export use them

## 2. Anchors

- [x] 2.1 Add `Anchor` to the model, `SetAnchor` and `RemoveAnchor` with undo, and keep them right on frame delete, insert, move and restore; verify each
- [x] 2.2 Save and read `anchors` in the sprite file; verify old files open and files without anchors stay equal
- [x] 2.3 Add `set_anchor` and `remove_anchor` and anchors in `get_sprite`; verify errors, undo and the activity log

## 3. Export

- [x] 3.1 Add `frames[].anchors` to the sheet JSON; verify override, scale and empty list
- [x] 3.2 Document the Godot and Unity mapping with an example; verify the sample equals the real JSON

## 4. Tiles

- [ ] 4.1 Add `check_tile`; verify seamless, seam, transparent pairs and the mismatch cap
- [ ] 4.2 Add `render_tiled`; verify the size and the cap

## 5. Docs and finish

- [ ] 5.1 Update `docs/ai-and-mcp.md`, the guide and the skill, and the changelog; verify the docs tests
- [ ] 5.2 Run format, lint, tests, builds and `openspec validate --all --strict`
