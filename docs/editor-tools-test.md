# Editor power tools: manual test

A checklist for the maintainer, on Linux. These tools were built and tested
with fake cores and with Rust tests, but **nothing was run in the real app
window yet**. Please run this page once before a release and write down what
fails.

## Before you start

1. `npm ci`, then `npm run tauri dev`.
2. Create a project. Create a sprite `knight` of 16x16 and a second sprite
   `slime` of 16x16 (so you can try paste between sprites).
3. Open `knight` in the **Editor** tab. Draw a small shape with two colors:
   for example a 6x6 body, a head and one single pixel far from them.
4. Every action below must be **one step of Undo** (Ctrl+Z). After each one,
   press Undo and check the sprite is as before, then Redo (Ctrl+Shift+Z).

Shortcuts are listed in the Settings page (Keyboard shortcuts). The keys used
here are the defaults.

## 1. Selection actions

Press **M** (Select) and drag a rectangle around the shape.

| # | Do this | Expected |
| --- | --- | --- |
| 1.1 | Look at the top bar before and after you select | The bar keeps the same height and the same icons. The flip, rotate and mirror icons are grey (disabled) before, and turn on after. Hover a grey icon: the tip says "Select an area first". In a narrow window they sit in the **More** (⋯) menu |
| 1.2 | Click **Flip horizontally** (or press **H**) | The selected pixels are mirrored left to right. Pixels outside the rectangle do not change. The selection stays |
| 1.3 | Click **Flip vertically** (or **Shift+H**) | Mirrored top to bottom |
| 1.4 | Click **Rotate 90° right** on a square selection (or **Shift+R**) | The shape turns clockwise |
| 1.5 | Click **Rotate 90° left** (**Shift+L**) and **Rotate 180°** (**Shift+U**) | Turns left; turns upside down |
| 1.6 | Select a rectangle that is not square and click **Rotate 90° right** | A message says it cannot turn 90°. Nothing changes |
| 1.7 | Click **Mirror the left half onto the right** (**Shift+M**) | The right half becomes the mirror of the left half. Try the three other mirror buttons too (top/bottom has **Shift+N**) |
| 1.8 | Press the **arrow keys** | The pixels inside the selection move 1 pixel. What leaves the rectangle is lost |
| 1.9 | Turn on **Wrap around when moving** (the loop icon, or press **W**) and press an arrow key | What leaves one side comes back on the other side |
| 1.10 | Click once on the canvas with Select to clear the selection, then press an arrow key | The whole frame moves |

## 2. Copy and paste

| # | Do this | Expected |
| --- | --- | --- |
| 2.1 | Select the shape and press **Ctrl+C** | A message says "Copied 6 × 6" |
| 2.2 | Go to frame 2 (**.** key) and press **Ctrl+V** | The shape appears at the top left of the selection (or of the frame). It is selected, so you can move it |
| 2.3 | Open `slime` (same palette) and press **Ctrl+V** | The shape is pasted there. Transparent pixels do not erase what is under them |
| 2.4 | In the Text view, select some text and press Ctrl+C / Ctrl+V | The text is copied by the text box, not by the editor |
| 2.5 | Press Ctrl+V before copying anything (new session) | A message says nothing is copied yet |
| 2.6 | Give `slime` another palette (Project tab, palette of the sprite) and paste | A message says the sprite uses another palette. Nothing changes |

## 3. Symmetry

| # | Do this | Expected |
| --- | --- | --- |
| 3.1 | In the top bar click the **Symmetry: horizontal** icon (or press **S**) | A dashed pink vertical line shows in the middle of the canvas |
| 3.2 | Draw with the Pencil on the left side | The same stroke appears mirrored on the right side |
| 3.3 | Press **Ctrl+Z** once | Both sides are undone together |
| 3.4 | Try **Vertical** and **Both** | A horizontal line, and both lines. Strokes mirror in 2 and 4 places |
| 3.5 | With a symmetry mode on, try Line, Rectangle, Ellipse and Bucket | Each one is mirrored. One Undo per shape |
| 3.6 | Choose **Off** | The lines go away. Drawing is normal |

## 4. Outline and Shade

