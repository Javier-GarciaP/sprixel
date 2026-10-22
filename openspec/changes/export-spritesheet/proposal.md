# Proposal

## Why

The whole point of making sprites is to use them in a game. Export turns a
project into files an engine can read.

## What Changes

- Build the Export tab with three steps: what to export, the format, the
  options, and one Export button.
- Export a sprite sheet (one PNG) with a JSON file describing frames and
  animations.
- Export single PNG files and animated GIFs.
- Support scale, spacing, output folder, trimming, pivot and naming options.
- Show the result, the error states and an "Open folder" action.

## Capabilities

### New Capabilities
- `export`: exporting sprites to a sheet with JSON, PNG files and GIF, with options and clear results.

### Modified Capabilities

## Impact

- New feature folder `src/features/export/` and the Rust modules `render` and
  `export`.
- The JSON layout becomes a public format: later changes need a version.
- The `export` MCP tool uses the same module.
