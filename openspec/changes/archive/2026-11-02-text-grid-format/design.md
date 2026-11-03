# Design

## Context

See proposal.md and `docs/text-grid-format.md`. The grid module lives in the
Rust core so that the app and the MCP tools share one implementation. The Text
view is a UI over that module.

## Goals / Non-Goals

**Goals:**
- One parser and one validator, tested with many cases.
- Errors that tell exactly what to fix.

**Non-Goals:**
- Syntax highlighting beyond line errors, or a full code editor.
- Other text formats (JSON or ASCII art variants).

## Decisions

- **Hand-written parser** in Rust with line and column tracking. Alternative:
  a parsing library (more weight for a very small grammar).
- **Collect all errors**, not only the first, so an AI can fix them in one
  retry.
- **Validation runs in the core** through a command; the UI debounces calls
  while typing.
- **Plain `<textarea>` first** with line numbers and highlighting of error
  lines. A richer editor can come later if needed.

## Risks / Trade-offs

- [Typing in large grids is slow to validate] → debounce, and validate only the
  edited frame.
- [Users edit the legend by mistake] → the legend is informational; the
  palette is the truth, and a mismatch is shown as a warning, not an error.
