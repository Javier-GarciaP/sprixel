# AI and MCP

> **Status: planned.** Nothing is built yet. This is how the AI integration
> will work in version 1. The exact tool list is defined in the
> `mcp-server-and-tools` spec.

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
| `sprixel mcp add claude` | Runs `claude mcp add sprixel --scope user -- sprixel mcp` |
| `sprixel mcp add opencode` | Adds a local `sprixel` MCP server to the OpenCode config file |

The first time an agent connects, the app asks you to **allow or reject** it.
You can remove the connection at any time with `sprixel mcp remove <client>`
or from the **Agents** tab.

## What the AI can do: tools

The AI sees these tools. Names are proposed and may change in the spec.

**Read**

| Tool | What it does |
| --- | --- |
| `get_project` | Project name, folders, palette name and counts |
| `list_sprites` | Sprites with size, frames and animations |
| `get_sprite` | One sprite as a [text grid](text-grid-format.md) |
| `get_palette` | The colors with their one-letter keys |
| `render_preview` | An image of a sprite or frame, so the AI can see its work |
| `open_sprite` | Shows a sprite in the editor, so you see what it works on |

**Write**

| Tool | What it does |
| --- | --- |
| `create_folder` | Creates a folder |
| `create_sprite` | Creates a sprite with a size, a folder and a palette |
| `apply_grid` | Writes a text grid into a frame (all of it, or at an `x`,`y` position). A `legend` line with a `#hex` adds a color that is not in the palette yet |
| `set_pixels` | Sets individual pixels (`x`, `y`, `key`) |
| `draw_shape` | Draws a line, rectangle, ellipse or fill |
| `add_frame` / `delete_frame` | Adds (or copies) and removes frames |
| `create_tag` | Creates an animation (name, from, to, fps, mode) |
| `set_palette_color` | Adds or changes a palette color |
| `delete_sprite` | Moves a sprite to the trash (needs permission) |
| `export` | Exports a sheet, PNG files or a GIF |
| `undo_last_ai_action` | Reverts the last action made by an AI |

The same tools can be called from a terminal:

```sh
sprixel tools
sprixel tool get_sprite name=goblin
```

## Drawing tips for an AI

- **Small first.** Text grids work best up to about 32x32. Bigger sprites are
  allowed (up to 512x512) but the AI makes more counting mistakes.
- **Read the palette first** (`get_palette`) and only use its keys.
- **Use `apply_grid`** with the legend, and `set_pixels` or `draw_shape` to fix
  details. For big sprites, draw one region at a time with `x` and `y`.
- **Check the result** with `render_preview`.
- Errors are exact (for example: `row 7 has 7 columns, expected 8`) so the AI
  can fix them on its own.

Example prompts:

> "Create a 16x16 goblin in the Characters folder with a 4 frame walk cycle."
>
> "Make the hero blink: add 2 frames and an animation called idle at 6 fps."
>
> "Export the Characters folder as a sprite sheet."

## Permissions

In the **Agents** tab you choose what agents may do. These are enforced inside
the app, not only in the interface.

| Permission | Default |
| --- | --- |
| Create sprites | On |
| Edit pixels | On |
| Delete sprites | **Off** |
| Export files | On |
| Ask me when a new agent connects | On |

Everything an agent does is saved in the activity list with its name and time.
**Stop** cancels the work in progress. **Undo** reverts the last action.
Exported files cannot be undone.

## Security

The server can change your files, so it is closed by default.

- Listens on **`127.0.0.1` only** (never the network), on a random free port.
- Needs a random **token** on every request. It is stored in a file only your
  user can read.
- Checks the `Host` header and **rejects browser requests** (any request with
  an `Origin` header). A web page you visit cannot control Sprixel.
- A **new AI client must be approved** by you.
- File access and exports are **limited to the project folder**.
- Requests have a **size limit**, and writes are applied one at a time.

Note: the name an agent gives itself ("Claude Code") is only a label. The real
protection is the token and your approval.

## Two things to know

- **Concurrency.** If you and an agent edit the same frame, the agent's
  operation finishes first and your strokes wait a moment. The agent can pass a
  `base_revision`; if the project changed, the tool fails with
  `stale_revision` and the agent reads again.
- **No app, no AI.** The tools need the app to be running. `sprixel mcp` starts
  it when it can. On a computer with no screen (a server) it cannot.

## Not in version 1

- Codex and other agents (any MCP client can connect, but only Claude Code and
  OpenCode get the one-command setup).
- A headless mode (running the app without a window).
- Remote access over a network.
