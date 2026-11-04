// A small project used by the tests of this feature.
import type {
  FolderNode,
  FrameData,
  ProjectTree,
  SpriteSummary,
} from "@/shared/lib/core";

export function sprite(
  folder: string[],
  name: string,
  extra: Partial<SpriteSummary> = {},
): SpriteSummary {
  return {
    name,
    folder,
    width: 8,
    height: 8,
    frames: 4,
    palette: "dungeon16",
    tags: [],
    lastEditor: null,
    ...extra,
  };
}

export function folder(
  parent: string[],
  name: string,
  sprites: SpriteSummary[] = [],
  folders: FolderNode[] = [],
): FolderNode {
  return { name, path: [...parent, name], sprites, folders };
}

// Personajes: goblin (8x8, 4 frames, a "walk" animation), hero, merchant.
// Enemigos: slime. Root: orc. The palette is dungeon16.
export function sampleTree(revision = 1): ProjectTree {
  return {
    name: "Dungeon Kit",
    revision,
    root: {
      name: "",
      path: [],
      folders: [
        folder(
          [],
          "Personajes",
          [
            sprite(["Personajes"], "goblin", {
              tags: [{ name: "walk", from: 1, to: 4, fps: 12, mode: "loop" }],
            }),
            sprite(["Personajes"], "hero", { frames: 3 }),
            sprite(["Personajes"], "merchant", { frames: 2 }),
          ],
          [],
        ),
        folder([], "Enemigos", [
          sprite(["Enemigos"], "slime", { width: 16, height: 16, frames: 2 }),
        ]),
      ],
      sprites: [sprite([], "orc", { frames: 1 })],
    },
    palettes: [{ name: "dungeon16", colors: 16 }],
  };
}

export function emptyTree(): ProjectTree {
  return {
    name: "Empty",
    revision: 0,
    root: { name: "", path: [], folders: [], sprites: [] },
    palettes: [],
  };
}

export const frameOf = (revision = 1): FrameData => ({
  revision,
  width: 2,
  height: 2,
  rows: ["R.", ".R"],
  colors: [{ key: "R", hex: "#F04545" }],
});
