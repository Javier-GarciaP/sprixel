# Tasks

> **Demo cut.** Every task is in the demo. The palette grows by itself with the colors that are used (the core part is built in `pixel-canvas-editor`, task 3.6); this spec adds the panel, the editor, the presets and the import. See `docs/roadmap.md`.

## 1. Palette model and operations

- [ ] 1.1 Implement color validation (key rules, limit 93, uniqueness) and verify unit tests for each rule
- [ ] 1.2 Implement color edit, add, remove-with-replacement and key rewrite as grouped operations and verify the key change keeps every sprite looking the same
- [ ] 1.3 Add the presets "Pocket RPG 16" and "Farm 32" as data, with generic names and no trademarks, and verify each has 16 and 32 valid colors with unique keys

## 2. Panel and editor

- [ ] 2.1 Build the palette panel with main and secondary selection, transparent entry and "Show keys", and verify against the design
- [ ] 2.2 Build the palette editor dialog with the list, fields, RGB sliders, key warning, Cancel and Done, and verify the cancel and apply scenarios
- [ ] 2.3 Block key changes while an agent writes and verify with a simulated agent

## 3. Creating and importing

- [ ] 3.1 Implement create from presets and the "+" in the folders panel and verify the new palette appears
- [ ] 3.2 Implement `.gpl` and `.hex` import with generated keys and verify with fixture files

## 4. Documentation

- [ ] 4.1 Update `docs/usage.md` and `docs/text-grid-format.md` and verify they match the key rules
