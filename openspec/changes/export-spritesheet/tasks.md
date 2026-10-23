# Tasks

> **Demo cut.** The tasks under "Demo" are built for milestone M2, after the AI path. The MCP `export` tool (task 3.5 of `mcp-server-and-tools`) is added in this phase. See `docs/roadmap.md`.

## Demo

### 1. Rendering

- [ ] 1.1 Implement frame rendering with nearest-neighbor scale and verify against fixture images
- [ ] 1.2 Implement the row packer with spacing and verify the 71x17 example and determinism

### 2. Formats

- [ ] 2.1 Implement the sheet PNG and the JSON writer (frames, durations, animations, pivot, meta) and verify against a golden JSON
- [ ] 2.2 Implement single PNG files with the naming pattern and verify the names

### 3. Safety

- [ ] 3.1 Implement output folder checks (inside the project only), temporary folder writing and replace on success, and verify the outside path and failed write tests

### 4. Export tab

- [ ] 4.1 Build the three steps with the scope counts, formats and options, and verify against screen 24
- [ ] 4.2 Build the success, error and empty states and verify against screens 25, 26 and 27

### 5. Documentation

- [ ] 5.1 Document the JSON format in `docs/` and verify a sample export matches it

## After the demo

- [ ] 2.3 Implement GIF export per animation with timing and transparency and verify frame delays
- [ ] 2.4 Implement trimming and pivot options and verify with tests
- [ ] 4.3 Add progress for large exports and verify the UI stays responsive
