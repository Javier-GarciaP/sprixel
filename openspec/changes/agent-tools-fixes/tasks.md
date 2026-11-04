# Tasks

## 1. Dry run

- [x] 1.1 Add `dry_run` to `write()` (copy of the project, no history, revision, replay or event) and to `apply_grid`, `auto_outline`, `auto_shade`, `transform_region`, `copy_region`; verify no change, counts, preview, `base_revision` and a legend color not added

## 2. Warnings

- [x] 2.1 Add `analysis::thin_regions` and the `inner` warning of `auto_outline`; verify a brim 2 rows high, a thick shape and no warning without `inner`
- [x] 2.2 Add `analysis::isolated_changes` and the speckle warning of `auto_shade`; verify the 2x2 block and a clean shade

## 3. Analysis

- [x] 3.1 Add `checked`, `not_checked` and outline detection to `analyze_sprite`; verify detected, tie, no shape, given key and one frame
- [x] 3.2 Add `analysis::lowest_row` and the support rule of `ground_kept`; verify the lifted foot, the `feet` anchor, the frame anchor and the old fields

## 4. Erase safety

- [x] 4.1 Make region `apply_grid` skip `.` by default and add `erased` and its warning to every write; verify region, whole frame, explicit false and `transform_region`

## 5. Frame durations

- [x] 5.1 Show `null` in `frame_durations` and accept `milliseconds: null`; verify default, own value, reset and an explicit 100 note

## 6. Guides

- [x] 6.1 Update the `animation` and `walk-and-attack-cycles` guides and the skill; verify the required words
- [ ] 6.2 Add one example call per tool to the `tools` guide; verify every example parses and names a real tool
- [ ] 6.3 Update the instructions and the skill, `docs/ai-and-mcp.md`, `docs/text-grid-format.md` and `CHANGELOG.md`; run all checks and `npx openspec validate --all --strict`
