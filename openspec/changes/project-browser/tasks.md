# Tasks

> **Demo cut.** The tasks under "Demo" are built for milestone M2. In the demo the sprite menu (the "..." button) offers Open, Rename, Duplicate and Delete; the shared action list, the context menu and the keyboard entries (5.2) come after the demo. The palette in the New sprite dialog is optional (see the `sprite-management` spec). See `docs/roadmap.md`.

## Demo

### 1. Reading the project

- [x] 1.1 Add the project tree and sprite list queries and events and verify a test that opens a sample project and lists it
- [x] 1.2 Add the thumbnail renderer with caching and verify the thumbnail updates when a sprite changes

### 2. Project tab

- [x] 2.1 Build the folders tree with counts, selection and the palettes list, and verify against screens 06 and 03
- [x] 2.2 Build the sprite cards, the "New sprite" card and the details panel (animations, last edited by) and verify the scenarios in the spec
- [x] 2.4 Open a sprite in the editor from double-click and the button, and verify the tab switch

### 3. Creating

- [x] 3.1 Build the New sprite dialog with presets, custom size, the 128 warning and name validation, and verify the three size scenarios
- [x] 3.2 Build the New folder dialog with the duplicate name error and verify the red field and disabled button

### 4. Organizing

- [x] 4.1 Implement rename (menu, F2, double-click) and duplicate with unique names, and verify with tests
- [x] 4.3 Implement delete to the trash with the 8 second Undo message and Ctrl+Z, and verify a delete and undo test

### 6. Documentation

- [ ] 6.1 Update `docs/usage.md` for the Project tab and verify it matches the app

## After the demo

- [x] 2.3 Build search (case and accent insensitive) and the empty states, and verify with tests
- [x] 4.2 Implement move with the "Move to folder" submenu and the destination conflict message, and verify with tests
- [x] 4.4 Implement the folder and group delete confirmation with focus on Cancel, and verify the dialog text lists the contents
- [ ] 5.1 Implement multiple selection with the action bar and combined details and verify the "3 selected" scenario
- [ ] 5.2 Add the shared action list to the "..." button, right-click menu and keyboard, and verify each entry point triggers the same action
