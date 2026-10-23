# Sprixel design

This folder holds the idea behind Sprixel and every screen of version 1, in
light and dark. It is the starting point to understand the project.

- [The idea](#the-idea)
- [How it works](#how-it-works)
- [Design system](#design-system)
- [Screens](#screens)
- [Known differences between the designs and the specs](#known-differences-between-the-designs-and-the-specs)
- [Source files](#source-files)

## The idea

Making sprites for a 2D game takes many small steps: draw, animate, name,
organize and export to the game engine. Good editors exist (Piskel, Aseprite,
Pyxel Edit), but none of them was made to work together with an AI.

**Sprixel is a pixel art editor focused on game sprites, where an AI agent is a
real user of the app.** The agent can read the project and draw on the same
canvas you are looking at, at the same time as you.

**Who is it for?** Indie game developers and pixel artists. The interface is
available in Spanish and English, Spanish first.

### Principles

1. **Clear before clever.** Every screen shows one obvious next step. Nobody
   should need help to use it.
2. **The AI is a guest, you are in control.** You can see what it does, stop
   it, undo it and choose what it may do.
3. **Plain files.** A project is a folder of text files: it works with Git and
   both people and AIs can read it.
4. **Ready for any engine.** Export a sprite sheet and a JSON file that
   describes the frames and animations.
5. **Small and fast.** A focused tool, not a big suite.

## How it works

### The project model

```
Project  (a folder on your disk)
├── Palette           colors, each with a name and a one-letter key
└── Folders
    └── Sprites       a size, and one or more frames
        └── Animations    tags: a name, from-to frames, speed, repeat mode
```

- The **canvas size** is chosen when the sprite is created (8x8 to 512x512).
- An **animation** is a tag on a sprite (for example "walk" = frames 1 to 4 at
  12 fps). It is not a separate file.
- Version 1 has **one layer** per sprite. The file format already allows more.

### The three main flows

1. **Draw by hand.** New project, new sprite, choose a size, draw with the
   tools, add frames, tag an animation, watch the preview.
2. **Draw with an AI.** Connect Claude Code or OpenCode with one command. Ask
   for "a goblin, 16x16, with a 4 frame walk". The sprite appears on the real
   canvas. Stop or undo at any time.
3. **Export.** Choose what to export (project, folder or selection), the format
   (sheet and JSON, PNG files or GIF) and the options, then press Export.

## Design system

Screens are designed at 1440 x 900.

### Colors

| Name | Hex | Use |
| --- | --- | --- |
| Amber | `#F5C542` | Brand, main buttons, selected items |
| Ink | `#17171C` | Dark surfaces and text on amber |
| Paper | `#F3EFE6` | Light theme background |
| Cream | `#FFF8E1` | The ghost body |
| Rose | `#F59DB0` | The ghost cheeks |
| Success | `#4ADE80` dark, `#1E9E55` light | Done, connected, valid |
| Warning | `#FB923C` dark, `#C2570C` light | Careful |
| Error | `#F87171` dark, `#C62828` light | Errors, delete |

### Typography

| Font | Use |
| --- | --- |
| Space Grotesk (700) | Titles |
| Inter (400 to 700) | Interface text |
| DM Mono (400) | Data: sizes, keys, file names, code |

### Look and feel

Retro and modern at the same time: hard offset shadows on the main buttons,
clear borders, mono data and pixel art everywhere, with a clean and calm
layout. The pixel canvas uses a neutral gray checkerboard in both themes so
black outlines stay visible.

Menus and dialogs always open **over the screen they come from**, so you can
see what opened them.

### The ghost

The app icon is a small ghost on a 16x16 grid, drawn with integer scaling
(never blurred). A simplified 8x8 version is used at 16 px. The gray version
is used for empty states.

<p align="center">
  <img src="assets/sprixel-icon.png" width="160" alt="Sprixel icon">
</p>

## Screens

Each screen has a dark and a light version.

### Start
#### 01 · Welcome

The first screen when no project is open. One clear next step: create or open a project. The other tabs are disabled until a project exists.

| Dark | Light |
| --- | --- |
| <img src="oscuro/01-bienvenida.png" width="430" alt="Welcome, dark"> | <img src="claro/01-bienvenida.png" width="430" alt="Welcome, light"> |
#### 02 · New project

Name, location and starting palette. Sprixel creates one folder with your sprites inside.

| Dark | Light |
| --- | --- |
| <img src="oscuro/02-nuevo-proyecto.png" width="430" alt="New project, dark"> | <img src="claro/02-nuevo-proyecto.png" width="430" alt="New project, light"> |

### Project
#### 03 · Empty project

A new project with no sprites yet. One main action: create the first sprite.

| Dark | Light |
| --- | --- |
| <img src="oscuro/03-proyecto-vacio.png" width="430" alt="Empty project, dark"> | <img src="claro/03-proyecto-vacio.png" width="430" alt="Empty project, light"> |
#### 04 · New folder

A small dialog over the project screen. Folders keep sprites ordered (characters, enemies, items).

| Dark | Light |
| --- | --- |
| <img src="oscuro/04-nueva-carpeta.png" width="430" alt="New folder, dark"> | <img src="claro/04-nueva-carpeta.png" width="430" alt="New folder, light"> |
#### 05 · New folder: name already exists

The field turns red and explains the problem. The create button stays disabled.

| Dark | Light |
| --- | --- |
| <img src="oscuro/05-nueva-carpeta-error.png" width="430" alt="New folder: name already exists, dark"> | <img src="claro/05-nueva-carpeta-error.png" width="430" alt="New folder: name already exists, light"> |
#### 06 · Project

The heart of the project: folders on the left, sprites in the middle and details on the right. A sprite shows its animations (a name, a range of frames and a speed).

| Dark | Light |
| --- | --- |
| <img src="oscuro/06-proyecto.png" width="430" alt="Project, dark"> | <img src="claro/06-proyecto.png" width="430" alt="Project, light"> |
#### 07 · Multiple selection

Select several sprites with the checkboxes. A bar appears with Move and Delete.

| Dark | Light |
| --- | --- |
| <img src="oscuro/07-proyecto-seleccion-multiple.png" width="430" alt="Multiple selection, dark"> | <img src="claro/07-proyecto-seleccion-multiple.png" width="430" alt="Multiple selection, light"> |
#### 08 · Delete several sprites

Deleting a group asks for confirmation. Items go to the system trash, so they can be recovered.

| Dark | Light |
| --- | --- |
| <img src="oscuro/08-borrar-varios-sprites.png" width="430" alt="Delete several sprites, dark"> | <img src="claro/08-borrar-varios-sprites.png" width="430" alt="Delete several sprites, light"> |
#### 09 · Project menu

Opens from the project name in the top bar. The app has no native window menu: every command is in this menu and has its shortcut. The project saves by itself, so there is no Save command.

| Dark | Light |
| --- | --- |
| <img src="oscuro/09-menu-del-proyecto.png" width="430" alt="Project menu, dark"> | <img src="claro/09-menu-del-proyecto.png" width="430" alt="Project menu, light"> |
#### 10 · Sprite menu

Right-click (or the ... button) on a sprite: open, rename, duplicate, move or delete.

| Dark | Light |
| --- | --- |
| <img src="oscuro/10-menu-del-sprite.png" width="430" alt="Sprite menu, dark"> | <img src="claro/10-menu-del-sprite.png" width="430" alt="Sprite menu, light"> |
#### 11 · Sprite menu: move to folder

The submenu to move a sprite to another folder, or to a new one.

| Dark | Light |
| --- | --- |
| <img src="oscuro/11-menu-del-sprite-mover-a-carpeta.png" width="430" alt="Sprite menu: move to folder, dark"> | <img src="claro/11-menu-del-sprite-mover-a-carpeta.png" width="430" alt="Sprite menu: move to folder, light"> |
#### 12 · Sprite deleted

Deleting one sprite does not ask. It goes to the trash and a message offers Undo for a few seconds.

| Dark | Light |
| --- | --- |
| <img src="oscuro/12-sprite-borrado.png" width="430" alt="Sprite deleted, dark"> | <img src="claro/12-sprite-borrado.png" width="430" alt="Sprite deleted, light"> |
#### 13 · Delete folder

A folder holds many sprites, so this one asks first. The focus starts on Cancel.

| Dark | Light |
| --- | --- |
| <img src="oscuro/13-borrar-carpeta.png" width="430" alt="Delete folder, dark"> | <img src="claro/13-borrar-carpeta.png" width="430" alt="Delete folder, light"> |
#### 14 · New animation

An animation is a tag on a sprite: a name, from-to frames, speed and what happens at the end.

| Dark | Light |
| --- | --- |
| <img src="oscuro/14-nueva-animacion.png" width="430" alt="New animation, dark"> | <img src="claro/14-nueva-animacion.png" width="430" alt="New animation, light"> |
#### 15 · New sprite

The size is chosen here, once: presets (8x8 to 64x64) or any size up to 512x512.

| Dark | Light |
| --- | --- |
| <img src="oscuro/15-nuevo-sprite.png" width="430" alt="New sprite, dark"> | <img src="claro/15-nuevo-sprite.png" width="430" alt="New sprite, light"> |
#### 16 · New sprite: large size

A warning for big canvases: AI drawing is less accurate and the editor can be slower.

| Dark | Light |
| --- | --- |
| <img src="oscuro/16-nuevo-sprite-tamano-grande.png" width="430" alt="New sprite: large size, dark"> | <img src="claro/16-nuevo-sprite-tamano-grande.png" width="430" alt="New sprite: large size, light"> |

### Editor
#### 17 · Editor with no sprite

The Editor tab with nothing open. It sends you to the Project tab.

| Dark | Light |
| --- | --- |
| <img src="oscuro/17-editor-sin-sprite.png" width="430" alt="Editor with no sprite, dark"> | <img src="claro/17-editor-sin-sprite.png" width="430" alt="Editor with no sprite, light"> |
#### 18 · Editor: new empty sprite

A blank canvas with a short hint to get started.

| Dark | Light |
| --- | --- |
| <img src="oscuro/18-editor-vacio.png" width="430" alt="Editor: new empty sprite, dark"> | <img src="claro/18-editor-vacio.png" width="430" alt="Editor: new empty sprite, light"> |
#### 19 · Editor

Frames on the left, the canvas in the middle and the preview on the right. The previous frame appears in blue (onion skin). Here an AI agent is drawing frame 2 and you can stop it.

| Dark | Light |
| --- | --- |
| <img src="oscuro/19-editor.png" width="430" alt="Editor, dark"> | <img src="claro/19-editor.png" width="430" alt="Editor, light"> |
#### 20 · Editor: save error

The top bar and a message show that saving failed, with a Retry button.

| Dark | Light |
| --- | --- |
| <img src="oscuro/20-editor-error-al-guardar.png" width="430" alt="Editor: save error, dark"> | <img src="claro/20-editor-error-al-guardar.png" width="430" alt="Editor: save error, light"> |
#### 21 · Editor: text view

The same sprite as a text grid, with a live preview. This is the format an AI uses to draw.

| Dark | Light |
| --- | --- |
| <img src="oscuro/21-editor-texto.png" width="430" alt="Editor: text view, dark"> | <img src="claro/21-editor-texto.png" width="430" alt="Editor: text view, light"> |
#### 22 · Editor: text view with an error

Errors name the line and the row, and the preview keeps the last valid state.

| Dark | Light |
| --- | --- |
| <img src="oscuro/22-editor-texto-con-error.png" width="430" alt="Editor: text view with an error, dark"> | <img src="claro/22-editor-texto-con-error.png" width="430" alt="Editor: text view with an error, light"> |
#### 23 · Palette editor

Colors have a name and a one-letter key. Changes apply to every sprite that uses the palette.

| Dark | Light |
| --- | --- |
| <img src="oscuro/23-editar-paleta.png" width="430" alt="Palette editor, dark"> | <img src="claro/23-editar-paleta.png" width="430" alt="Palette editor, light"> |

### Export
#### 24 · Export

Three steps in order: what to export, which format, which options. One big button at the end.

| Dark | Light |
| --- | --- |
| <img src="oscuro/24-exportar.png" width="430" alt="Export, dark"> | <img src="claro/24-exportar.png" width="430" alt="Export, light"> |
#### 25 · Export: nothing to export

The steps are dimmed and the export button is disabled until there is a sprite.

| Dark | Light |
| --- | --- |
| <img src="oscuro/25-exportar-vacio.png" width="430" alt="Export: nothing to export, dark"> | <img src="claro/25-exportar-vacio.png" width="430" alt="Export: nothing to export, light"> |
#### 26 · Export: done

A green confirmation with a button to open the folder.

| Dark | Light |
| --- | --- |
| <img src="oscuro/26-exportado.png" width="430" alt="Export: done, dark"> | <img src="claro/26-exportado.png" width="430" alt="Export: done, light"> |
#### 27 · Export: error

A clear message with the reason and a way to retry.

| Dark | Light |
| --- | --- |
| <img src="oscuro/27-exportar-error.png" width="430" alt="Export: error, dark"> | <img src="claro/27-exportar-error.png" width="430" alt="Export: error, light"> |

### AI agents
#### 28 · Agents

Connected AI agents, what they did (with Stop and Undo) and what they are allowed to do.

| Dark | Light |
| --- | --- |
| <img src="oscuro/28-agentes.png" width="430" alt="Agents, dark"> | <img src="claro/28-agentes.png" width="430" alt="Agents, light"> |
#### 29 · Agents: none connected

Two agents can be connected. The activity area waits for the first one.

| Dark | Light |
| --- | --- |
| <img src="oscuro/29-agentes-vacio.png" width="430" alt="Agents: none connected, dark"> | <img src="claro/29-agentes-vacio.png" width="430" alt="Agents: none connected, light"> |
#### 30 · Agents: connection lost

The agent card and the top bar turn red. The work in progress stops.

| Dark | Light |
| --- | --- |
| <img src="oscuro/30-agentes-conexion-perdida.png" width="430" alt="Agents: connection lost, dark"> | <img src="claro/30-agentes-conexion-perdida.png" width="430" alt="Agents: connection lost, light"> |
#### 31 · Connect an AI: Claude Code

Three steps: copy the command, paste it in a terminal, wait for the connection.

| Dark | Light |
| --- | --- |
| <img src="oscuro/31-conectar-agente-claude-code.png" width="430" alt="Connect an AI: Claude Code, dark"> | <img src="claro/31-conectar-agente-claude-code.png" width="430" alt="Connect an AI: Claude Code, light"> |
#### 32 · Connect an AI: OpenCode

Same dialog with the OpenCode tab.

| Dark | Light |
| --- | --- |
| <img src="oscuro/32-conectar-agente-opencode.png" width="430" alt="Connect an AI: OpenCode, dark"> | <img src="claro/32-conectar-agente-opencode.png" width="430" alt="Connect an AI: OpenCode, light"> |
#### 33 · Approve a connection

The first time an agent connects, you decide. The top bar shows the waiting state.

| Dark | Light |
| --- | --- |
| <img src="oscuro/33-aprobar-conexion.png" width="430" alt="Approve a connection, dark"> | <img src="claro/33-aprobar-conexion.png" width="430" alt="Approve a connection, light"> |

### Settings and brand
#### 34 · Settings: appearance and language

Theme (light, dark, system), language (Spanish, English) and canvas options.

| Dark | Light |
| --- | --- |
| <img src="oscuro/34-ajustes.png" width="430" alt="Settings: appearance and language, dark"> | <img src="claro/34-ajustes.png" width="430" alt="Settings: appearance and language, light"> |
#### 35 · Settings: keyboard shortcuts

All shortcuts in three groups: project, tools, and editor and view.

| Dark | Light |
| --- | --- |
| <img src="oscuro/35-ajustes-atajos.png" width="430" alt="Settings: keyboard shortcuts, dark"> | <img src="claro/35-ajustes-atajos.png" width="430" alt="Settings: keyboard shortcuts, light"> |
#### 36 · Brand

The ghost icon on a 16x16 grid, its sizes, colors and fonts.

| Dark | Light |
| --- | --- |
| <img src="oscuro/36-identidad.png" width="430" alt="Brand, dark"> | <img src="claro/36-identidad.png" width="430" alt="Brand, light"> |

## Known differences between the designs and the specs

The designs were finished before the technical specs. Where they differ, **the
specs win**. The designs will be updated when the matching feature is built.

| Design shows | The specs say |
| --- | --- |
| "Connect an AI" shows a long `claude mcp add ... --header` command with a token | The command is `sprixel mcp add claude` (or `opencode`). The `sprixel` command reads the token from a local file, so you never copy it. See [AI and MCP](../ai-and-mcp.md). |
| The text view shows Spanish words (`paleta:`, `frame 1`) | The keywords in a text grid are always English (`palette`, `frame`). The editor can show translated labels around them. See [Text grid format](../text-grid-format.md). |
| The Spanish interface only | Every text comes from a translation file in Spanish and English. |
| The Project menu is a drop-down | There is no native window menu (File, Edit, View); every command lives in this menu and has a shortcut. |
| "Import PNG" was removed | It is not part of version 1. |
| Settings has two pages | More pages can be added later. |
| Sample data (Dungeon Kit, goblin, slime) | Mock data to show the screens. |

## Source files

| File | What it is |
| --- | --- |
| `sprixel.op` | The editable design, made with [OpenPencil](https://openpencil.dev). Open it with `op start --file docs/design/sprixel.op`. It holds all the screens in a grid: dark on top, light below. |
| `oscuro/` | The 36 screens in the dark theme (PNG, 1440 x 900). The folder name is Spanish ("oscuro" = dark). |
| `claro/` | The 36 screens in the light theme (PNG, 1440 x 900). ("claro" = light.) |
| `assets/` | The app icon. |

The file names are in Spanish because the designs were drawn in Spanish.
