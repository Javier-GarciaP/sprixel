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

## Decisions for the demo cut

These decisions were taken when the demo was planned (see `docs/roadmap.md`).

- **Events:** after every operation on the open project the core emits the
  Tauri event `project://changed` (new revision and touched targets). The UI
  re-queries; it never polls for the document.
- **Queries:** `project_tree` (nested folders and sprite summaries with the last
  editor taken from the history of the session), `sprite_frame` (rows of keys
  and palette colors, for thumbnails and previews) and the list of palettes.
- **Undo and redo** are core commands, so Ctrl+Z restores a deleted sprite.
- **New operations** with inverses: rename sprite, rename folder and duplicate
  sprite ("goblin copy", "goblin copy 2"). The storage layer renames the file.
- **Creating a sprite is one grouped action.** With "No palette" and no palette
  `main` in the project, the core creates an empty `main` and then the sprite.
- **Names** are checked by the core (`check_name` returns a stable error code),
  so the dialog shows the same rules as the core.
- **Rendering:** a pure function turns rows and colors into an RGBA buffer (the
  tests use it, because jsdom has no canvas); a thin canvas function draws it
  with integer scaling; a cache keyed by sprite and revision keeps thumbnails
  current. They live in `src/shared/lib/render/`. Calls to the core live in
  `src/shared/lib/core/`.
- **Open in editor:** the feature calls an `onOpenSprite` prop; the app keeps
  the active sprite and switches to the Editor tab.
- **In the demo** the sprite menu offers Open, Rename, Duplicate and Delete. The
  shared action list, the context menu, search, move, folder delete and
  multi-select come after the demo.

## Risks / Trade-offs

- [Many sprites make the grid slow] → virtualize the list and load thumbnails
  lazily.
- [Undo after the message disappears] → Ctrl+Z still restores it as long as
  the app session lasts.
