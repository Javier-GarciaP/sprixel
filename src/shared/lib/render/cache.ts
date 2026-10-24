import type { ChangedEvent, FrameData } from "@/shared/lib/core";

type Loader = (
  folder: string[],
  name: string,
  frame: number,
) => Promise<FrameData>;

const keyOf = (folder: string[], name: string, frame: number) =>
  JSON.stringify([...folder, name, frame]);

// Keeps the frames that were asked for, so a card does not ask the core again
// each time it is drawn. A change of the project tells the cache which
// sprites were touched (see `invalidate`); those are asked again the next
// time. The cache keeps the promise, so two cards that ask for the same
// frame at once make one call.
export class FrameCache {
  private frames = new Map<string, Promise<FrameData>>();

  constructor(private load: Loader) {}

  get(folder: string[], name: string, frame = 1): Promise<FrameData> {
    const key = keyOf(folder, name, frame);
    let found = this.frames.get(key);
    if (!found) {
      found = this.load(folder, name, frame);
      // A failed call is not kept, so the next ask tries again.
      found.catch(() => this.frames.delete(key));
      this.frames.set(key, found);
    }
    return found;
  }

  // Forgets the frames of the sprites a change touched. Touching a folder
  // forgets every sprite inside it. Returns true if something was forgotten.
  invalidate(touched: ChangedEvent["touched"]): boolean {
    let forgotten = false;
    for (const key of [...this.frames.keys()]) {
      const path: string[] = JSON.parse(key);
      const sprite = path.slice(0, -1);
      const hit = touched.some(({ kind, path: part }) =>
        kind === "sprite"
          ? sameList(part, sprite)
          : kind === "folder" && startsWith(sprite, part),
      );
      if (hit) {
        this.frames.delete(key);
        forgotten = true;
      }
    }
    return forgotten;
  }

  clear() {
    this.frames.clear();
  }
}

const sameList = (a: string[], b: string[]) =>
  a.length === b.length && a.every((item, i) => item === b[i]);

// Is `list` inside `prefix`? (The sprite path has the folders and the name.)
const startsWith = (list: string[], prefix: string[]) =>
  list.length > prefix.length && prefix.every((item, i) => item === list[i]);
