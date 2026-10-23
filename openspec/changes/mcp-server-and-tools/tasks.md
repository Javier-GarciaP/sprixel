# Tasks

> **Demo cut.** The tasks under "Demo" are built for milestone M2. Task 3.5 (`export`) is built with the `export-spritesheet` spec, after this one. `apply_grid` uses the text grid module, so legend colors work in it (see the `mcp-tools` spec). See `docs/roadmap.md`.

## Demo

### 1. Server foundation

- [ ] 1.1 Decide and add the MCP implementation (after approval of the dependency) and verify a client can list an example tool
- [ ] 1.2 Implement the loopback server, the free port, `runtime.json` and the token file with user-only permissions on each system, and verify permissions in a test
- [ ] 1.3 Implement token checks, Host and Origin rejection and request limits, and verify one test per rule including a request with a valid token and an Origin

### 2. Approval and permissions

- [ ] 2.1 Implement client approval (pending, allowed, rejected) with persistence and removal, and verify blocked calls before approval
- [ ] 2.2 Implement the permission policy and `permission_denied`, and verify each of the four permissions and the change-without-reconnect scenario

### 3. Tools

- [ ] 3.1 Implement the read tools and `render_preview` and verify they do not change the revision
- [ ] 3.2 Implement `create_folder`, `create_sprite` and `add_frame` and verify the shared error codes
- [ ] 3.3 Implement `apply_grid`, `set_pixels` and `draw_shape` as atomic actions and verify the invalid grid test
- [ ] 3.4 Implement `delete_frame`, `create_tag`, `set_palette_color` and `delete_sprite` and verify validation matches the app
- [ ] 3.6 Implement revisions, `base_revision` and `stale_revision` on all write tools and verify the stale test
- [ ] 3.7 Write the tool descriptions and schemas (including the grid format and keys) and verify a client sees them

### 4. Agent activity

- [ ] 4.1 Implement the activity log, attribution and the "last edited by" data, and verify the log entries
- [ ] 4.2 Implement busy frames, user stroke queuing and Stop with `stopped_by_user`, and verify the stop and busy scenarios with a simulated agent
- [ ] 4.3 Implement `undo_last_ai_action` and the Undo availability rule, and verify the `newer_changes` case

### 5. Security and documentation

- [ ] 5.1 Run a security review of the server (token, Origin, paths, limits) and verify all findings are fixed or recorded
- [ ] 5.2 Update `docs/ai-and-mcp.md` with the final tool list and verify each tool is described

## Demo, built with the export-spritesheet spec

- [ ] 3.5 Implement `export` and verify files stay inside the project

## After the demo

- [ ] 4.4 Implement connection state and the 10 second lost detection, and verify by killing a test client
