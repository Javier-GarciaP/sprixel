# Spec Delta

## Purpose

Defines the pixel art guidance an agent receives so its designs are good.

## ADDED Requirements

### Requirement: Guide tool
The tool `get_guide` SHALL return the guide text for the topic `design`, `animation` or `palette`, MUST need no permission and MUST NOT change the project.

#### Scenario: Known topic
- **WHEN** an agent calls `get_guide` with `animation`
- **THEN** it gets the animation guidance

#### Scenario: Unknown topic
- **WHEN** the topic is not one of the three
- **THEN** the tool returns an error that lists the valid topics

### Requirement: Guide content
The `design` guide SHALL describe a small palette with color ramps, one light direction, silhouette first, flat fills, two or three tones of shading, selective outline, no stray pixels and a review loop with `render_preview` at least once. The `animation` guide SHALL describe key poses first, the same palette in every frame and a loop that closes.

#### Scenario: Names are real
- **WHEN** the guides are tested
- **THEN** every tool and error code they name exists

### Requirement: Same text everywhere
The server instructions and the Claude Code skill SHALL carry the same workflow and a worked example (a 16x16 sprite with an idle and an action frame using `apply_grid`).

#### Scenario: Skill and server agree
- **WHEN** the tests compare them
- **THEN** both name the same steps in the same order

### Requirement: The agent sees its result
Every write tool that changes pixels (`apply_grid`, `set_pixels`, `draw_shape`) SHALL return a preview image of the edited frame with its result, unless the call sets `preview` to false, and the guides and the skill MUST tell the agent to look at the image and fix what looks wrong before it says it is done.

#### Scenario: Preview after a write
- **WHEN** an agent calls `set_pixels`
- **THEN** the result carries a PNG of the edited frame, scaled so its side is at most 512 pixels

#### Scenario: Preview off
- **WHEN** the call sets `preview` to false
- **THEN** the result has no image

#### Scenario: Through the command line
- **WHEN** the same call is made with `sprixel tool`
- **THEN** the image is saved to a file and its path is printed
