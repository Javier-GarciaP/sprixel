# Tasks

## 1. Live view

- [ ] 1.1 Emit the ordered `agent-ops` event from `write.rs` and verify the order for line, shape, grid and `set_pixels`
- [ ] 1.2 Write the pure timeline (cap 8 s, queue) and verify it with unit tests
- [ ] 1.3 Replay on the canvas with `useAgentPlayback`, ending equal to the stored frame; verify with UI tests (no fixed sleeps, 15 runs and one loaded run)
- [ ] 1.4 Draw the cursor overlay with the agent name; verify position with zoom and offset
- [ ] 1.5 Keep the frame busy until `agent_playback_done` (and a core timeout); verify held strokes and the timeout
- [ ] 1.6 Stop and reduced motion jump to the final state; verify both
- [ ] 1.7 Follow the agent only when the person is free; verify the busy person case

## 2. Design guide

- [ ] 2.1 Write the guides and the tool `get_guide`; verify topics and the unknown topic error
- [ ] 2.3 Return a preview image from `apply_grid`, `set_pixels` and `draw_shape` (argument `preview`, default true, max side 512) and verify the image, the opt-out and the size cap
- [ ] 2.2 Rewrite the server instructions and the skill with the workflow and the worked example; verify they match and that every named tool and code exists

## 3. Check with a real agent

- [ ] 3.1 Write `docs/demo.md` with the chicken test, run it with Claude Code and record what is seen
- [ ] 3.2 Update `docs/ai-and-mcp.md` and `docs/roadmap.md` and run `openspec validate --all --strict`
