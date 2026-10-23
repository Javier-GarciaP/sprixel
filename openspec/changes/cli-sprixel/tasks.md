# Tasks

> **Demo cut.** The tasks under "Demo" are built for milestone M2, and checked on Linux. See `docs/roadmap.md`.

## Demo

### 1. Shared runtime

- [ ] 1.1 Implement the shared data folder, `runtime.json` and token reading for the three systems, and verify with tests on fixtures
- [ ] 1.2 Add output helpers (`--json`, exit codes, help) and verify the documented codes

### 2. Bridge

- [ ] 2.1 Implement the stdio to HTTP proxy with the token and client name, and verify a test client lists and calls a tool
- [ ] 2.2 Implement app start, waiting, reconnect and the no-display error, and verify each scenario
- [ ] 2.3 Add the test that fails if non-protocol data reaches stdout and verify it passes

### 3. Setup commands

- [ ] 3.1 Verify the Claude Code and OpenCode configuration shapes against their documentation and record them in `docs/ai-and-mcp.md`
- [ ] 3.2 Implement `mcp add`, `mcp remove` and `mcp list` for both clients, idempotent, and verify with fixture configs
- [ ] 3.3 Implement `doctor` with all checks and fix hints, and verify a passing and a failing run

### 4. Other commands

- [ ] 4.1 Implement `start`, `status`, `version`, `tools` and `tool` and verify against a running test app

### 5. Documentation

- [ ] 5.1 Update `docs/usage.md` and `docs/ai-and-mcp.md` with the final commands and verify each command shown runs

## After the demo

- [ ] 4.2 Run the whole command set on Linux, Windows and macOS and verify the same results
