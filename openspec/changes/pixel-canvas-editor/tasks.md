# Tasks

> **Demo cut.** Every task is in the demo, including onion skin and Select and Move. Task 3.6 adds the color model: any color can be used and the palette grows by itself (see the `drawing-tools` and `palette-management` specs). See `docs/roadmap.md`.

## 1. Canvas rendering

- [x] 1.1 Build the canvas renderer with exact scaling and the neutral checkerboard and verify screenshots in both themes
- [x] 1.2 Add zoom (keys, wheel, buttons, Fit) and panning with the pointer kept fixed, and verify with tests
- [x] 1.3 Add the pixel grid option and the coordinates display and verify the 600% rule

## 2. Editor layout

- [x] 2.1 Build the editor layout (tools, frames column placeholder, canvas bar, right panel) and verify against screens 18 and 19
- [x] 2.2 Show the size next to the name, the active tool and the undo and redo buttons, and verify the bar contents

## 3. Tools

- [x] 3.1 Build the drawing controller, the Pencil and the Eraser with gap-free strokes and verify the fast stroke test
- [x] 3.2 Build the colors (main, secondary, swap, right button) and the Eyedropper and verify the scenarios
- [x] 3.3 Build the Bucket and verify the closed outline test and the 512x512 timing
- [x] 3.4 Build Line, Rectangle and Ellipse with preview, Shift and fill options, and verify the Shift scenarios
- [ ] 3.5 Build Select and Move with clipping and verify the move test
- [ ] 3.6 Implement the growing palette in the core and use it for any color the user picks (next free key, palette `main` created when there is none, limit of 93 colors, one undo step with the stroke), and verify the new color, the full palette and the undo scenarios

## 4. History and onion skin

- [ ] 4.1 Send each stroke as one operation and wire undo and redo, and verify a 30-pixel stroke undoes in one step
- [ ] 4.2 Build onion skin with the switch and the opacity slider and verify frame 1 and frame 2 behavior

## 5. Agents and speed

- [ ] 5.1 Show the agent marker, the busy frame and the Stop button from core events, and verify with a simulated agent
- [ ] 5.2 Run the 512x512 speed benchmark with the real editor and verify the 16 ms budget on each system

## 6. Documentation

- [ ] 6.1 Update `docs/usage.md` (tools, shortcuts, onion skin) and verify it matches the app
