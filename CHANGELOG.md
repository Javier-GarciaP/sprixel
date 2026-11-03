# Changelog

All notable changes to Sprixel are written here.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)
and the project follows [Semantic Versioning](https://semver.org/).

## [Unreleased]

### Added

- Edit tools for agents: `transform_region` (flip, mirror, move with `wrap`,
  rotate) and `copy_region` (inside a sprite or to another frame or sprite),
  `only_over` and `symmetry` on `set_pixels`, `draw_shape` and `apply_grid`,
  and `region` and `include_palette` on `get_sprite` to read big sprites cheaply.
- `import_reference` draws a PNG from the `references/` folder of the project
  into a frame, reduced to the sprite palette (optionally adding colors).
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
