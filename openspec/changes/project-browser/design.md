# Design

## Context

See proposal.md and the screens 03 to 16 in `docs/design/`. Documents and
operations come from `project-format-and-storage`. Sprite thumbnails are small
renderings of the first frame.

## Goals / Non-Goals

**Goals:**
- Everyday actions are fast: deleting never blocks, undo always works.
- Every action can be started from a visible place.

**Non-Goals:**
- Importing images, tags on folders, or sorting options beyond name.
- Emptying the trash from the app.

## Decisions

- **Trash plus an Undo message instead of a confirmation for one sprite.**
  Safer and faster. Groups and folders still ask because more is at stake.
- **Browser state in the feature, data from the core.** The feature asks the
  core for the project tree and subscribes to document events.
- **Thumbnails** are rendered with the shared canvas renderer and cached by
  sprite and revision.
- **One action list** feeds the "..." button, the context menu and the
  keyboard, so they cannot disagree.
- **Drag and drop** for moving is optional in the first version of the task
  list; the menu is the required way.

## Risks / Trade-offs

- [Many sprites make the grid slow] → virtualize the list and load thumbnails
  lazily.
- [Undo after the message disappears] → Ctrl+Z still restores it as long as
  the app session lasts.
