# Proposal

## Why

All the features live inside one window layout. The shell also decides how a
person starts (welcome, new project, open project), and settings decide the
language and theme for the whole app.

## What Changes

- Build the window layout from the design: top bar (logo, project menu, save
  status, tabs, agents indicator), body and bottom bar (settings, language,
  theme, context).
- Build the Welcome screen, the New project dialog, Open project and the list of
  recent projects, and the project menu.
- Disable the Editor, Export and Agents tabs when no project is open.
- Add light, dark and system themes, and Spanish and English with translation
  files.
- Build Settings (appearance and language, keyboard shortcuts) and the
  shortcut system.

## Capabilities

### New Capabilities
- `app-shell`: the window layout, tabs, project menu, welcome flow and recent projects.
- `app-settings`: theme, language, preferences and keyboard shortcuts.

### Modified Capabilities

## Impact

- New `app` and `settings` code in `src/`, shared layout components in `src/shared/`.
- Translation files for Spanish and English (every later feature adds its keys).
- App preferences are stored in the system config folder, not in projects.
- Adds the fonts Space Grotesk, Inter and DM Mono to the app bundle.
