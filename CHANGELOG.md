# Changelog

All notable changes to Sprixel are written here.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)
and the project follows [Semantic Versioning](https://semver.org/).

## [Unreleased]

### Added

- The install scripts are friendlier. On a terminal they show a summary, ask for
  confirmation, ask if you want the menu entry and if you want to connect Claude
  Code or OpenCode now (only the tools they find), and show a progress bar for
  every download. With no terminal, or with `--yes`, they ask nothing and behave
  as before. Options for scripts: `--yes`, `--no-menu`, `--connect`; PowerShell
  uses `SPRIXEL_YES`, `SPRIXEL_MENU` and `SPRIXEL_CONNECT`.
- Sprixel appears in the applications menu with its icon on Linux (a `.desktop`
  entry and an icon) and on Windows (a Start Menu shortcut). The app writes its
  own entry, so it works with the install script and with a manual download.
  `sprixel-app --register-menu` (or `Sprixel.exe --register-menu`) writes it
  without opening a window; the install scripts and `sprixel update` use it.
  `sprixel uninstall` removes it. To have no entry, create `~/.sprixel/no-menu-entry`.
  Not checked on a real Windows or macOS machine yet, only by automatic tests.

## [0.1.0] - 2026-11-04

First public version of Sprixel. It is an early version (0.x), so expect rough
edges. Known limits:

- Only tested by hand on Linux. Windows and macOS are checked by the automatic
  tests only.
- GIF export is not ready yet (the button is disabled). Trimming, pivot and a
  progress bar for big exports come later too.
- The app and the installers are not signed, so your system may show a warning
  the first time.
- If an AI tool loses its connection, the app does not show it after 10 seconds
  yet.

### Fixed

- The frames column is tidier: the groups are the only selector of the
  animation (the dropdown is gone), only the chosen group is open and shows its
  frames in two columns with their number and duration, each header keeps one
  height with the frame count, play and menu, and the actions sit at the bottom
  with tooltips. The preview has a list ("All frames" or one animation) and
  plays only that range with its speed and mode.
- The top bar of the editor no longer jumps when you select: every control stays
  in place (disabled when it does not apply), uses an icon with a tooltip that
  shows its name and shortcut, and the least used ones go to a "More" menu in
  narrow windows.

### Added

- The frames strip is grouped by animation: a collapsible header for each one
  (name, speed, how it ends, play and a menu to rename, duplicate or delete it
  with or without its frames), a Base group first and a No animation group
  last. "New animation…" copies the base frame. Groups start closed above 40
  frames. Every action is one step of Undo.
- Animations organized like in other tools: a base frame and one named animation
  per action, with no shared frames. New AI tools `create_animation` (copies the
  base frame, tags the copies, one Undo step), `duplicate_animation`,
  `rename_animation`, `delete_animation` (with or without its frames) and
  `move_frame`; `transform_region` and `copy_region` accept `frames` (a list or
  a range like `"8-13"`) as one step. New guide `animations-from-base` and a
  rule for agents: never make pixels outside the app (scripts, image libraries)
  without the agreement of the person. New editor commands `animation_create`,
  `animation_duplicate`, `animation_rename` and `animation_delete`. A new
  animation that shares a frame with another one is refused (`animation_overlap`).
- Named parts for the AI: `set_part` and `remove_part` name a rectangle of a sprite
  (head, tail), and `transform_region` and `copy_region` accept it as `part`.
  Stored in an optional `parts` list of the sprite file; old projects open as
  before.
- Power tools for people in the editor, the same ones the AI has: a selection
  bar (flip, rotate, mirror a half, move with the arrow keys and wrap), copy
  and paste between frames and sprites, a symmetry mode with the axis on the
  canvas, Outline and Shade panels with a live preview, a Review panel that
  marks loose pixels and outline gaps, Generate ramp and Remove unused colors
  in the palette editor, a duration for each frame, anchors with the Anchor
  tool, a preview background (checker, light, dark), a 3x3 tile preview with a
  seam warning and an onion skin of up to 3 frames. Every action is one step of
  Undo. `docs/editor-tools-test.md` is the manual test page.
- Format and compatibility: files with no `formatVersion` or no
  `frameDurations` (the first ones) open as version 1, a project from a newer
  Sprixel is refused without writing anything, and `docs/project-format.md`
  lists the format versions and the rules for changing the format. Old-project
  fixtures test it.
- Export JSON for engines: every frame has `index` and `frame_number`, `pivot`
  uses the `feet` anchor, and the new option `json_format` `aseprite` (tool
  `export` and a selector in the Export screen) writes the Aseprite sprite
  sheet JSON.
- Change the canvas: the operation `ResizeSprite`, the tool `resize_sprite` and
  the dialog **Change canvas…** in the details panel crop or extend a sprite
  with an anchor, move its anchors and undo exactly. The sprite size is now
  changed only by this explicit action.
- `docs/game-engines.md`: from a drawing to Godot 4 and Unity, step by step.
- Safer AI tools: `dry_run` on `auto_outline`, `auto_shade`, `transform_region`,
  `copy_region` and `apply_grid` shows the result without writing; warnings for
  thin parts under `inner` outlines and for lone shade pixels; `analyze_sprite`
  returns `checked` and `not_checked` and detects the outline key; `ground_kept`
  follows the support foot (the `feet` anchor or the lowest row); every write
  reports `erased`; `frame_durations` uses `null` for the animation speed.
  Region grids of `apply_grid` no longer erase with `.` unless
  `skip_transparent` is false. The guides teach the walk rules and every tool
  has an example call.
- Richer guides for the AI: `get_guide` has five new topics (`large-sprites`,
  `faces-and-animals`, `walk-and-attack-cycles`, `game-ready` and `tools`, a
  map of which tool to use for what). Each one has a worked example, and the
  tests run every example through the real tools. The server instructions, the
  Claude Code skill and `docs/ai-and-mcp.md` list the topics.
- Game data for the AI: `set_frame_duration`, anchor points (`set_anchor`,
  `remove_anchor`, saved in the sprite file as an optional `anchors` list),
  `check_tile` and `render_tiled` for seamless tiles. The sheet JSON lists the
  anchors of every frame, and `docs/export-format.md` explains how to use it
  in Godot and Unity.
- Finishing tools for the AI: `auto_outline` draws a one pixel outline (4 or 8
  neighbors, outside only or inner), `auto_shade` shades a region with a ramp
  of tones for a light direction, and `analyze_sprite` reports loose pixels,
  gaps in the outline, unused colors and how each frame differs from the next
  (changed pixels, box, ground kept).
- Edit tools for agents: `transform_region` (flip, mirror, move with `wrap`,
  rotate) and `copy_region` (inside a sprite or to another frame or sprite),
  `only_over` and `symmetry` on `set_pixels`, `draw_shape` and `apply_grid`,
  and `region` and `include_palette` on `get_sprite` to read big sprites cheaply.
- `import_reference` draws a PNG from the `references/` folder of the project
  into a frame, reduced to the sprite palette (optionally adding colors).
- Palettes for the AI: `create_palette`, `set_sprite_palette`, `remove_unused_colors`
  and `generate_ramp` (color ramps with an optional hue shift), and
  `own_palette` in `create_sprite`. A sprite can now have a palette of its own.
- `apply_grid` never ignores a `palette:` line (`palette_mismatch`,
  `palette_not_found`). A legend color that conflicts with a palette key moves
  to a free key and is reported in `remapped` and `warnings`.
- Previews for the AI: `render_preview` and the image of every write now sit on a
  gray checkerboard by default (so white pixels are visible); the `background`
  option can be `checker`, `light`, `dark` or `transparent`. New tools
  `render_frames` (all frames in one grid, with optional numbers) and
  `render_onion` (frames over each other, earlier ones tinted). `sprixel tool`
  saves their images as files too.
- Watch the AI draw: the editor replays each agent write (at most 8 seconds)
  with a cursor that carries the agent name, keeps the frame busy until the
  replay ends, and follows the agent to its sprite and frame when you are free.
  Stop and reduced motion show the final state at once.
- `get_guide` tool with pixel art guides for the AI (`design`, `animation` and
  `palette`). The server instructions and the Claude Code skill now share one
  workflow and a worked 16x16 example.
- `apply_grid`, `set_pixels` and `draw_shape` return an image of the edited
  frame (at most 512 pixels on a side) so the AI can look at its work; send
  `preview` false to skip it.
- `docs/demo.md`: the chicken test to check this with a real agent.
- Agents tab: Claude Code and OpenCode cards, activity list with Stop and
  Undo, permissions panel, Connect dialog, approval prompt and top bar state.
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
- Frames: a column with thumbnails, add (empty or a copy) after the current
  frame, duplicate, delete and drag to reorder, with animation ranges that
  follow the frames.
- Animations: create, edit and delete them in the sprite details (name, frames,
  1 to 60 fps, repeat, once or ping-pong), and an animation list above the
  frames that filters the column and the preview.
- A live preview with play, pause and speed that follows what you draw.
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
- Export tab: choose the project, a folder or the selected sprites, export a
  sprite sheet with a JSON file or single PNG files, with scale (1x to 8x),
  space between frames, an output folder inside the project and a naming
  pattern. It shows a preview, the files, a success message with "Open folder",
  and clear error and empty states.
- Exports are the same every time, are written only inside the project folder
  and leave no partial files when they fail. The format is in
  `docs/export-format.md`.
- Palette panel in the editor with a "Show keys" switch (the one-letter key on
  each color) and an Edit button.
- Palette editor: change the name, key and color (hex or RGB sliders) of a
  color, add and remove colors, with Cancel and Done. Done applies everything
  as one undo step, and a key change rewrites every sprite that uses the palette
  and warns that text grids with the old key stop being valid.
- Removing a color that sprites use asks for a replacement (a color or
  transparent) for its pixels.
- Key changes and removals wait while an AI agent is writing to the sprite.
- New palette: empty, from the presets "Pocket RPG 16" and "Farm 32", or
  imported from a `.gpl` (GIMP) or `.hex` file, from the "+" of the palettes
  list. A palette of the list opens in the palette editor.
- Text grid format: a parser, a writer and a validator in the core, with
  precise errors (code, line, row and column) that are all reported together,
  grids written as a region at `x`, `y` with an option to keep existing pixels,
  and legend lines that add new colors to the palette. Applying a grid is one
  undo step, and a grid with any error changes nothing.
- Text view: the Canvas and Text switch shows the sprite as a text grid with
  line numbers, a live preview that keeps the last valid state, a status line,
  marked lines with errors, and Apply, Copy and Paste.
- MCP server inside the app, for AI agents: it listens only on `127.0.0.1`, needs
  a random token kept in `~/.sprixel/` (readable only by you), rejects web page
  requests (`Origin` and `Host` checks) and has limits on size, time and rate.
- You approve each new AI client before it gets any tool, and you choose what
  agents may do: create sprites, edit pixels, delete sprites (off by default)
  and export files. A switch applies to the next call, with no reconnecting.
- AI tools: read the project, sprites (as a text grid), palettes and an image of
  a frame; create folders, sprites and frames; draw with a text grid, single
  pixels or shapes; delete frames; create animations; set palette colors; delete
  sprites; export; and undo the last action of the agent. Each call is atomic,
  one undo step, and errors are exact (`row_length`, `unknown_key`...).
- Agents see a `revision` and can pass `base_revision` to avoid overwriting
  newer changes (`stale_revision`).
- The core records the activity of every agent, shows who edited a sprite last,
  marks the frame an agent is writing, and lets you Stop an agent
  (`stopped_by_user`) and undo only its latest action.
- The `sprixel` command: `sprixel mcp` (the stdio bridge for AI tools, which
  starts the app and reconnects), `mcp add|remove|list` for Claude Code and
  OpenCode, `doctor`, `start`, `status`, `tools` and `tool`, with `--json` and
  documented exit codes.
- Install scripts: `install.sh` (Linux and macOS) and `install.ps1` (Windows)
  install Sprixel for the current user from a GitHub Release, check the SHA-256
  of every download, show the MIT license notice, add the `sprixel` command to
  the PATH and keep your projects, settings and tokens when run again.
- `sprixel update` (and `--check`) installs the latest release with the same
  checksum checks, and `sprixel uninstall` removes the app, the command and the
  PATH entry after asking, keeping your projects and asking separately about
  the local data folder.
- `node scripts/bump-version.mjs` sets the version in every file and checks that
  they match, and can check that a tag matches the code.
- The release pipeline (`release.yml`): on a version tag from `main` it builds
  the app and the command for Linux, macOS (Apple silicon and Intel) and
  Windows, smoke tests the install on each system, and publishes a GitHub
  Release with checksums and the notes from this changelog.
- Documentation of the install, update and uninstall steps and of the first-run
  warnings of an unsigned app.

### Fixed

- `runtime.json` now also has the version of the app.
- The "..." menu of a folder is no longer cut by the folders column.
- The canvas no longer jumps when the "pick a color" hint goes away.
- The project root lists the sprites of every folder.
- An empty project shows the gray ghost, as in the design.
- Opening another project from the menu now shows that project, not the old one.
- The "..." button of a sprite card is at the top right, as in the design.
- The folders column has a "Folders" title, no strong background on the selected
  folder and "..." buttons that show only on hover, as in the design.
- The Editor tab with no sprite shows the gray ghost and a "Go to project" button.

- Error messages under a field now float below it, so the dialog no longer jumps
  when one appears.
- Apply in the Text view says "Grid applied" when it works.
- The Export tab follows the design: a check on the chosen format, "Export
  ready" with a check after exporting, and folder and download icons.
- The palette editor list shows all the colors, with no row cut in half.
- The errors of the Text view now show in the language of the app (Spanish or
  English), with the line and the numbers.

- `sprixel tool` and `sprixel tools` close their app session when they finish,
  so the app no longer reaches its limit of 16 clients.
- `sprixel mcp` sends the tool calls of an AI to the app one after another, in
  order.
- The approval dialog closes after any answer and shows the error when the
  answer cannot be saved.
- New tools `list_projects` and `open_project`, and a new permission "Open
  projects" (off by default).
- `sprixel mcp add claude` installs a skill that teaches Claude Code how to use
  Sprixel; `mcp remove`, `uninstall` and `doctor` know about it.
- `sprixel start` prints the app path and version and warns when the version
  differs from the command. It says that it does not open a project path yet.
- `sprixel tool` saves the PNG of `render_preview` to `~/.sprixel/previews/`
  and prints its path.
- The frame thumbnails and the preview of a big or wide sprite now fit their
  box instead of covering the tools and the neighbors.
- Zooming with Control and the mouse wheel (or a pinch) now zooms only the pixel
  canvas, not the whole app.
- Only one process starts the Sprixel app at a time: several AI tools (each with its
  own `sprixel mcp` bridge) no longer open several windows at once.
- `sprixel mcp add claude` and `doctor` now see a server that is registered but cannot
  connect, instead of saying it is not registered.
- A source build starts the app that was built last (debug or release), not an old
  release build left in `src-tauri/target`.
