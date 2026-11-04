# Sprixel guide

This is the long guide to Sprixel: how to install it, how to use it, and how an
AI connects to it and learns to use it. The [README](../README.md) is the short
version. For every detail, each part links to a reference page.

## Contents

1. [Install](#install)
   - [Method 1: one command](#method-1-one-command)
   - [Method 2: download from GitHub](#method-2-download-from-github)
   - [The first time you open it](#the-first-time-you-open-it)
   - [Update and uninstall](#update-and-uninstall)
2. [Use the app](#use-the-app)
3. [Connect an AI](#connect-an-ai)
4. [How the AI understands Sprixel](#how-the-ai-understands-sprixel)
   - [Layer 1: the server instructions](#layer-1-the-server-instructions)
   - [Layer 2: the tools](#layer-2-the-tools)
   - [Layer 3: the guides (`get_guide`)](#layer-3-the-guides-get_guide)
   - [Layer 4: the Claude Code skill](#layer-4-the-claude-code-skill)
   - [The drawing loop](#the-drawing-loop)
5. [You stay in control](#you-stay-in-control)
6. [When something goes wrong](#when-something-goes-wrong)
7. [Where to read more](#where-to-read-more)

---

## Install

Sprixel has **two parts**, and you need both:

| Part | What it is |
| --- | --- |
| **The app** | The pixel art editor (a desktop window). It also runs the MCP server that an AI talks to. |
| **The `sprixel` command** | A small program for the terminal. It connects your AI to the app, and it can start, update and check Sprixel. |

Both ways below install both parts. Supported systems: Linux x64, macOS (Apple
silicon and Intel) and Windows x64. Nothing needs administrator rights.

### Method 1: one command

This is the easiest way. The script downloads the files of the latest release,
checks them, and puts everything in the right place.

Linux and macOS:

```sh
curl -fsSL https://github.com/Javier-GarciaP/sprixel/releases/latest/download/install.sh | sh
```

Windows (PowerShell):

```powershell
irm https://github.com/Javier-GarciaP/sprixel/releases/latest/download/install.ps1 | iex
```

What the script does:

- Installs **for your user only**.
- Downloads the files of the release and **checks their SHA-256**. If a check
  fails, nothing is installed.
- Shows the MIT license notice first.
- Adds the `sprixel` command to your PATH if needed. **Open a new terminal
  afterwards.**
- Is safe to run again. It upgrades or repairs the install and keeps your
  projects, settings and tokens.

To install a specific version:

```sh
curl -fsSL https://github.com/Javier-GarciaP/sprixel/releases/latest/download/install.sh | sh -s -- --version 0.1.0
```

```powershell
$env:SPRIXEL_VERSION = "0.1.0"; irm https://github.com/Javier-GarciaP/sprixel/releases/latest/download/install.ps1 | iex
```

Check that it worked:

```sh
sprixel doctor
```

### Method 2: download from GitHub

Use this if you do not want to run a script. You download the files yourself
from the [Releases page](https://github.com/Javier-GarciaP/sprixel/releases)
and put them in place. You need **two files** for your system, plus
`checksums.txt` to check them:

| System | The app | The `sprixel` command |
| --- | --- | --- |
| Linux | `Sprixel-linux-x64.AppImage` | `sprixel-linux-x64` |
| macOS, Apple silicon | `Sprixel-macos-arm64.app.tar.gz` | `sprixel-macos-arm64` |
| macOS, Intel | `Sprixel-macos-x64.app.tar.gz` | `sprixel-macos-x64` |
| Windows | `Sprixel-windows-x64.zip` | `sprixel-windows-x64.exe` |

**1. Check the files.** `checksums.txt` has the SHA-256 of every file. On Linux
and macOS, in the folder with your downloads:

```sh
sha256sum -c checksums.txt --ignore-missing       # Linux
shasum -a 256 -c checksums.txt --ignore-missing   # macOS
```

On Windows: `Get-FileHash .\Sprixel-windows-x64.zip` and compare the result
with the line in `checksums.txt`.

**2. Put the files in place.** These are the same places the script uses, and
they are also the first places where `sprixel start` looks for the app.

Linux:

```sh
mkdir -p ~/.local/bin
install -m 755 Sprixel-linux-x64.AppImage ~/.local/bin/sprixel-app
install -m 755 sprixel-linux-x64 ~/.local/bin/sprixel
```

macOS (use `arm64` or `x64` in the names):

```sh
mkdir -p ~/Applications ~/.local/bin
tar -xzf Sprixel-macos-arm64.app.tar.gz -C ~/Applications
install -m 755 sprixel-macos-arm64 ~/.local/bin/sprixel
```

Windows (PowerShell):

```powershell
$dir = "$env:LOCALAPPDATA\Sprixel"
New-Item -ItemType Directory -Force "$dir\bin" | Out-Null
Expand-Archive .\Sprixel-windows-x64.zip -DestinationPath $dir -Force
Copy-Item .\sprixel-windows-x64.exe "$dir\bin\sprixel.exe"
```

**3. Add the command folder to your PATH**, so the terminal finds `sprixel`.
`~/.local/bin` on Linux and macOS, `%LOCALAPPDATA%\Sprixel\bin` on Windows.
Open a new terminal and run `sprixel doctor`.

> The app alone is enough to draw by hand. Without the `sprixel` command you
> cannot use `sprixel mcp add` to connect an AI (see
> [Connect without the command](ai-and-mcp.md#connect-without-the-command)).

### The first time you open it

Version 0.1.0 is **not signed**, so your system may warn you. This is expected.

- **macOS:** right-click Sprixel in `~/Applications`, choose **Open** and
  confirm once. A file you download with the browser is marked by macOS; the
  install script does not trigger this.
- **Windows:** in the SmartScreen window choose **More info** and then **Run
  anyway**.
- **Linux:** the app is an AppImage. If it does not start, install FUSE 2
  (`libfuse2` on Debian and Ubuntu, `fuse-libs` on Fedora).

### Update and uninstall

```sh
sprixel update --check      # only say if a new version exists
sprixel update              # install the latest version
sprixel uninstall           # remove Sprixel (your projects stay)
```

`sprixel update` checks the SHA-256 of every file before it replaces anything.
`sprixel uninstall` asks before it removes the app, the command, the PATH entry
and the AI registrations. **It never removes your projects.**

More about install paths and options: [Usage guide](usage.md#install).

---

## Use the app

Start Sprixel from your applications menu, or with `sprixel start`.

1. **Create a project.** A project is a folder of plain text files. It works
   well with Git, and it is saved automatically.
2. **Create a sprite.** Choose a preset (8x8 up to 64x64) or any size up to
   512x512.
3. **Draw.** Pick a color from the palette, use the tools, and zoom with
   Control and the mouse wheel. Every stroke can be undone.
4. **Add frames and animations.** Draw a base frame, then make one named
   animation for each action (idle, walk, attack). The preview plays it with
   its speed, and onion skin shows the previous frames.
5. **Export.** Open the Export tab and write a sprite sheet with a JSON file, or
   single PNG files, inside your project folder. GIF export is not ready yet.

Each step has more detail in the [Usage guide](usage.md). The same sprites can
also be read and written as text. See the [text grid format](text-grid-format.md).

---

## Connect an AI

An AI tool such as **Claude Code** or **OpenCode** can use the running app and
draw on the real canvas while you watch. It uses the
[Model Context Protocol (MCP)](https://modelcontextprotocol.io/), an open
standard that AI tools already know.

```
AI tool ──stdio──▶ sprixel mcp ──HTTP + token, 127.0.0.1──▶ Sprixel app
```

- The **app** is the MCP server. It listens only on your own computer
  (`127.0.0.1`), never on the network.
- The **`sprixel mcp`** command is a small bridge. The AI tool starts it, and it
  talks to the app. If the app is closed, it starts the app.
- The access **token** is a file in `~/.sprixel/`. The bridge reads it for you.
  You never copy a token.

To connect your AI, once:

```sh
sprixel mcp add claude      # Claude Code
sprixel mcp add opencode    # OpenCode
sprixel doctor              # check that everything works
```

Or open the **Agents** tab in the app and press **Connect**: it shows the
command to copy and says **Connected** when the AI arrives.

The first time an AI connects, the app asks you to **Allow** or **Reject** it.
Until you answer, its tools fail with `awaiting_approval`. Then ask your AI
for something:

> "Create a 16x16 goblin in the Characters folder with a 4 frame walk cycle."

You will see a cursor with the name of the AI move over the canvas and the
pixels appear in drawing order.

More: [AI and MCP](ai-and-mcp.md).

---

## How the AI understands Sprixel

An AI does not know Sprixel in advance. The app teaches it, in layers. Each
layer goes deeper, and the AI reads a layer only when it needs it. This keeps
the AI fast and makes it follow the same rules as a good pixel artist.

### Layer 1: the server instructions

When the AI connects, the server sends a short text with the **workflow** and a
worked example (a 16x16 slime with an idle frame and a jump frame). The AI
receives it with any MCP client, with nothing else installed. The workflow is:

1. `get_project`: read the project.
2. `get_guide`: read the guide for the job (see layer 3).
3. `get_palette`: read the colors, and draw only with those keys.
4. `create_sprite` or `get_sprite`: make or read a sprite.
5. Draw with `apply_grid`, `set_pixels` or `draw_shape`.
6. **Look at the image** that every write returns, fix what is wrong, and look
   again.

### Layer 2: the tools

The AI sees more than 45 tools, each with a description that already says how to
use it. They are grouped by what they do, and each group has its own
permission:

| Group | Examples | Permission |
| --- | --- | --- |
| Read | `get_project`, `get_sprite`, `get_palette`, `render_preview`, `analyze_sprite` | Always allowed |
| Create | `create_sprite`, `create_folder`, `add_frame` | Create sprites |
| Edit | `apply_grid`, `set_pixels`, `draw_shape`, `auto_outline`, `auto_shade`, `transform_region`, `create_animation`, `set_palette_color` | Edit pixels |
| Export | `export` | Export files |
| Delete | `delete_sprite` | Delete sprites (**off** by default) |
| Open projects | `open_project` | Open projects (**off** by default) |

The AI draws with a **text grid**: one character per pixel, one line per row,
where `.` is always transparent. This is what `apply_grid` takes:

```
legend
  K outline #1E3A2B
  B base #6FD36B
frame 1
.KKKK.
KBBBBK
.KKKK.
```

Every error has a stable `code` and a `message` that says how to fix it (for
example `row 7 has 7 columns, expected 8`), so the AI can correct itself. The
full list of tools is in [AI and MCP](ai-and-mcp.md#what-the-ai-can-do-tools).

### Layer 3: the guides (`get_guide`)

`get_guide` is a tool that returns **pixel art advice**, by `topic`. The AI asks
for the guide that fits the job. The guides live inside the app, so they always
match the tools of that version. There are nine:

| Topic | What it teaches |
| --- | --- |
| `design` | The method: a small palette with ramps, one light direction, silhouette first, flat fills, two or three tones of shading, a selective outline, no stray pixels |
| `animation` | How to animate: key poses first, the same palette in every frame, a loop that closes |
| `animations-from-base` | Draw a base frame, then make one named animation per action and change only what moves |
| `palette` | How to choose and build the colors |
| `large-sprites` | Sprites bigger than 32x32: work region by region with `x` and `y` |
| `faces-and-animals` | How to draw faces and animals |
| `walk-and-attack-cycles` | How to make walk and attack animations |
| `game-ready` | What a game needs from a sprite: one size, anchors (feet, hand, head), frame times, seamless tiles and the export |
| `tools` | A map of which tool to use for what |

Most guides have a worked example with exact calls, and a test runs those
calls through the real tools, so the examples do not go out of date.

Two rules are repeated everywhere (the server instructions, the guides and the
skill):

- **Base first, then animations.** Use `create_animation`, which copies the base
  frame, and change only what moves.
- **Never make pixels outside the app** (scripts, image libraries) unless you
  agree.

### Layer 4: the Claude Code skill

`sprixel mcp add claude` also writes a skill to
`~/.claude/skills/sprixel/SKILL.md`. It carries the same workflow, the same
example and the design rules, and it adds how to connect and what to do when
there are no MCP tools (only a shell, using `sprixel tool ...`). The skill comes
inside the `sprixel` command, so nothing is downloaded. A test fails if the
skill and the server say different things. `sprixel mcp remove claude` and
`sprixel uninstall` remove its folder and nothing else of Claude Code.

The skill is only for Claude Code. OpenCode and other clients learn from layers
1 to 3, which are the main source anyway.

### The drawing loop

An AI is better when it can **see** its work. Every tool that changes pixels
returns a PNG of the frame it edited. The AI must look at it and fix what is
wrong before it says it is done. Safer tools help it:

- `dry_run` on the big edits shows what **would** happen (counts, warnings and
  an image) and writes nothing.
- `analyze_sprite` finds loose pixels, gaps in the outline and unused colors,
  and says what it did not check.
- `render_frames` and `render_onion` show many frames at once.
- `base_revision` makes a write fail with `stale_revision` if the project
  changed, so the AI reads again instead of overwriting your work.

---

## You stay in control

- **You approve** every new AI client, and a new version of a client asks again.
- **You choose** what it may do with the permission switches in the **Agents**
  tab. A switch applies to the next call, with no reconnecting.
- **You can stop it.** **Stop** cancels the call in progress, and the AI's next
  writes fail with `stopped_by_user`.
- **You can undo it.** **Undo** reverts the last AI action, as one step.
- **You see everything** it did in the activity list, with its name and time.
- **It is closed by default.** The server listens only on `127.0.0.1`, needs a
  random token on every request, rejects requests from web pages, and only
  writes exports inside your project folder.

Details and limits: [AI and MCP: Permissions](ai-and-mcp.md#permissions) and
[Security](ai-and-mcp.md#security).

---

## When something goes wrong

Run `sprixel doctor` first. It checks the install, whether the app runs, the
token and your AI registrations, and says how to fix what fails.

| Problem | What to try |
| --- | --- |
| The AI cannot find Sprixel | Run `sprixel mcp add claude` again, then `sprixel doctor` |
| The tools fail with `awaiting_approval` | Press **Allow** for the AI in the Agents tab |
| The app is closed | Run `sprixel start` |
| An AI action was wrong | Press **Undo** in the editor or in the Agents tab |
| The app does not start on Linux | Install FUSE 2 (`libfuse2` or `fuse-libs`) |

More: [Usage guide: Troubleshooting](usage.md#troubleshooting).

---

## Where to read more

| Page | What is inside |
| --- | --- |
| [Usage guide](usage.md) | Every screen and shortcut of the app, and the `sprixel` command |
| [AI and MCP](ai-and-mcp.md) | Every tool, every error code, permissions and security |
| [Text grid format](text-grid-format.md) | The text format an AI uses to draw |
| [Export format](export-format.md) | The sprite sheet, the JSON and the PNG files |
| [Game engines](game-engines.md) | From a drawing to Godot 4 and Unity |
