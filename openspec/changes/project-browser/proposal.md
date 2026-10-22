# Proposal

## Why

The Project tab is the center of the app: it is where a person sees every
sprite, keeps them in order and decides what to open, rename, move or delete.

## What Changes

- Build the Project tab: folders tree on the left, sprites in the middle,
  details on the right.
- Build New sprite (size presets and any size up to 512x512), New folder,
  rename, duplicate, move and delete.
- Delete sprites by sending them to the trash with an Undo message, and ask
  only for folders and for groups.
- Build multiple selection with an action bar.
- Add search, context menus, keyboard actions and the empty states.

## Capabilities

### New Capabilities
- `project-browser`: seeing and finding the folders, sprites and palettes of the project.
- `sprite-management`: creating, renaming, duplicating, moving and deleting sprites and folders.

### Modified Capabilities

## Impact

- New feature folder `src/features/project-browser/`.
- Uses the document operations and the trash from `project-format-and-storage`
  and the shell from `app-shell-and-settings`.
- Thumbnails need a small rendering helper shared with the editor preview.
