# Demo: the chicken test

A short, repeatable check of the whole experience with a real AI agent: the
agent follows the guide, draws a sprite, looks at its work and fixes it, and the
person watches a cursor with the agent name draw the pixels.

Run it with Claude Code. Keep the notes in the last section.

## Before you start

1. Build the command and the app: `npm run build:cli` and `npm run tauri dev`
   (or install a release build).
2. Put `bin/sprixel` on your `PATH`, then run `sprixel mcp add claude`. This
   registers the server and writes the Sprixel skill (`~/.claude/skills/sprixel/SKILL.md`).
3. In the app, create a project (Project tab, **New project**) and open the
   **Agents** tab.
4. Start Claude Code in any folder and ask it to draw (step 2 below). The first
   call asks for approval: press **Allow** in the Agents tab.

## The test

1. Open the **Editor** tab so the empty editor is on screen.
2. Give the agent exactly this prompt:

   > Draw a 16x16 chicken in the Sprixel project that is open. Make an idle
   > frame and a frame where it pecks the ground, and make the two frames an
   > animation called peck at 4 fps. Look at your result and fix anything that
   > looks wrong before you tell me you are done.

3. Watch the app while the agent works.

## What you must see

| # | Expected result |
| --- | --- |
| 1 | The agent reads the guide first (`get_guide` with `design` in its tool calls) |
| 2 | The editor shows the new sprite by itself (it follows the agent) |
| 3 | A cursor with the agent name (for example "Claude Code") moves over the canvas and the pixels appear one after another, not all at once |
| 4 | While the pixels appear, the frame is marked as busy and the Stop button is shown; if you draw on it, your stroke waits and is applied after |
| 5 | The agent looks at the image that each write returns (you see "image" results in its output) and fixes at least one thing, or says why nothing needs a fix |
| 6 | The chicken has a small palette (about 4 to 8 colors), one clear light direction, a clean outline and no stray pixels |
| 7 | The two frames differ only where the chicken moves (head and beak), and the animation `peck` plays in the preview panel |
| 8 | When the replay ends the canvas is exactly what the Text view shows (open the Text view to compare) |
| 9 | Pressing **Stop** during a replay shows the final state at once and frees the frame |
| 10 | With "reduce motion" on in your system, the final state appears with no cursor movement |

If any line fails, write what happened in the notes. Do not change the code in
the same run.

## Result recorded

Not run yet. The maintainer runs the test with Claude Code and fills this in.

- [ ] Run with Claude Code
- Date and versions (app, `sprixel version`, Claude Code):
- Lines that passed:
- Lines that failed, and what was seen:
- The image the agent finished with (attach it or write its path):
