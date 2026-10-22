# Design

## Context

See proposal.md and the screens 24 to 27 in `docs/design/`. The JSON follows
the idea of the Aseprite sprite sheet JSON (frames with `frame`, `duration`
and `frameTags`), so existing importers can be adapted, but the exact layout is
defined here.

## Goals / Non-Goals

**Goals:**
- Deterministic, simple files that engines can read.
- Clear success and error results.

**Non-Goals:**
- Engine specific formats (Godot resources, Unity assets). They can be built
  later from the JSON.
- Packing algorithms beyond a simple row layout.

## Decisions

- **Simple row packer** (rows of frames, left to right, wrapping by width).
  Deterministic and easy to understand. A smarter packer can come later.
- **JSON layout** with `meta` (app, version, image, size, scale), `frames`
  (ordered list with `name`, `frame {x,y,w,h}`, `duration`, `pivot`) and
  `animations` (`name`, `from`, `to`, `mode`, `fps`).
- **Rendering in Rust** with the `image` crate for PNG and a GIF encoder,
  shared by the app and the MCP tool.
- **Write to a temporary folder and move** the files so no partial output is
  left on failure.
- **No compatibility claim** with any engine is made in the interface until
  verified in a later change.

## Risks / Trade-offs

- [Engines expect different layouts] → keep the JSON documented and versioned
  (`meta.version`).
- [Large exports take time] → show progress and run off the UI thread.
- [GIF has 256 colors and 1-bit transparency] → fine for pixel art with a
  small palette; documented.