| # | Do this | Expected |
| --- | --- | --- |
| 4.1 | Click the **Outline** icon (or **Shift+O**) | A panel opens on the right. The canvas shows a see-through preview of the outline. The sprite is not changed yet |
| 4.2 | Change the color, 4 or 8 neighbors, "Skip holes", "Draw inside the shape" | The preview changes each time. The text says how many pixels will change |
| 4.3 | Press **Cancel** or **Escape** | The panel closes, the preview goes away, the sprite did not change |
| 4.4 | Open it again and press **Apply** | The outline is drawn. The panel closes. One Undo removes it |
| 4.5 | Click the **Shade** icon (**Shift+S**). Pick a base color and 2 to 4 colors for the ramp, and a light direction | The preview shows lighter pixels toward the light and darker pixels away. Apply is off until the base and a ramp are chosen |
| 4.6 | Select a rectangle first, open Shade and check "Only inside the selection" | Only that rectangle is shaded |
| 4.7 | Apply, then Undo | The sprite is as before |

## 5. Review panel

| # | Do this | Expected |
| --- | --- | --- |
| 5.1 | Click the **Review sprite** icon (**Shift+A**) | The panel lists "Loose pixels" (the single pixel you drew apart), "Unused colors" and, with 2+ frames, how each frame differs from the next |
| 5.2 | Click the loose pixel in the list | The editor goes to its frame and a black and white box marks the pixel on the canvas |
| 5.3 | Pick an outline color in the panel | "Outline gaps" lists shape pixels open to the outside, if any |
| 5.4 | Fix the sprite | The panel updates by itself. When all is fine it says "Nothing wrong found." |

## 6. Palette editor

Open the palette editor (click the palette, or the pencil in the palette panel).

| # | Do this | Expected |
| --- | --- | --- |
| 6.1 | In **Generate ramp** type a base color, steps 5, hue shift 20 | A strip previews the tones, dark to light |
| 6.2 | Click **Add ramp** | The new tones are in the list of colors. One Undo removes them all |
| 6.3 | Add the same ramp again | No tone is duplicated (existing tones are reused) |
| 6.4 | Edit a color name without pressing Done | Both tools show "Press Done first" and are off |
| 6.5 | Click **Remove unused colors...** | The dialog lists the colors no sprite uses. Nothing is removed yet |
| 6.6 | Click Cancel, then ask again and confirm | The colors are removed. One Undo brings them back |

## 7. Frames: durations and anchors

| # | Do this | Expected |
| --- | --- | --- |
| 7.1 | Make an animation with 3 frames at 8 fps | Each frame box under the thumbnails is empty and shows 125 (the animation speed) |
| 7.2 | Type 300 in the box of frame 2 and press Enter | Frame 2 shows 300; the others still show 125. The preview plays frame 2 longer |
| 7.3 | Empty the box of frame 2 and press Tab | It goes back to the animation speed |
| 7.4 | Type 5 | The box goes back (the limit is 10 to 10000) |
| 7.5 | Press **A** (Anchor tool) | The Anchors panel opens |
| 7.6 | Click an empty pixel, type "feet" and Save | An amber cross with the name "feet" shows on that pixel |
| 7.7 | Drag the cross to another pixel | It moves when you release. One Undo moves it back |
| 7.8 | Select it in the list, change the name to "boots" and Save | The cross shows the new name. One Undo |
| 7.9 | Use "New anchor", type X and Y in the boxes and Save | Works without the mouse |
| 7.10 | Check "Only on this frame" when creating one, then change frame | It only shows on that frame |
| 7.11 | Select an anchor and click Delete | The cross is gone. Undo brings it back |

## 8. Preview tools

| # | Do this | Expected |
| --- | --- | --- |
| 8.1 | Under the preview click **Light**, then **Dark** (or **Shift+B**) | The preview background is a flat light color, then a flat dark color. The main canvas stays the gray checkerboard |
| 8.2 | Turn on **Tile preview (3 × 3)** (or **T**) | The frame is repeated 3 times on each side |
| 8.3 | Draw a pixel in the first column and not in the last | A warning says how many edge pixels do not match |
| 8.4 | Make the first and last columns/rows equal | The warning says "Seamless" |
| 8.5 | Go to frame 4 with the onion skin on and choose **3** frames (or **Shift+F**) | The pixels of frames 3, 2 and 1 show in blue, older ones fainter |
| 8.6 | Go to frame 2 with 3 frames chosen | Only frame 1 shows |

## 9. Agents and busy frames

| # | Do this | Expected |
| --- | --- | --- |
| 9.1 | Ask an agent to draw on the frame you have open, and press a selection button while it draws | A message says an agent is drawing. Nothing changes |
| 9.2 | Open the Outline panel while the agent draws and press Apply | The panel shows the same message |

## Notes

Write here what failed (number, what you saw):
