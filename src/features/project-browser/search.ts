import type { FolderNode, ProjectTree, SpriteSummary } from "@/shared/lib/core";
import { allSprites } from "./paths";

// Lower case and without accents, so "Árbol", "arbol" and "ARBOL" are the
// same word. (A letter with an accent is the letter plus a combining mark,
// and the marks are removed.)
export function normalize(text: string): string {
  return text.normalize("NFD").replace(/\p{M}/gu, "").toLowerCase();
}

export type SearchResults = {
  folders: FolderNode[];
  sprites: SpriteSummary[];
};

function allFolders(folder: FolderNode): FolderNode[] {
  return folder.folders.flatMap((child) => [child, ...allFolders(child)]);
}

// The sprites and folders of the whole project whose name contains the text.
// An empty text finds nothing: the caller shows the normal tree then.
export function search(tree: ProjectTree, text: string): SearchResults {
  const wanted = normalize(text.trim());
  if (!wanted) return { folders: [], sprites: [] };
  const matches = (name: string) => normalize(name).includes(wanted);
  return {
    folders: allFolders(tree.root).filter((f) => matches(f.name)),
    sprites: allSprites(tree.root).filter((s) => matches(s.name)),
  };
}
