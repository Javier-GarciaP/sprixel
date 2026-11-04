# Tasks

## 1. Format versions

- [x] 1.1 Treat a file with no `formatVersion` as version 1 and open sprite files with no `frameDurations`; verify with fixture projects of old versions
- [x] 1.2 Verify a newer project is refused and nothing is written, and a multi-step migration chain with a backup; write `docs/project-format.md`

## 2. Export JSON

- [x] 2.1 Add `index` and `frame_number` to the frames and the `feet` pivot; verify scale, fallback and the docs sample
- [x] 2.2 Add `json_format` `aseprite` to the core and the `export` tool; verify the shape, tags and `bad_argument`
- [x] 2.3 Add the JSON format selector to the Export screen; verify it with UI tests
- [x] 2.4 Update `docs/export-format.md` with both formats; verify the docs tests

## 3. Change the canvas

- [x] 3.1 Add the pure resize functions (pixels and anchors) for the 9 anchors; verify crop, extend and anchor movement
- [x] 3.2 Add `ResizeSprite` and `RestoreCanvas` with undo; verify limits, tags, durations and exact undo
- [x] 3.3 Add the tool `resize_sprite`; verify permission, errors, warnings, one history entry and the activity log
- [x] 3.4 Add the dialog "Change canvas" to the sprite details panel; verify it with UI tests
- [ ] 3.5 Update the rule about the sprite size in `AGENTS.md` and the docs

## 4. Game engines

- [ ] 4.1 Write `docs/game-engines.md` and a test that checks the options and files it names

## 5. Finish

- [ ] 5.1 Update `docs/ai-and-mcp.md`, the changelog; run format, lint, tests, builds and `openspec validate --all --strict`
