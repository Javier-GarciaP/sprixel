# Spec Delta

## Purpose

Defines the skill that teaches Claude Code to use Sprixel.

## ADDED Requirements

### Requirement: Skill install
`sprixel mcp add claude` SHALL write the skill file `~/.claude/skills/sprixel/SKILL.md` atomically, also when Sprixel is already registered, and the file MUST come from the CLI itself, with no download.

#### Scenario: First add
- **WHEN** the person runs `sprixel mcp add claude`
- **THEN** Sprixel is registered and the skill file exists

#### Scenario: Already registered
- **WHEN** Sprixel is registered and the skill file is missing or old
- **THEN** running the command again writes the current skill file

### Requirement: Skill removal and check
`sprixel mcp remove claude` and `sprixel uninstall` SHALL delete the skill, and `sprixel doctor` SHALL report whether it is installed and current.

#### Scenario: Remove
- **WHEN** the person runs `sprixel mcp remove claude`
- **THEN** the skill folder is deleted

### Requirement: Skill content
The skill SHALL name the connection steps, the tools with the order to use them, the coordinate and frame rules, and how to react to the error codes. Design guidance comes with the spec `agent-live-design`.

#### Scenario: Agent reads it
- **WHEN** Claude Code loads the skill
- **THEN** it knows to read the project first, create or open the sprite, draw, and check with `render_preview`
