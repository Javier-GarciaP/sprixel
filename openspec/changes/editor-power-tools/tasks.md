# Tasks

## 1. IPC layer

- [x] 1.1 Add `ipc/power.rs` with the region commands (flip, rotate, move, mirror, copy, paste) and verify each equals the MCP `transform_region` / `copy_region` result, is one entry, and fails on a busy frame, a rectangle rotate and a different palette
- [x] 1.2 Add the outline and shade commands (preview and apply), the review command and the tile check command, and verify they equal the MCP tool results and that preview writes nothing
- [x] 1.3 Add the palette commands (generate ramp, list unused, remove unused) and the frame commands (duration, anchors list, save and delete; durations are already in the frame info), and verify them against the MCP tools

## 2. Selection actions

- [x] 2.1 Add the editor API functions and the commands in the registry (with default shortcuts, `en.json`, `es.json`) and verify the registry tests
- [x] 2.2 Add the selection bar (flip, rotate, mirror half) and verify each button with a UI test, the rectangle rotate message and the busy frame
- [x] 2.3 Add the arrow keys move with the wrap option and verify the move, the wrap and that the selection stays

## 3. Copy and paste

- [x] 3.1 Add copy and paste of a region between frames and sprites and verify paste in another frame, the palette message and the shortcuts

## 4. Symmetry

- [x] 4.1 Add the symmetry math to the drawing controller and verify pencil, line, rectangle, ellipse and bucket mirror, with one commit per stroke
- [x] 4.2 Add the symmetry toggle, the axis on the canvas and the shortcut, and verify the toggle states and the axis

## 5. Outline and shade

- [x] 5.1 Add the canvas overlay for preview pixels and verify the renderer test
- [x] 5.2 Add the Outline dialog with live preview and verify the preview does not write, Apply writes one entry and Cancel changes nothing
- [ ] 5.3 Add the Shade dialog with live preview and verify the same, and the selection region

## 6. Review panel

- [ ] 6.1 Add the Review panel and the pixel highlight and verify the lists, the empty state and the click-to-highlight

## 7. Palette editor

- [ ] 7.1 Add Generate ramp to the palette editor and verify the new tones and the reuse of an existing tone
- [ ] 7.2 Add Remove unused colors with the list and confirmation and verify the list, cancel and confirm

## 8. Frames and anchors

- [ ] 8.1 Add the duration field to the frames strip and verify the value, the animation speed when empty and the update
- [ ] 8.2 Add anchors on the canvas (draw, create, drag, rename, delete) and verify each action

## 9. Preview tools

- [ ] 9.1 Add the preview background selector and verify the three backgrounds
- [ ] 9.2 Add the 3x3 tile preview with the seam warning and verify seamless and mismatch cases
- [ ] 9.3 Add the onion skin with 1 to 3 frames and verify the frames shown and the fading

## 10. Manual test page and end

- [ ] 10.1 Write `docs/editor-tools-test.md` and `CHANGELOG.md` lines, run all checks and `npx openspec validate --all --strict`
