# AI and MCP

> **Status: the server, the tools, the `sprixel` command and the Agents tab are
> built** (the `mcp-server-and-tools`, `cli-sprixel` and `agents-ui` specs). The
> **Agents** tab has the approval prompt, the permissions, the activity, and the
> Stop and Undo buttons. An MCP client can also connect directly over HTTP (see
> [Connect without the command](#connect-without-the-command)).

Sprixel is made to be used by an AI. An AI agent such as Claude Code or
OpenCode connects to the running app, reads your project and draws on the real
canvas while you watch. You stay in control: you approve who connects, you
choose what they may do, and you can stop or undo them.

It uses the [Model Context Protocol (MCP)](https://modelcontextprotocol.io/),
an open standard that AI tools already know.

## How the connection works

```
AI tool ──stdio──▶ sprixel mcp ──HTTP + token, 127.0.0.1──▶ Sprixel app
```

1. The app runs an MCP server, only on your own computer.
2. The AI tool starts `sprixel mcp` (a small Go program). It translates between
   the AI tool and the app.
3. `sprixel mcp` reads the access token from a file in `~/.sprixel/`. **You
   never copy a token.**
4. If the app is not running, `sprixel mcp` starts it.

This is the same idea OpenPencil uses: the app is the live server and a small
command is its remote control.

## Connect your AI

First install Sprixel (see the [README](../README.md#install)). Then:

```sh
sprixel mcp add claude      # Claude Code
sprixel mcp add opencode    # OpenCode
sprixel doctor              # checks that everything works
```

What these commands do:

| Command | What it writes |
| --- | --- |
| `sprixel mcp add claude` | Runs `claude mcp add sprixel --scope user -- sprixel mcp` and writes the Sprixel skill to `~/.claude/skills/sprixel/SKILL.md` (see below) |
| `sprixel mcp add opencode` | Adds a local `sprixel` MCP server to the OpenCode config file |

The first time an agent connects, the app asks you to **allow or reject** it.
You can remove the connection at any time with `sprixel mcp remove <client>`
or from the **Agents** tab.

## How each client is configured

`sprixel mcp add` follows the official documentation of each tool (read on
2026-11-03).

**Claude Code** ([MCP docs](https://code.claude.com/docs/en/mcp)). The command
registers a stdio server with Claude Code's own CLI, so Claude Code writes its
own files:

```sh
claude mcp add --scope user sprixel -- sprixel mcp   # add (user scope)
claude mcp remove sprixel --scope user               # remove
claude mcp get sprixel                               # is it registered?
```

The `--` separates the options of Claude Code from the server command. The
user scope is stored in `~/.claude.json` and applies to all projects. Server
names may only use letters, numbers, hyphens and underscores. The docs do not
state the exit codes of `claude mcp`, so Sprixel reads the text of
`claude mcp get` and does not trust an exit code alone.

**OpenCode** ([MCP servers](https://opencode.ai/docs/mcp-servers/),
[Config](https://opencode.ai/docs/config/)). The command edits the global
file `~/.config/opencode/opencode.json` (JSON and JSONC are both valid
OpenCode files). The local server is an entry under `mcp`, and `command` is an
array:

```json
{
  "$schema": "https://opencode.ai/config.json",
  "mcp": {
    "sprixel": { "type": "local", "command": ["sprixel", "mcp"], "enabled": true }
  }
}
```

Sprixel reads the file, changes only the `mcp.sprixel` entry, and writes it
back atomically. A file with comments (JSONC) cannot be rewritten without
losing them, so Sprixel stops and prints the entry for you to paste. The docs
do not document an `opencode mcp add` command and do not name a Windows path;
Sprixel uses `~/.config/opencode` on every system.

## What the AI can do: tools

The AI sees these tools. Frames are numbered from 1, and `x` and `y` count from
0 at the top left. A sprite is named by `name`, with `folder` when two sprites
share a name (or a path in `name`, like `Characters/goblin`). Every tool
description tells the AI the rules, so it does not need this page.

**Read** (they never change the project)

| Tool | What it does |
| --- | --- |
| `get_project` | Project name, revision, folders, palettes and number of sprites |
| `list_sprites` | Sprites with size, frames, palette, animations and who edited them last |
| `get_sprite` | One sprite as a [text grid](text-grid-format.md), with its animations. For a big sprite, `frame` and `region` (`x`, `y`, `width`, `height`) read only a part, and `include_palette` false leaves out the color legend; the part is still a valid grid |
| `get_palette` | The colors with their one-letter keys |
| `render_preview` | A PNG image of a frame, so the AI can see its work (at most 1024 pixels on a side). `background`: `checker` (default, gray checkerboard), `light`, `dark` or `transparent`; the images of `apply_grid`, `set_pixels` and `draw_shape` take it too |
| `render_frames` | One PNG with several frames in a grid (`from`, `to`, `columns`, `scale`, `labels`, `background`), at most 1024 pixels on a side |
| `render_onion` | One PNG with several frames drawn over each other (`frames`, `scale`, `background`): earlier frames are tinted and see-through, the last one is normal |
| `analyze_sprite` | Checks a sprite: loose pixels, gaps in the outline (`outline_key`), unused colors, and how each frame differs from the next (`changed_pixels`, `bbox`, `ground_kept`, which follows the support foot: the `feet` anchor, or the lowest row with a pixel). It lists what it `checked` and `not_checked` and detects the outline key when you give none (`outline_key_used`). Changes nothing |
| `check_tile` | Checks if a frame repeats without seams: compares opposite edges and lists the pixels that differ |
| `render_tiled` | Returns a PNG of a frame repeated 3x3 on a checkerboard, to see the seams of a tile (at most 1024 pixels on a side) |
| `open_sprite` | Asks the editor to show a sprite, so you see what it works on |
| `list_projects` | The recent projects (name and path) and the project that is open now. Needs no permission |
| `get_guide` | Pixel art guidance for the AI, by `topic`: `design`, `animation`, `palette`, `large-sprites`, `faces-and-animals`, `walk-and-attack-cycles`, `game-ready` or `tools` (a map of which tool to use for what). Needs no permission |

**Open projects** (permission "Open projects")

| Tool | What it does |
| --- | --- |
| `open_project` | Opens the project in a folder, as if you opened it. Off by default, because it can open any folder |

**Create** (permission "Create sprites")

| Tool | What it does |
| --- | --- |
| `create_folder` | Creates a folder |
| `create_sprite` | Creates an empty sprite with a size (1 to 512), a folder and a palette |
| `add_frame` | Adds an empty frame, or a copy of another frame, at a place |

**Edit** (permission "Edit pixels")

| Tool | What it does |
| --- | --- |
| `apply_grid` | Writes a text grid into a frame (all of it, or at an `x`,`y` position). A `legend` line with a `#hex` adds a color that is not in the palette yet. In a region (`x`,`y`) a `.` keeps the pixel under it unless `skip_transparent` is false. Takes `dry_run`. Returns an image of the edited frame |
| `set_pixels` | Sets individual pixels (`x`, `y`, `key`). Returns an image of the edited frame |
| `draw_shape` | Draws a line, rectangle, ellipse or fill with a palette key. Returns an image of the edited frame |
| `auto_outline` | Draws a one pixel outline around the shape of a frame (`key`, `connectivity` 4 or 8, `outside_only`, `inner`). With `inner` it warns about parts thinner than 3 pixels (`thin_regions`). Takes `dry_run`. Returns an image of the edited frame |
| `auto_shade` | Shades the pixels of one key with a ramp of tones for a `light` direction (`base`, `keys` from dark to light, optional `region`). Returns `speckled_pixels` and a warning for lone shade pixels. Takes `dry_run`. Returns an image of the edited frame |
| `transform_region` | Flips, mirrors, moves (with `wrap`) or rotates a region of a frame in one step. Takes `dry_run`. Returns an image of the edited frame |
| `copy_region` | Copies a region inside a sprite, or to another frame or sprite (same palette), with an offset and `skip_transparent`. Takes `dry_run`. Returns an image of the edited frame |
| `import_reference` | Draws a PNG from the project folder `references/` into a frame, using the nearest colors of the palette (`width`, `height` scale it down, `max_colors` adds colors). Returns an image of the edited frame |
| `delete_frame` | Removes a frame (a sprite keeps at least one) |
| `set_frame_duration` | Sets how long a frame lasts (`frame`, `milliseconds` 10 to 10000, or `null` for the animation speed) |
| `set_anchor` | Adds or moves a named point of a sprite (`feet`, `hand`...), for the whole sprite or one `frame` |
| `remove_anchor` | Removes an anchor by name |
| `create_tag` | Creates an animation (`sprite`, `name`, `from`, `to`, `fps`, `mode`) |
| `set_palette_color` | Adds a palette color, or changes the name or hex of the color with a key |
| `create_palette` | Creates a palette from a list of colors (a color without a key gets the next free key) |
| `set_sprite_palette` | Makes a sprite use another palette; fails with `keys_missing` if the sprite uses a key the palette lacks |
| `remove_unused_colors` | Removes the colors of a palette that no sprite uses, and returns their keys |
| `generate_ramp` | Adds 3 to 9 darker and lighter tones of one color to a palette, and returns their keys |
| `undo_last_ai_action` | Takes back the last action of this agent, if nothing happened after it |

**Delete and export**

| Tool | What it does |
| --- | --- |
| `delete_sprite` | Moves a sprite to the trash (permission "Delete sprites", off by default) |
| `export` | Writes a sprite sheet with JSON, or PNG files, into a folder inside the project (permission "Export files") |

Every write tool is **atomic**: it is applied whole or not at all, as one entry
of the history by the agent. Every write returns the new `revision` and takes
an optional `base_revision`.

The pixel tools (`apply_grid`, `set_pixels`, `draw_shape`, `transform_region`,
`copy_region` and `import_reference`) return a PNG
of the frame they edited, at most 512 pixels on a side, next to their usual
answer. The AI must look at it and fix what is wrong. Pass `preview` false to
skip the image. With `sprixel tool` the image is saved to a file and its path
is printed.

`apply_grid` (with `x` and `y`), `set_pixels` and `draw_shape` also take two
optional arguments:

- `only_over`: one palette key. Only the pixels that have that key before the
  call change; the others stay. Use it to shade or recolor one material.
- `symmetry`: `horizontal` (left to right), `vertical` (top to bottom) or
  `both`. Every pixel is also written at its mirrored place. The mirror line is
  the center of the sprite, or `axis` (`{x, y}`, whole or half numbers). If a
  mirrored pixel would fall outside the sprite the call fails with
  `out_of_range` and nothing changes.

`transform_region` works on a `region` (default: the whole frame) with the
operations `flip_horizontal`, `flip_vertical`, `mirror` (`direction`), `move`
(`dx`, `dy`, `wrap`) and `rotate` (`degrees` 90, 180 or 270; 90 and 270 need a
square). `copy_region` copies a `region` to `to_x`, `to_y` of the same or
another frame (`to_frame`) or sprite (`to_sprite`, `to_folder`; same palette).

`import_reference` reads a PNG only from the folder `references/` of the open
project. The file path must be relative with no `..`, must not be (or go
through) a symbolic link, must end with `.png`, and is limited to 2 MB and 512
pixels on a side. Colors become the nearest palette color; alpha under 128 is
transparent; `max_colors` allows new colors to be added to the palette, and
`width` and `height` scale the picture down.

### Dry runs, warnings and `erased`

`auto_outline`, `auto_shade`, `transform_region`, `copy_region` and
`apply_grid` take `dry_run` (default false). With `true` nothing is written:
there is no new revision, no history entry and no replay in the editor. The
answer has the same counts as a real call, `dry_run: true`, `would_change` and
an image of what the frame would look like. Warnings come in `warnings`:
`auto_outline` with `inner` (also `thin_regions`, the boxes of parts thinner
than 3 pixels that turn entirely into outline), `auto_shade` (`speckled_pixels`)
and `apply_grid` (a region grid that erased with `skip_transparent` false).
Every write that erased opaque pixels has `erased`.

`analyze_sprite` returns `checked` and `not_checked`. Without `outline_key` it
uses the key of most of the silhouette edge (`outline_key_used`); if there is
no clear one, `outline` is in `not_checked`. `ground_kept` compares the `y` of
the `feet` anchor of two frames, or the lowest row with a pixel if the sprite
has no such anchor (`ground_rule`); `bottom_rows_equal` is the old check.
`get_sprite` shows `null` in `frame_durations` for a frame that follows the
animation speed; `set_frame_duration` takes `milliseconds: null` for that. An
explicit 100 ms cannot be told from the default (the file stores 100 for both).

### Errors

Every error has a stable `code`, a `message` that says how to fix it, and
details when they apply (`line`, `row`, `column`, `valid_keys`, `limit`...).
The most common codes:

| Code | Meaning |
| --- | --- |
| `awaiting_approval`, `client_rejected`, `not_approved` | The person has not allowed this client |
| `permission_denied` | A permission is off (the message names it) |
| `not_a_project` | `open_project` got a folder that does not exist or has no `sprixel.json` |
| `stopped_by_user` | The person pressed Stop; do not continue |
| `stale_revision` | The project changed since `base_revision`; read it again |
| `unknown_topic` | `get_guide` got a topic that is not one of the eight (the error lists them) |
| `newer_changes` | `undo_last_ai_action` would hide a newer change |
| `not_found`, `ambiguous_name`, `name_exists`, `invalid_name` | Sprite, folder and name problems |
| `palette_mismatch`, `palette_not_found` | The `palette:` line of a grid names a palette that is not the one of the sprite, or that does not exist (the message says how to fix it) |
| `keys_missing` | `set_sprite_palette` to a palette that lacks a key the sprite uses |
| `row_length`, `row_count`, `unknown_key`, `no_frame`, `bad_frame`, `region_out`, `too_big` | A text grid or a drawing that is not valid (all problems come together in `errors`) |
| `out_of_range`, `unknown_key` | A pixel outside the sprite, or a key that is not in the palette (`valid_keys` lists the ones that are) |
| `region_out`, `palette_mismatch` | A region that does not fit in the sprite, or a copy between sprites with different palettes |
| `bad_image` | `import_reference` got a file that is not a valid PNG |
| `invalid_path`, `outside_project` | A path that is not allowed (also a reference file outside `references/`) |
| `limit_exceeded` | A limit was passed (see [Security](#security)) |

The same tools can be called from a terminal with the `sprixel` command:

```sh
sprixel tools
sprixel tool get_sprite name=goblin
sprixel tool set_pixels pixels='[{"x":1,"y":2,"key":"a"}]' --json
```

`sprixel tool` connects as the client **Sprixel CLI**, so the app asks you to
allow it once, like any AI. Values are typed by the tool's schema: text stays
text, and numbers, `true`/`false`, lists and objects are written as JSON.
Add `--json` to get the result as JSON only. The exit code is 4 when the tool
answers with an error. Neither `tool` nor `tools` start the app; use
`sprixel start`.

## Drawing tips for an AI

The AI gets this advice from the server itself: the server instructions carry
the workflow and a worked example (a 16x16 slime with an idle and a jump frame),
the tool `get_guide` serves the full guides, and the Claude Code skill has the
same workflow, example and design rules. A test fails if the skill and the
server differ. The guides are:

Guides (`get_guide`, argument `topic`): `design`, `animation`, `palette`, `large-sprites`, `faces-and-animals`, `walk-and-attack-cycles`, `game-ready` and `tools`.

Each guide besides the first three has a worked example with exact calls, and a
test runs those calls through the real tools.

- **Small first.** Text grids work best up to about 32x32. Bigger sprites are
  allowed (up to 512x512) but the AI makes more counting mistakes; for them it
  works region by region with `x` and `y`.
- **Read the palette first** (`get_palette`) and only use its keys.
- **Method.** A small palette with ramps, one light direction, silhouette first,
  flat fills, two or three tones of shading, a selective outline and no stray
  pixels.
- **Look at the result.** Every pixel write returns an image of the frame. The
  AI must look at it and fix what is wrong before it says it is done.
- **Animation.** Key poses first, the same palette in every frame and a loop
  that closes.
- Errors are exact (for example: `row 7 has 7 columns, expected 8`) so the AI
  can fix them on its own.

### Watching the AI draw

Every write of an AI is applied at once, but the editor replays it: a cursor
with the name of the AI moves over the canvas and the pixels appear in drawing
order. The replay takes at most 8 seconds, however big the write is. The frame
is busy until the replay ends, so your strokes wait and are applied after it.
**Stop** shows the final state at once. With "reduce motion" on in your system
the final state appears with no replay. If many writes arrive in a row, the
editor shows the final state instead of playing all of them. When the AI edits
another sprite or frame, the editor follows it, unless you are in the middle of
a stroke or in another tab.

Example prompts:

> "Create a 16x16 goblin in the Characters folder with a 4 frame walk cycle."
>
> "Make the hero blink: add 2 frames and an animation called idle at 6 fps."
>
> "Export the Characters folder as a sprite sheet."

## Permissions

In the **Agents** tab you choose what agents may do. These are enforced inside
the app core, not only in the interface: a switch applies to the **next call** of
every agent, with no reconnecting. They are saved in `~/.sprixel/permissions.json`.

| Permission | Default |
| --- | --- |
| Create sprites (`create_folder`, `create_sprite`, `add_frame`) | On |
| Edit pixels (the drawing and structure tools, and `undo_last_ai_action`) | On |
| Delete sprites (`delete_sprite`) | **Off** |
| Export files (`export`) | On |
| Open projects (`open_project`) | **Off** |
| Ask me when a new agent connects | On |

Everything an agent does is saved in the activity list (for the session) with
its name and time. Calls that failed are listed too; reading is not.

- **Stop** cancels the call in progress before anything is applied, and the
  agent's next write calls fail with `stopped_by_user` until you let it
  continue. Stop is by agent name and also stops its other sessions.
- **Undo** reverts the last action. It is offered only on the most recent AI
  action, while nothing else changed the project after it. Exported files
  cannot be undone.

## Connect without the command

Any MCP client can connect to the running app:

1. Read `~/.sprixel/runtime.json` (`{"port": ..., "pid": ...}`) and
   `~/.sprixel/token`.
2. Send JSON-RPC 2.0 requests with `POST http://127.0.0.1:<port>/mcp`, the
   header `Authorization: Bearer <token>` and a `Content-Length` (no chunked
   bodies). Send no `Origin` header.
3. Call `initialize` with your `clientInfo` (name and version). The answer has
   the `Mcp-Session-Id` header: send it on every later request.
4. The app asks you to allow the client. Until then `tools/list` is empty and
   `tools/call` fails with `awaiting_approval`.

The server speaks `initialize`, `ping`, `tools/list` and `tools/call`, with
the protocol versions 2025-06-18, 2025-03-26 and 2024-11-05. Images from
`render_preview`, `render_frames` and `render_onion` come as MCP image content.

## Security

The server can change your files, so it is closed by default.

- Listens on **`127.0.0.1` only** (never the network), on a free port written to
  `runtime.json` (removed when the app stops).
- Needs a random **token** (256 bits) on every request. It is stored in
  `~/.sprixel/token` in a folder only your user can read (mode 0600 and 0700 on
  Linux and macOS, an access list with only your user on Windows), compared in
  constant time and never written to a log. Delete the file to rotate it.
- Checks the `Host` header (only the local address and port) and **rejects
  browser requests**: any request with an `Origin` header, even with a valid
  token. A web page you visit cannot control Sprixel. No CORS header is sent.
- A **new AI client must be approved** by you. A client is known by its name and
  version; a new version asks again.
- Paths from a client are checked: no `..`, no absolute paths. Exported files
  go **inside the project folder**, never in hidden folders or in the folders
  that hold the project files, and a link that points outside is refused.
- **Limits** (answered with `limit_exceeded`): 4 MB per request, 10 seconds to
  send a request, 100 requests per second for each client, 16 clients at the
  same time, 32 connections open at once, 16 clients waiting for an answer, an
  export of at most 64 million pixels.
- Writes are applied **one at a time**.

Note: the name an agent gives itself ("Claude Code") is only a label. The real
protection is the token and your approval. A program of your own user that can
read the token file can use the app like an agent; that is the limit of this
design.

## The `sprixel` command

| Command | What it does |
| --- | --- |
| `sprixel mcp` | The MCP server over stdio that AI tools start. It starts the app if needed (waits up to 20 seconds) and reconnects if the app restarts |
| `sprixel mcp add <claude\|opencode>` | Registers it. Safe to run twice; prints what changed and how to undo it. For Claude Code it also writes the skill again if it is missing or old |
| `sprixel mcp remove <claude\|opencode>` | Removes only the Sprixel entry (and, for Claude Code, the skill folder) |
| `sprixel mcp list` | Shows if Sprixel is registered in each client |
| `sprixel doctor` | One line per check (OK, FAIL or SKIP) with how to fix a failure, including whether the Claude Code skill is installed and current; exit code 1 if one fails |
| `sprixel start` | Starts the app, returns once the server answers, and prints the app path and version (it warns if the version differs from the command). It does not open a project: use the app or `open_project` |
| `sprixel status` | Whether the app runs, with its port and version (exit code 3 if closed) |
| `sprixel tools`, `sprixel tool <name> key=value ...` | List and call tools |
| `sprixel version` | The version of the command |

Every command accepts `--json` (valid JSON only on stdout, also for errors)
and `--help`. Exit codes: **0** ok, **1** failure (for example a failed doctor
check), **2** usage error, **3** the app is not running, **4** a tool answered
with an error.

`sprixel tool` and `sprixel tools` end their session in the app when they
finish, so many calls never fill the limit of clients. `sprixel mcp` runs the
`tools/call` messages of an AI one after another, in the order they arrived.

When a tool returns an image (`render_preview`, `render_frames`, `render_onion`),
`sprixel tool` saves it as a PNG in
`~/.sprixel/previews/<sprite>-<frame>-<revision>.png` (or
`<sprite>-<frames|onion>-<revision>-<hash>.png` for the two new tools; readable only by
you), keeps the 20 newest files and prints `image saved: <path>`, so an AI that
only has a shell can open the file and see its work. With `--json` each image
keeps its `data` (base64) and has a `path` too.

### The Claude Code skill

`sprixel mcp add claude` installs a small skill that teaches Claude Code how to
use Sprixel: how to connect, the order of the tools, the rules for coordinates
and frames, how to design a sprite that looks good (with a worked example), and
what to do with each error code. The file comes inside the
`sprixel` command (no download). `sprixel mcp remove claude` and
`sprixel uninstall` delete its folder, `~/.claude/skills/sprixel/`, and
nothing else of Claude Code.

### Which app starts

`sprixel start` prints the path and the version of the app. If the version is
not the one of the command, it warns: an old build in another place is the
usual reason. Choose the app by setting `SPRIXEL_APP` to its path.

Where the command looks for the app to start it: `SPRIXEL_APP` (a path), then
next to the command (`sprixel-app`), then the usual places of each system
(`~/.local/bin/sprixel-app` on Linux, `/Applications/Sprixel.app` on macOS,
`%LOCALAPPDATA%\Sprixel\Sprixel.exe` on Windows), and the build folder when
you built from source. Linux needs `DISPLAY` or `WAYLAND_DISPLAY`; without a
screen the command says that Sprixel needs a desktop session.

## Two things to know

- **Concurrency.** If you and an agent edit the same frame, the agent's
  operation finishes first and your strokes wait a moment (the frame shows who
  is drawing). The agent can pass a `base_revision`; if the project changed,
  the tool fails with `stale_revision` and the agent reads again.
- **No app, no AI.** The tools need the app to be running. `sprixel mcp` starts
  it when it can. On a computer with no screen (a server) it cannot.

## Not in version 1

- Codex and other agents (any MCP client can connect, but only Claude Code and
  OpenCode get the one-command setup).
- A headless mode (running the app without a window).
- Remote access over a network.
- The "connection lost" state of an agent after 10 seconds (planned).
