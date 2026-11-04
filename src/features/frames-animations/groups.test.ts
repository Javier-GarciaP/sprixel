import { describe, expect, it } from "vitest";
import { groupsOf } from "./groups";
import type { Tag } from "./types";

const tag = (name: string, from: number, to: number): Tag => ({
  name,
  from,
  to,
  fps: 12,
  mode: "loop",
});

const shape = (tags: Tag[], count: number) =>
  groupsOf(tags, count).groups.map((g) => [g.id, g.frames]);

describe("groupsOf", () => {
  // No animation at all: every frame is the base, nothing else is listed.
  it("puts every frame in Base when there are no animations", () => {
    expect(shape([], 3)).toEqual([["base", [1, 2, 3]]]);
  });

  // Two animations after one base frame: Base first, then each animation in
  // order of its first frame, and no empty "No animation" group.
  it("lists Base, then the animations", () => {
    expect(shape([tag("jump", 4, 5), tag("walk", 2, 3)], 5)).toEqual([
      ["base", [1]],
      ["tag:walk", [2, 3]],
      ["tag:jump", [4, 5]],
    ]);
  });

  // Frames after the animations that no tag owns go last. An animation that
  // starts at frame 1 leaves Base out.
  it("puts the frames of no animation last and hides an empty Base", () => {
    expect(shape([tag("walk", 1, 2), tag("jump", 4, 4)], 6)).toEqual([
      ["tag:walk", [1, 2]],
      ["tag:jump", [4]],
      ["none", [3, 5, 6]],
    ]);
  });

  // Old files may have overlapping tags: the frame is shown once, in the first
  // group, and the result says so.
  it("shows a shared frame once and reports the overlap", () => {
    const result = groupsOf([tag("a", 1, 3), tag("b", 3, 4)], 4);
    expect(result.overlaps).toBe(true);
    expect(result.groups.map((g) => [g.id, g.frames])).toEqual([
      ["tag:a", [1, 2, 3]],
      ["tag:b", [4]],
    ]);
  });

  // A tag that points past the last frame is cut to the frames that exist.
  it("ignores frames that do not exist", () => {
    expect(shape([tag("a", 2, 9)], 3)).toEqual([
      ["base", [1]],
      ["tag:a", [2, 3]],
    ]);
  });
});
