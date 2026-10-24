import type { FolderNode, ProjectTree, SpriteSummary } from "@/shared/lib/core";

// A folder or a sprite is found by the names from the root. This makes one
// string of that path to use as a key (names cannot contain "/").
export const pathKey = (path: string[]) => path.join("/");

export const spriteKey = (sprite: Pick<SpriteSummary, "folder" | "name">) =>
  pathKey([...sprite.folder, sprite.name]);

const same = (a: string, b: string) => a.toLowerCase() === b.toLowerCase();

// The folder at `path`, or null if it does not exist (it was deleted or renamed).
export function findFolder(
  tree: ProjectTree,
  path: string[],
): FolderNode | null {
  let folder = tree.root;
  for (const name of path) {
    const next = folder.folders.find((f) => same(f.name, name));
    if (!next) return null;
    folder = next;
  }
  return folder;
}

// Sprites and folders inside a folder, directly.
export const itemCount = (folder: FolderNode) =>
  folder.folders.length + folder.sprites.length;

export function allSprites(folder: FolderNode): SpriteSummary[] {
  return [...folder.sprites, ...folder.folders.flatMap(allSprites)];
}
