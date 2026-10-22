# Design

## Context

See proposal.md and the screens in `docs/design/`. The window layout is
fixed-size regions with scrolling inside the body. The design uses one set of
color tokens with a dark and a light value.

## Goals / Non-Goals

**Goals:**
- Theme and language that switch instantly.
- A layout that later features plug into without changing the shell.

**Non-Goals:**
- Any project content (that is `project-browser` and `pixel-canvas-editor`).
- More languages than Spanish and English.

## Decisions

- **Design tokens as CSS variables** with a `data-theme` attribute, mapped in
  Tailwind. Alternative: two Tailwind themes duplicated by hand (more drift).
- **A small i18n layer** with JSON files per language and typed keys.
  Alternative: a large library (more dependencies, not needed for two
  languages). The decision is confirmed when the dependency is approved.
- **Native menu from Tauri** built from the same list of commands as the
  project menu, so there is one source for command names and shortcuts.
- **A command registry** (id, label key, shortcut, action) used by menus,
  shortcuts, the shortcuts page and tests.
- **Preferences in the Tauri app config directory.**
- **The shell leaves the body to features.** Tabs map to features through
  `index.ts` exports only.

## Risks / Trade-offs

- [Two sets of translations drift apart] → a test fails when a key is missing
  in English and warns when it is missing in Spanish.
- [Shortcuts clash with the system or the web view] → keep the list short and
  test the main ones on each system.
- [Native menus differ across systems] → keep only the common commands there.
