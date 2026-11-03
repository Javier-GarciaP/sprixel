# Tasks

## 1. Reading

- [x] 1.1 Add `region` and `include_palette` to `get_sprite` (partial text grid); verify the region, no legend, region out, old behavior and that the result parses as a grid

## 2. Shared pixel helper

- [x] 2.1 Add `finish_pixels` (symmetry, `only_over`, unchanged filter) and use it in `set_pixels`, `draw_shape` and `apply_grid`; verify each scenario of only over a color and symmetry, and that old calls give the same result

## 3. Region tools

- [x] 3.1 Add the pure functions (flip, mirror, move, rotate, copy) with unit tests
- [ ] 3.2 Add the tool `transform_region` with history, preview and replay; verify every scenario, one history entry and undo
- [ ] 3.3 Add the tool `copy_region`; verify copy inside a sprite, to another frame and sprite, skip transparent, palette mismatch and region out

## 4. Reference images

- [ ] 4.1 Add `import_reference` with the path checks; verify import, nearest color, `max_colors`, scale down, alpha, `..`, absolute, symlink, not PNG, big file and big image

## 5. Docs

- [ ] 5.1 List the tools in `docs/ai-and-mcp.md`, the activity log, the guide text and the skill, the text grid page and `CHANGELOG.md`; run all checks and `npx openspec validate --all --strict`
