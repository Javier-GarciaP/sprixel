# Design

## Context

See proposal.md. The architecture is in `docs/architecture.md`: the Rust core
owns the document and the interface is a view. The file format must work with
Git, with humans and with AI tools that read the files.

## Goals / Non-Goals

**Goals:**
- One model and one operation path for every change.
- A file format that is readable, diffable and safe to write.

**Non-Goals:**
- A visible layers interface (the model supports layers, the UI shows one).
- Saving the history on disk. History lives in memory for the session.
- Empty-trash features (the trash is just a folder in version 1).

## Decisions

- **JSON files with row strings.** Pixels are one string per row of palette
  keys. Alternatives: PNG files (not readable or diffable), binary formats and
  SQLite (not Git friendly).
- **Operations with inverses.** Each applied operation stores how to undo it.
  Alternative: snapshots of the whole sprite (simple, but a 512x512 sprite
  would use too much memory per step).
- **Grouped actions.** A history entry can hold many operations, so a stroke
  or an AI call is one undo step.
- **Revision number.** One counter per open project, used by the MCP
  `base_revision` check.
- **Atomic writes.** Write `name.tmp` in the same folder, flush, then rename.
- **Debounced autosave** (about 500 ms) with a final flush when the app closes.
- **File watching with the `notify` crate**, ignoring events caused by the
  app's own writes.
- **Trash inside the project** (`.sprixel/trash/`) so it moves with the
  project and works across drives. Alternative: the system trash (not portable
  for undo).

## Risks / Trade-offs

- [Large sprites make large JSON files] → rows are plain strings and compress
  well in Git; only the changed rows differ.
- [The file format is hard to change later] → `formatVersion` and a migration
  step from the first version, tested with golden files.
- [Watching files can trigger loops] → record the app's own writes and ignore
  them.
