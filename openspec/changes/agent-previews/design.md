# Design

## Context

`render_frame` draws one frame as RGBA with transparent pixels left
transparent. The export uses it too, and must keep transparency. The preview
tools encode that image as PNG (`export::encode_png`), so a viewer shows the
transparent pixels on its own background (often white or black).

## Goals / Non-Goals

**Goals:**
- The agent always sees a sprite, whatever its colors.
- The agent can judge an animation in one image.
- Small code, no new dependency.

**Non-Goals:**
- **GIF.** An animated GIF needs a new dependency (an encoder). It is out of
  scope. `render_frames` gives the same information in a PNG.
- Changing the export. It keeps transparent pixels.

## Decisions

- **Background is applied after drawing, in the preview module.**
  `render_frame` stays as it is (export needs it). A new `render/preview.rs`
  has `Background`, `flatten` (puts an image over a background), `render_sheet`
  and `render_onion`. The preview tools call it.
- **Default `checker`.** Two grays from the editor (`#B4B4B4`, `#9C9C9C`).
  Squares are half a sprite pixel wide (at least 1 px), so they are not read as
  sprite pixels. `light` is white, `dark` is `#202020`, `transparent` keeps the
  alpha channel. This changes the default of `render_preview`; it is the point
  of the spec (a white sprite must be visible).
- **Same size cap.** All images are at most 1024 px on a side (the write
  preview keeps its 512). Too big gives `too_big` with a `max_scale`.
- **`render_frames`.** Arguments: `name`, `folder`, `from`, `to` (frames, from 1,
  default all), `columns` (default: about a square), `scale` (default: the
  largest of 1 to 8 that fits), `labels` (default false) and `background`. Each
  frame is one cell; cells are separated by a gap of one sprite pixel. With
  labels a band above each cell shows the frame number, drawn with a built-in
  3x5 pixel font (digits only). Label color: white on `dark`, black on the rest.
- **`render_onion`.** Arguments: `name`, `folder`, `frames` (a list, default all),
  `scale` (default as above), `background`. Frames are drawn in the order given.
  The last one is opaque and normal. Each earlier frame is tinted red (half way
  to `#FF4040`) and translucent; older frames are fainter (alpha from 90 to 190).
  Where frames overlap, the newest wins.
- **Files in `sprixel tool`.** The app sends the image and a JSON text with
  `name`, `frame` and `revision`. The new tools send `kind` (`frames` or
  `onion`) instead of `frame`, and the file is
  `<sprite>-<kind>-<revision>-<hash>.png`, where the hash is 4 bytes of the
  image, so two different ranges never overwrite each other.
- **Write tools.** `background` is read in `write()` before anything changes, so
  a wrong value gives `bad_argument` and changes nothing.

## Risks / Trade-offs

- The new default changes what old agents see. It is better for them, and the
  option `transparent` brings the old image back.
- Many frames at a big scale hit the cap; the error says which scale fits.
