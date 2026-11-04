<p align="center">
  <img src="docs/design/assets/sprixel-icon.png" width="128" alt="Sprixel icon">
</p>

<h1 align="center">Sprixel</h1>

<p align="center">
  <b>Pixel art for games.</b><br>
  Draw sprites and animations, or let your AI draw them for you.
</p>

<p align="center">
  <img alt="Status" src="https://img.shields.io/badge/status-building%20the%20demo-F5C542?style=flat-square&labelColor=17171c">
  <img alt="Platforms" src="https://img.shields.io/badge/platforms-Linux%20%7C%20Windows%20%7C%20macOS-F5C542?style=flat-square&labelColor=17171c">
  <img alt="License" src="https://img.shields.io/badge/license-MIT-F5C542?style=flat-square&labelColor=17171c">
</p>

<p align="center">
  <img src="docs/design/oscuro/19-editor.png" alt="Sprixel editor with onion skin and an AI agent drawing" width="860">
</p>

## What is Sprixel?

Sprixel is a desktop pixel art editor made for **game sprites**. It is inspired
by Piskel, Aseprite and Pyxel Edit, with a clean and simple interface, and one
big difference: **your AI can use it**.

Claude Code and OpenCode connect to the running app and draw on the real canvas
while you watch. You can step in at any moment, stop the AI, or undo what it did.

## Highlights

- **Sprites and animations.** Frames, named animations (walk, idle, attack),
  live preview with FPS and onion skin.
- **Any canvas size.** Presets from 8x8 to 64x64, or any size up to 512x512
  (for example 120x30).
- **Everything in a project.** Folders and sprites, kept as plain text files
  that work well with Git. Your work is saved automatically.
- **Built for AI.** A simple text grid format and an MCP server let an AI
  create, draw and export sprites. You decide what the AI is allowed to do.
- **Ready for your game.** Export a sprite sheet with a JSON file, single
  PNG files or an animated GIF.
- **Your language and your theme.** Spanish and English, light and dark.

## Try it (from the source code)

> **There is no published release yet.** For now you run Sprixel from the source
> code. The one-line installers below come with the first release.

You need Node and npm, Rust, Go and Git. The steps for each system are in
[Contributing](CONTRIBUTING.md#prerequisites).

```sh
git clone https://github.com/Javier-GarciaP/sprixel.git
cd sprixel
git switch dev              # the demo is built on the dev branch
npm ci
npm run tauri dev           # opens the app
npm run build:cli           # builds the sprixel command into bin/
```

The demo needs the AI connection, which is being built. Follow the
[roadmap](docs/roadmap.md) to see how far it is.

## Install

> **The first release (0.1.0) is not published yet.** The commands below work
> once it is. Until then, run Sprixel from the source code (see above).

Linux and macOS:

```sh
curl -fsSL https://github.com/Javier-GarciaP/sprixel/releases/latest/download/install.sh | sh
```

Windows (PowerShell):

```powershell
irm https://github.com/Javier-GarciaP/sprixel/releases/latest/download/install.ps1 | iex
```

The installer puts the app and the `sprixel` command on your computer. Then
connect your AI with one command:

```sh
sprixel mcp add claude      # Claude Code
sprixel mcp add opencode    # OpenCode
sprixel doctor              # check that everything works
```

What the installer does:

- It installs **for your user only**. It never asks for administrator rights.
- It downloads the files of the GitHub Release and **checks their SHA-256**
  before it installs anything. If a check fails, nothing is installed.
- It shows the MIT license notice and the source URL first, and installs the
  license and the third-party notices with the app.
- It adds the `sprixel` command to your PATH if needed and tells you to open a
  new terminal.
- It is safe to run again: it upgrades or repairs the installation and keeps
  your projects, settings and tokens.

Supported systems: Linux x64, macOS (Apple silicon and Intel) and Windows x64.

To install a specific version, add `--version`:

```sh
curl -fsSL https://github.com/Javier-GarciaP/sprixel/releases/latest/download/install.sh | sh -s -- --version 0.2.0
```

```powershell
$env:SPRIXEL_VERSION = "0.2.0"; irm https://github.com/Javier-GarciaP/sprixel/releases/latest/download/install.ps1 | iex
```

Update and remove Sprixel with the same command:

```sh
sprixel update              # install the latest version
sprixel uninstall           # remove Sprixel (your projects stay)
```

**Warnings the first time.** Version 1 is not signed, so your system may warn
you when you open the app. This is expected. On macOS, right-click Sprixel in
`~/Applications`, choose **Open** and confirm. On Windows, in the SmartScreen
window choose **More info** and then **Run anyway**. On Linux the app is an
AppImage, which may need FUSE 2 (`libfuse2`) on some distributions. See the
[usage guide](docs/usage.md#install) for more.

> Sprixel is **open source** under the [MIT License](#license). You can use it,
> change it and share it, and contributions are welcome.

## Documentation

| Document | What is inside |
| --- | --- |
| [Usage guide](docs/usage.md) | How to use the app and the `sprixel` command |
| [AI and MCP](docs/ai-and-mcp.md) | Connecting an AI, available tools and security |
| [Text grid format](docs/text-grid-format.md) | The text format an AI uses to draw sprites |
| [Export format](docs/export-format.md) | The sprite sheet, the JSON and the PNG files |
| [Game engines](docs/game-engines.md) | From a drawing to Godot 4 and Unity, step by step |
| [Project format](docs/project-format.md) | Format versions and the rules for changing the project files |
| [Architecture](docs/architecture.md) | How the app, the CLI and the project files fit together |
| [Design](docs/design/README.md) | The idea behind Sprixel and all the screens (light and dark) |
| [Roadmap](docs/roadmap.md) | The plan for version 1 and the specs behind it |
| [Contributing](CONTRIBUTING.md) | Branches, commits, versions and releases |
| [Third-party software](docs/third-party.md) | Fonts and dependencies, and their licenses |

## Status

Sprixel is built from specs written in [`openspec/`](openspec/), one at a time.
The foundation, the project storage and the app shell are done. The current goal
is a **demo where an AI draws and animates sprites in the editor while you
watch**, and then version 1. Follow the [roadmap](docs/roadmap.md).

If you want to help, read [Contributing](CONTRIBUTING.md). Issues and pull
requests are welcome.

## License

Copyright (c) 2026 Javier-GarciaP. Released under the **MIT License**. Read the
full text in [LICENSE](LICENSE). The fonts and other third-party software have
their own licenses, listed in [Third-party software](docs/third-party.md).
