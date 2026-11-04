# Project format

A Sprixel project is a folder of text files (see
[architecture](architecture.md) for the layout). This page defines the
**format version** and the rules for changing the format. The code is in
`src-tauri/src/storage/`.

## Format version

Every file (`sprixel.json`, `*.palette.json`, `*.sprite.json`) has an integer
`formatVersion`.

| Version | Sprixel | Notes |
| --- | --- | --- |
| 1 | 0.1.0 and later | The first format. |

Fields added to version 1 without a new version (they are optional, so older
files stay valid):

- `anchors` in a sprite file (named points). A sprite without anchors does not
  write the field.
- `parts` in a sprite file (named rectangles `{name, x, y, width, height}` that
  the region tools use by name, at most 64, names unique ignoring case). A
  sprite without parts does not write the field, so old files open unchanged
  and stay byte for byte the same. A part is not checked against the size of
  the sprite when the file is read (a canvas change can leave it outside); the
  tools say `region_out` when such a part is used.
- A sprite file without `frameDurations` (the very first files) is accepted:
  every frame lasts 100 ms.

A file **without** `formatVersion` is version 1. A `formatVersion` that is not
a whole number is an error (the file is damaged).

## What happens when a project opens

1. **The same version:** the project opens. Nothing is written.
2. **An older version:** the project files are copied to
   `.sprixel/backups/v<version>-<time>/`, every file is migrated by running
   **one step per version, in order** (`1 -> 2`, `2 -> 3`, ...), and the files
   are written atomically (the manifest last, so an interrupted migration runs
   again). A missing step is an error: no version is skipped. A file that
   cannot be migrated is left as it is and listed as damaged.
3. **A newer version:** the project is **refused** with the message
   "saved with a newer Sprixel (format version N); update the app". Nothing is
   written, not even a backup. A single sprite or palette file with a newer
   version in an otherwise valid project is listed as damaged and not touched.

A file that opens without a migration (no `formatVersion`, no
`frameDurations`) is written in the current form the next time it is saved.

## Rules for changing the format

1. **Prefer an optional field.** A new field that has a default does not need
   a new version: old files open, and files that do not use it do not change.
2. **Raise the version** (`FORMAT_VERSION` in `storage/format.rs`) when the
   change cannot be a default: a field is renamed, removed or changes meaning,
   or old data must be rewritten.
3. **Add one step** to `Migrator::app()` (`storage/migrate.rs`) for the new
   version. A step works on the JSON of one file and never drops data it does
   not understand.
4. **Add a fixture** of the old version in `src-tauri/src/storage/testdata/`
   and a test that opens it. Keep every old fixture.
5. **Update this table** and the layout in `architecture.md`.
6. Old Sprixel versions must refuse the new files (rule 3 of the section
   above does it): never write a file that an old version would read wrongly.

## Fixtures

| Folder in `storage/testdata/` | What it checks |
| --- | --- |
| `dungeon-kit` | The current format (also the sample of the docs tests). |
| `old-kit-no-version` | Files with no `formatVersion`, no `frameDurations` and no `anchors`. |
| `old-kit-v1-no-anchors` | Version 1 files written before anchors existed (they have no `parts` either). |
