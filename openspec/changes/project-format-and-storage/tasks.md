# Tasks

## 1. Document model

- [x] 1.1 Implement project, folder, sprite, palette, frame, layer and tag types with their validation rules and verify unit tests cover each rule in the spec
- [x] 1.2 Implement name rules (unique ignoring case, valid on all systems) and verify tests with reserved Windows names and accents

## 2. Operations and history

- [x] 2.1 Implement operations with author, inverse and revision, and verify a failed operation leaves the project unchanged
- [x] 2.2 Implement grouped actions applied atomically and verify the 41st-pixel failure test
- [x] 2.3 Implement undo, redo and `undo_last_ai_action` with the `newer_changes` rule and verify the scenarios in the spec

## 3. File format

- [x] 3.1 Implement reading and writing of `sprixel.json`, palette files and sprite files and verify golden file tests, including a one-pixel diff test
- [x] 3.2 Implement `formatVersion` checks, migration with backup and the newer-version refusal and verify tests for each path

## 4. Saving

- [x] 4.1 Implement atomic writes and verify a test that kills the writer mid-save leaves a valid file
- [x] 4.2 Implement debounced autosave and the saved, saving and error statuses and verify the one-second scenario
- [ ] 4.3 Implement save error handling with retry and verify data stays in memory when the disk is full (simulated)

## 5. Trash and outside changes

- [ ] 5.1 Implement the trash move and restore on undo and verify a delete-and-undo test
- [ ] 5.2 Implement file watching with self-write filtering and the reload or keep notice and verify a test that edits a file externally
- [ ] 5.3 Implement damaged file handling and verify a project with one broken sprite still opens

## 6. Performance and documentation

- [ ] 6.1 Add the 500-sprite project benchmark and verify it lists in 2 seconds or less
- [ ] 6.2 Update `docs/architecture.md` with the final file format and verify the examples match the real files
