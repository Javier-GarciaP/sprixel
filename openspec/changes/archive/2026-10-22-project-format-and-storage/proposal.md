# Proposal

## Why

Every feature reads or changes the same data: projects, folders, sprites,
frames, palettes and animations. They need one clear model, one way to change
it (so that undo, redo and the AI all behave the same) and one way to keep it
safe on disk.

## What Changes

- Define the document model: project, folders, sprites, palettes, frames,
  layers and animation tags, with their rules.
- Define operations: every change is an operation with an author, applied by
  the Rust core, with a revision number and undo and redo.
- Define the project folder format and the sprite and palette files, with a
  `formatVersion`.
- Add automatic and atomic saving, a trash for deleted items, detection of
  changes made outside the app, migrations and recovery from damaged files.

## Capabilities

### New Capabilities
- `document-model`: the data model, its rules, operations, revisions and history.
- `project-storage`: how a project is saved, loaded, migrated and protected on disk.

### Modified Capabilities

## Impact

- New Rust modules `document`, `history` and `storage` in `src-tauri/`.
- Adds a file watching dependency (for changes outside the app).
- Defines the file format that `export`, `text-grid` and the MCP tools rely on.
- Sprite files must stay stable: changes later need a migration.
