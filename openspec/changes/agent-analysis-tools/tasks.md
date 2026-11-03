# Tasks

## 1. Pure analysis module

- [x] 1.1 Add `analysis` with the shared helpers (neighbors, exterior) and `auto_outline` logic, and verify with exact unit tests (4 and 8 neighbors, outside only, inner, clipped)
- [x] 1.2 Add `auto_shade` logic and verify with exact unit tests (the 6x6 square, longer ramp, light directions, region, other keys kept)
- [ ] 1.3 Add `analyze_sprite` logic (loose pixels, gaps, colors, frame comparison) and verify with exact unit tests

## 2. Tools

- [ ] 2.1 Add the tool `auto_outline` through `write()` and verify it with MCP tests (result, one history entry, errors, permission, stale revision)
- [ ] 2.2 Add the tool `auto_shade` through `write()` and verify it with MCP tests
- [ ] 2.3 Add the read-only tool `analyze_sprite` and verify it with MCP tests (JSON shape, no change of revision or history, not in the activity log)
- [ ] 2.4 Add the activity texts (Rust summary, `en.json`, `es.json`, activity list) and verify with a Rust and a UI test

## 3. Docs and guides

- [ ] 3.1 Update `docs/ai-and-mcp.md`, the design guide, `SKILL.md` and `CHANGELOG.md`, and verify the docs and guide tests pass
- [ ] 3.2 Run format, lint, all tests, builds and `openspec validate --all --strict`
