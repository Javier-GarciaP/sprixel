# Design

## Context

See proposal.md. Pixels are stored as keys, so a palette change is either a
value change (no file rewrite) or a key change (rewrite of sprites).

## Goals / Non-Goals

**Goals:**
- Safe editing: one undo reverts a whole palette edit.
- Keys that are short, unique and easy for an AI to use.

**Non-Goals:**
- Color pickers beyond hex and RGB sliders, palette generation or gradients.
- Several palettes in one sprite.

## Decisions

- **A palette edit session works on a copy** and is applied as one grouped
  operation on Done.
- **Key rewrite is done by the core** for all sprites of the palette in the
  same grouped operation, so it is atomic and undoable.
- **Generated keys** use letters and digits in order, skipping used ones.
- **Import parsers in Rust** (`.gpl`, `.hex`) with small fixtures.
- **Limit of 93 keys** comes from printable ASCII without space and dot.

## Risks / Trade-offs

- [A key change breaks AI prompts or text written earlier] → warning in the
  editor and a clear error from the text grid validator.
- [Large projects make key rewrites slow] → rewrite only sprites that use the
  palette and report progress when it takes long.
