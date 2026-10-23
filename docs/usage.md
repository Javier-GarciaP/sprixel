# Usage guide

> **Status: planned.** Sprixel is not built yet. This guide describes how
> version 1 will work, so you can see the goal. It will be checked against the
> real app when each feature is finished. Screens are in
> [docs/design](design/README.md).

## Contents

- [Install](#install)
- [Create a project](#create-a-project)
- [Create a sprite](#create-a-sprite)
- [Draw](#draw)
- [Frames and animations](#frames-and-animations)
- [Palette](#palette)
- [Text view](#text-view)
- [Work with an AI](#work-with-an-ai)
- [Export](#export)
- [Settings](#settings)
- [Keyboard shortcuts](#keyboard-shortcuts)
- [The `sprixel` command](#the-sprixel-command)
- [Where your files are](#where-your-files-are)
- [Troubleshooting](#troubleshooting)

## Install

See the [README](../README.md#install). After installing, check everything with:

```sh
sprixel doctor
```

Start the app from your applications menu, or with `sprixel start`.

## Create a project

A project is a folder on your computer. It holds all your sprites.

1. On the Welcome screen, press **New project** (or `Ctrl+N`).
2. Type a name. **Save in** starts at `~/Proyectos`; press **Choose...** to
   pick another folder. A line under the fields shows the folder that will be
   created, for example `~/Proyectos/Dungeon Kit/`.
3. Press **Create project**. The folder is created only now.

The project opens in the **Project** tab. The project name (with a menu arrow),
the save status and the tabs appear in the top bar, and the window title reads
`Dungeon Kit - Sprixel`.

If a folder with that name already exists, the dialog says so and creates
nothing. A name that cannot be a folder name (for example with `/`) is refused
too.

**Starting palette:** the list only offers **No palette (empty)** for now. The
project is created with no palette. Ready-made palettes arrive with the palette
editor spec.

Open a project you already have with **Open project** (or `Ctrl+O`) and pick
its folder. If the folder has no `sprixel.json`, a message says "This folder is
not a Sprixel project" and nothing is opened. Recent projects appear on the
Welcome screen and in **Open recent** in the project menu; click one to open it.
**Close project** in the project menu returns to the Welcome screen.

If the folder of a recent project was moved or deleted, a message says the
folder was not found and you can remove it from the list.

The project saves by itself. The top bar shows **Saved** or **Saving...**. If
saving fails it shows **Error saving** and a message with the reason and
**Retry**.

## Create a sprite

1. In the **Project** tab, press **New sprite**.
2. Type a name and choose a folder.
3. Choose the canvas size: a preset (8x8, 16x16, 32x32, 64x64) or any width and
   height up to 512x512, for example 120x30.
4. Press **Create sprite**. The editor opens.

Use folders to keep things in order (characters, enemies, items). Create them
with **New folder**. Right-click a sprite or folder for more actions: open,
rename, duplicate, move and delete. Deleted items go to the trash and a message
lets you **Undo**.

You can select several sprites with the checkboxes to move or delete them.

## Draw

The tools are on the left of the editor:

| Tool | Key |
| --- | --- |
| Pencil | `B` |
| Eraser | `E` |
| Bucket (fill) | `G` |
| Eyedropper | `I` |
| Line | `L` |
| Rectangle | `R` |
| Ellipse | `C` |
| Select | `M` |
| Move | `V` |

- Pick colors from the palette on the right. The two color boxes under the
  tools are your main and secondary colors.
- Zoom with `+`, `-` or `Ctrl` + mouse wheel. Move the view by holding `Space`.
- Undo with `Ctrl+Z` and redo with `Ctrl+Shift+Z`.

## Frames and animations

- The column next to the tools lists the **frames**. Press **+** to add one.
- **Onion skin** shows the previous frame in blue under your drawing, so you
  can see how the movement flows. Turn it on and set its strength in the
  right panel (`O` toggles it).
- The **preview** plays the animation. Choose the speed in FPS.
- An **animation** is a name plus a range of frames. In the Project tab,
  select a sprite and press **+** in the **Animations** box. For example
  `walk`: frames 1 to 4, 12 fps, repeating.
- The animation menu above the frames filters the frames you see.

## Palette

The palette lists your colors. Each color has a **name** and a **one-letter
key** (the key is what the text view and the AI use).

- Press **Edit** on the palette box to open the palette editor.
- Change a color, its name or its key, or **add a color**. Changes apply to
  every sprite that uses the palette.
- Turn **Show keys** on to see the letters on the colors.

Careful: changing a key breaks text grids that still use the old one. The editor
warns you.

## Text view

Switch from **Canvas** to **Text** (top right of the editor) to see the sprite
as a text grid, with a live preview.

- Edit the text and press **Apply**. Mistakes are shown with their line and
  row, and nothing is applied until it is valid.
- **Copy** and **Paste** let you move grids in and out of the app.

The format is explained in [Text grid format](text-grid-format.md).

## Work with an AI

1. Connect your AI once (see [AI and MCP](ai-and-mcp.md)):

   ```sh
   sprixel mcp add claude
   ```

2. In your AI tool, ask for what you need:

   > "Create a 16x16 goblin in Characters with a 4 frame walk cycle."

3. Allow the connection the first time. Then watch the sprite appear in the
   editor. An amber tag in the editor shows who is drawing.
4. Press **Stop** to cancel the agent, or **Undo** to revert its last action.

In the **Agents** tab you can see everything the AI did and choose what it may
do (create, edit, delete, export).

## Export

Open the **Export** tab and follow the three steps:

1. **What to export:** the whole project, one folder, or a selection.
2. **Format:**
   - **Sheet + JSON:** one PNG with all frames, plus a JSON file with the
     frames, durations and animations.
   - **Single PNG files:** one image per frame.
   - **GIF:** an animated GIF per animation.
3. **Options:** scale (1x to 8x), spacing between frames, and the folder to
   save in (inside the project, `assets/sprites` by default).

Press **Export**. When it finishes, press **Open folder**.

## Settings

Open **Settings** from the bottom left corner of the window. It has two pages.

**Appearance and language**

- **Theme:** Light, Dark or Follow the system. With "Follow the system" the
  app changes when your operating system changes. The choice applies at once.
- **Language:** Spanish or English. On the first start Sprixel uses your system
  language if it is Spanish or English, and English otherwise. Text changes at
  once, with no restart.
- **Canvas:** show a pixel grid when you zoom in (on by default).

**Keyboard shortcuts:** the full list, grouped by area (see below).

Theme and language also have quick buttons in the bottom bar.

Your preferences and your recent projects are saved on your computer, in the
Sprixel config folder (for example `~/.config/com.sprixel.app/preferences.json`
on Linux). They are never saved inside a project, so a project copied to
another computer does not carry them.

## Keyboard shortcuts

On macOS use `Cmd` instead of `Ctrl`. The same list is in
**Settings > Keyboard shortcuts**.

**Project**

| Command | Shortcut |
| --- | --- |
| New project | `Ctrl+N` |
| Open... | `Ctrl+O` |
| Export... | `Ctrl+E` |
| Open in editor | `Enter` |
| Rename | `F2` |
| Duplicate | `Ctrl+D` |
| Delete | `Delete` |

**Tools**

| Command | Shortcut |
| --- | --- |
| Pencil | `B` |
| Eraser | `E` |
| Bucket | `G` |
| Eyedropper | `I` |
| Line | `L` |
| Rectangle | `R` |
| Ellipse | `C` |
| Select | `M` |
| Move | `V` |

**Editor and view**

| Command | Shortcut |
| --- | --- |
| Undo | `Ctrl+Z` |
| Redo | `Ctrl+Shift+Z` |
| Previous frame | `,` |
| Next frame | `.` |
| Zoom in | `+` |
| Zoom out | `-` |
| Fit to window | `Ctrl+0` |
| Move the view | `Space` |
| Onion skin | `O` |

The project menu (click the project name) has these commands: New project, Open, Open recent, Rename project, Save a copy as, Show
in folder, Export and Close project. There is no Save command because the
project saves by itself. Some commands work only when the feature that owns
them is built.

## The `sprixel` command

| Command | What it does |
| --- | --- |
| `sprixel start [path]` | Start the app, optionally opening a project |
| `sprixel status` | Say if the app is running |
| `sprixel mcp add claude` | Connect Claude Code |
| `sprixel mcp add opencode` | Connect OpenCode |
| `sprixel mcp remove <client>` | Disconnect an AI tool |
| `sprixel tools` | List the tools an AI can use |
| `sprixel tool <name> key=value ...` | Run one tool from the terminal |
| `sprixel doctor` | Check the install and the connections |
| `sprixel update` | Update to the latest version |
| `sprixel uninstall` | Remove Sprixel |
| `sprixel version` | Show the version |

## Where your files are

| What | Where |
| --- | --- |
| Your projects | The folders you choose. Plain text files that work with Git. |
| Sprixel local data (port, token, approved clients) | `~/.sprixel/` (`%USERPROFILE%\.sprixel` on Windows) |
| Deleted items | `.sprixel/trash/` inside each project |

See [Architecture](architecture.md#the-project-on-disk) for the file format.

## Troubleshooting

Run `sprixel doctor` first. It checks the installation, whether the app is
running, the access token and your AI registrations, and says how to fix what
fails.

| Problem | What to try |
| --- | --- |
| The AI cannot find Sprixel | Run `sprixel mcp add claude` again, then `sprixel doctor`. |
| "Waiting for connection" never ends | Open the app, check that the AI tool is installed, and run `sprixel status`. |
| An AI action was wrong | Press **Undo** in the editor or in the Agents tab. |
| "Error saving" appears | Check the disk space and the folder permissions, then press **Retry**. |
