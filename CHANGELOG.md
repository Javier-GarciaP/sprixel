# Changelog

All notable changes to Sprixel are written here.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)
and the project follows [Semantic Versioning](https://semver.org/).

## [Unreleased]

### Added

- Project planning: concept, design (36 screens in light and dark), version 1
  specs and documentation.
- Pixel canvas editor: exact pixels over a gray checkerboard, zoom from 100% to
  3200%, Fit, panning, pixel grid from 600% and the pointer coordinates.
- Drawing tools: Pencil, Eraser, Bucket, Eyedropper, Line, Rectangle, Ellipse,
  Select and Move, with a main and a secondary color, Shift for 45-degree
  lines, squares and circles, and undo and redo (one step for each stroke).
- A palette that grows by itself with the colors you use, up to 93 colors.
- Onion skin: the previous frame in blue under the drawing, with an opacity
  from 10% to 80%.
- A marker, a busy frame and a Stop button while an AI agent is drawing.
- Project tab: a folders tree with item counts, sprite cards with thumbnails,
  a details panel (size, frames, palette, animations, who edited it last) and
  the palettes of the project.
- New sprite dialog (presets, any size up to 512x512, a warning from 128, a
  palette or `main`) and New folder dialog, with name validation.
- Rename, duplicate, move (menu or drag and drop) and delete for sprites and
  folders. One sprite goes to the trash with an 8 second Undo message; folders
  and groups ask first. `Ctrl+Z` and `Ctrl+Shift+Z` undo and redo.
- Search by name that ignores case and accents, multiple selection with an
  action bar, a right-click menu and the keys F2, Ctrl+D, Delete and Enter.
- The core sends the `project://changed` event after every change and answers
  `project_tree` and `sprite_frame` queries.
