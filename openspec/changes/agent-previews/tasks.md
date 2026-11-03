# Tasks

## 1. Backgrounds

- [x] 1.1 Add `render/preview.rs` with `Background` and `flatten`, and verify checker, light, dark and transparent with unit tests
- [ ] 1.2 Add `background` to `render_preview` and to the image of `apply_grid`, `set_pixels` and `draw_shape`, and verify the default, the options and `bad_argument`

## 2. Frames and onion

- [ ] 2.1 Add the 3x5 digit font and `render_sheet`, and verify the layout size and the labels with unit tests
- [ ] 2.2 Add the tool `render_frames` and verify range, columns, scale, `too_big` and `bad_frame`
- [ ] 2.3 Add `render_onion` (tinted, translucent earlier frames) and verify the colors and the errors
- [ ] 2.4 Name the files of the new images in `sprixel tool` and verify that two images never share a file

## 3. Docs

- [ ] 3.1 Update the guide, the skill, `docs/ai-and-mcp.md` and `CHANGELOG.md`, and run `openspec validate --all --strict`
