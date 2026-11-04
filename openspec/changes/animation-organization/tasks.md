# Tasks

## 1. Core and agent tools (agent A)

- [x] 1.1 Add the no-overlap rule and `animation_overlap` to `create_tag` and the editor operations; verify with old overlapping files kept as they are
- [x] 1.2 Add `create_animation` (copy the base, tag, one history entry) and verify frames, tag, undo and `name_exists`
- [ ] 1.3 Add `duplicate_animation`, `rename_animation`, `delete_animation` (`with_frames`) and `move_frame`; verify tags follow frames and undo
- [ ] 1.4 Add `frames` (list or range) to `transform_region` and `copy_region`; verify one atomic history entry
- [ ] 1.5 Add the guide topic `animations-from-base`, the rules (base first, one tag per animation, no pixels outside the app without asking), an example call per new tool, and update the skill, the instructions and `docs/ai-and-mcp.md`; verify skill equals guide

## 2. Interface (agent B)

- [ ] 2.1 Group the frames strip by animation (collapsible header with name, fps, loop, play; Base first, No animation last); verify with a sprite of two animations
- [ ] 2.2 Add "New animation..." (copy the base) and the header menu (rename, duplicate, delete with or without frames), each one undo step; verify
- [ ] 2.3 Play a single group, collapse long lists above 40 frames, and update `docs/editor-tools-test.md`; verify

## 3. Parts (phase 2, agent A after 1.5)

- [ ] 3.1 Add the optional `parts` list to the model, the operations, the file (optional field, `format_version` note) and verify old projects open
- [ ] 3.2 Add `set_part`, `remove_part` and `part` in the region tools, update the guide and skill; verify
