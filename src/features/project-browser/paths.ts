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

export type FolderOption = { path: string[]; label: string };

// Every folder as a choice for a "Save in" list: the root first (with the
// name of the project), then the folders from top to bottom.
export function folderOptions(tree: ProjectTree): FolderOption[] {
  const out: FolderOption[] = [{ path: [], label: tree.name }];
  const walk = (folder: FolderNode) => {
    for (const child of folder.folders) {
      out.push({ path: child.path, label: child.path.join(" / ") });
      walk(child);
    }
  };
  walk(tree.root);
  return out;
}

// Swaps the start of a path: a folder was renamed from `from` to `to`, so
// everything inside it has a new path. A path outside the folder stays.
export function followRename(
  path: string[],
  from: string[],
  to: string[],
): string[] {
  const inside = from.every((name, i) => path[i] === name);
  return inside && path.length >= from.length
    ? [...to, ...path.slice(from.length)]
    : path;
}

// The kind of data a dragged sprite card carries (a list of sprites as JSON).
export const DRAG_TYPE = "application/x-sprixel-sprites";
