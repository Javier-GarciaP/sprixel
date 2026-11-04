import type { FolderNode, ProjectTree, SpriteTarget } from "@/shared/lib/core";
import type { Scope, ScopeKind } from "./types";

const same = (a: string, b: string) => a.toLowerCase() === b.toLowerCase();
const key = (path: string[]) => path.join("/");

const sprites = (folder: FolderNode): FolderNode["sprites"] => [
  ...folder.sprites,
  ...folder.folders.flatMap(sprites),
];

export const countProject = (tree: ProjectTree) => sprites(tree.root).length;

// The folder at `path`, or null when it does not exist (it was deleted).
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

// Every folder with its sprites counted (sub-folders included).
export function folderChoices(tree: ProjectTree) {
  const out: { path: string[]; label: string; count: number }[] = [];
  const walk = (folder: FolderNode) => {
    for (const child of folder.folders) {
      out.push({
        path: child.path,
        label: child.path.join(" / "),
        count: sprites(child).length,
      });
      walk(child);
    }
  };
  walk(tree.root);
  return out;
}

// The chosen sprites that still exist in the project.
export function existing(tree: ProjectTree, chosen: SpriteTarget[]) {
  const all = sprites(tree.root);
  return chosen.filter((c) =>
    all.some(
      (s) =>
        same(s.name, c.name) &&
        key(s.folder).toLowerCase() === key(c.folder).toLowerCase(),
    ),
  );
}

// What is exported when the person has not chosen yet: the selection of the
// Project tab, else its open folder, else everything.
export function defaultKind(
  hasSelection: boolean,
  hasFolder: boolean,
): ScopeKind {
  return hasSelection ? "sprites" : hasFolder ? "folder" : "project";
}

export function toScope(
  kind: ScopeKind,
  folder: string[],
  selection: SpriteTarget[],
): Scope {
  if (kind === "folder") return { kind, path: folder };
  if (kind === "sprites") return { kind, sprites: selection };
  return { kind };
}
