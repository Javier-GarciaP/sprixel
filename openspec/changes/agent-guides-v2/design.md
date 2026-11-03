# Design

## Context

`get_guide` serves constants of `docs.rs` by topic. A test (`guide_tests.rs`)
checks that every name between backticks in the guides, the workflow, the
instructions and the skill is a real tool, argument or error code, and that the
skill and the server carry the same workflow and example.

## Goals / Non-Goals

**Goals:**
- Guides an agent can follow step by step, with exact tool names and arguments.
- Examples that are proven by tests, so a guide cannot drift from the tools.

**Non-Goals:**
- New tools or new arguments.
- Long art theory. Each guide is short and practical.

## Decisions

- **Texts stay in `docs.rs`** as `pub const` strings, like the other guides.
  `GUIDE_TOPICS` lists all eight topics, in this order: `design`, `animation`,
  `palette`, `large-sprites`, `faces-and-animals`, `walk-and-attack-cycles`,
  `game-ready`, `tools`. The tool schema and the `unknown_topic` error use it.
- **Examples share constants with the tests.** A grid used in a guide is a
  `macro_rules!` (like `example_grid!`) and a `pub const`, so the guide text
  and the test use the same characters. The test runs the calls in the order
  of the guide and checks the result with exact values.
- **The examples** (all tiny, so a grid is easy to read and check):
  - `large-sprites`: a 16x16 mushroom drawn region by region. The left half of
    the cap and of the stem is written with `apply_grid` (`x`, `y`) and
    `symmetry` `horizontal`, then `auto_outline`, `auto_shade` and
    `analyze_sprite` close the loop. The text says the same steps work for 64x64.
  - `faces-and-animals`: a 16x16 cat head. Half of the head with `symmetry`,
    the eyes with `set_pixels` and `symmetry`, then `auto_outline` and
    `analyze_sprite` with `outline_key`.
  - `walk-and-attack-cycles`: a 12x10 walker with 4 walk frames (contact,
    passing, contact, passing) with `set_frame_duration` for the contact frames
    and `create_tag`, then an attack of 3 frames made with `add_frame`
    (`copy_of`) and `transform_region` (`move`). `analyze_sprite` shows
    `ground_kept`.
  - `game-ready`: the walker gets an `set_anchor` for its feet, a tag and
    durations; an 8x8 grass tile is made and checked with `check_tile` and
    `render_tiled`; `export` writes the sheet.
  - `tools`: a short repair loop on a gem with a gap in its outline:
    `analyze_sprite` finds the gap, `set_pixels` fixes it, `analyze_sprite`
    confirms.
- **The `tools` guide is a map**: one line per job ("read", "draw", "edit",
  "finish", "check", "colors", "animate", "game data", "export"), each naming
  the tools. A test checks that every tool of the server is named in the
  `tools` guide, so a new tool cannot be forgotten.
- **One list line everywhere.** `GUIDE_LIST` is one sentence that lists the
  topics. The server instructions carry it, and the skill and
  `docs/ai-and-mcp.md` must contain the same sentence (a test checks both).
  The guide texts that are long stay out of the instructions (they are in every
  prompt) and out of the skill; the skill already has `design`.
- **Walk cycle fact.** `ground_kept` is true when the bottom rows did not
  change. In a walk the feet move, so `false` is normal there; the guide says
  to check that the lowest row of the feet is the same in every frame (the
  `bbox` of `changes`). `ground_kept` true is right for an idle.

## Risks / Trade-offs

- Long guides cost tokens when an agent reads them. Each topic is separate, so
  an agent reads only what it needs, and each guide is under about 25 lines.
- The examples are small on purpose. The guide says that big sprites use the
  same steps, only more regions.
