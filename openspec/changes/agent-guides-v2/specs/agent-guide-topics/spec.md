# Spec Delta

## Purpose

Defines the extra guide topics an agent can read, so it knows how to draw big
sprites, faces, animations and game assets, and which tool to use for each job.

## ADDED Requirements

### Requirement: New guide topics
The tool `get_guide` SHALL accept the topics `large-sprites`, `faces-and-animals`, `walk-and-attack-cycles`, `game-ready` and `tools`, besides `design`, `animation` and `palette`. It MUST need no permission and MUST NOT change the project.

#### Scenario: Known topic
- **WHEN** an agent calls `get_guide` with `game-ready`
- **THEN** it gets the guide text for that topic

#### Scenario: Unknown topic
- **WHEN** the topic is not one of the eight
- **THEN** the call fails with `unknown_topic` and the error lists the eight topics

### Requirement: Guide content
The `large-sprites` guide SHALL describe working region by region, using a reference image, symmetry and the loop of `analyze_sprite`. The `faces-and-animals` guide SHALL describe eyes, symmetry and the silhouette of an animal. The `walk-and-attack-cycles` guide SHALL describe key poses, frame counts, frame durations, ground contact and closing the loop. The `game-ready` guide SHALL describe anchors, durations, tiles and export.

#### Scenario: Guides have their rules
- **WHEN** the guides are tested
- **THEN** each one contains the ideas listed above

### Requirement: Tool map
The `tools` guide SHALL tell which tool to use for each job and MUST name every tool of the server, including the palette, preview, edit, analysis and game data tools.

#### Scenario: No tool is forgotten
- **WHEN** a test compares the tools of the server with the `tools` guide
- **THEN** every tool name appears in the guide

### Requirement: Worked examples that run
Each new guide SHALL have a worked example with exact calls and a grid, and a test MUST run those calls through the real tools and check the result.

#### Scenario: Example runs
- **WHEN** the test runs the calls of an example in order
- **THEN** every call succeeds and the sprite has the shape the guide describes (sizes, frames, tags, anchors, durations, `analyze_sprite` results)

#### Scenario: Names are real
- **WHEN** the guides are tested
- **THEN** every tool, argument and error code they name exists

### Requirement: Same list everywhere
The server instructions, the Claude Code skill and `docs/ai-and-mcp.md` SHALL carry the same sentence that lists the guide topics.

#### Scenario: Instructions, skill and docs agree
- **WHEN** the tests compare them
- **THEN** all three contain the sentence, and it names all eight topics
