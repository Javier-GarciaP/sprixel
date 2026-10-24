import type { Rect } from "../types";

// A set of pixel changes that can be undone locally. Tools write through it:
// it remembers the first value of every pixel it touches, so a preview can be
// reverted and drawn again, and the final list of changes is exactly the
// pixels whose value differs from the start. It also tracks the area that
// needs to be repainted.
export class Draft {
  private original: Uint8Array;
  private touched: number[] = [];
  private seen: Uint8Array;
  private dirty: Rect | null = null;

  constructor(
    private pixels: Uint8Array,
    readonly width: number,
    readonly height: number,
  ) {
    this.original = new Uint8Array(pixels.length);
    this.seen = new Uint8Array(pixels.length);
  }

  inside(x: number, y: number) {
    return x >= 0 && y >= 0 && x < this.width && y < this.height;
  }

  // Sets a pixel. Pixels outside the sprite are ignored (clipping).
  set(x: number, y: number, code: number) {
    if (!this.inside(x, y)) return;
    const at = y * this.width + x;
    if (this.pixels[at] === code) return;
    if (!this.seen[at]) {
      this.seen[at] = 1;
      this.original[at] = this.pixels[at];
      this.touched.push(at);
    }
    this.pixels[at] = code;
    this.mark(x, y);
  }

  // Puts every touched pixel back to its first value. The draft stays open.
  revert() {
    for (const at of this.touched) {
      this.pixels[at] = this.original[at];
      this.mark(at % this.width, Math.floor(at / this.width));
    }
  }

  // The pixels that now differ from their first value.
  changes(): { x: number; y: number; code: number }[] {
    const list = [];
    for (const at of this.touched) {
      if (this.pixels[at] !== this.original[at]) {
        list.push({
          x: at % this.width,
          y: Math.floor(at / this.width),
          code: this.pixels[at],
        });
      }
    }
    return list;
  }

  // Forgets the first values: what is on the canvas now is the new start.
  close() {
    for (const at of this.touched) this.seen[at] = 0;
    this.touched = [];
  }

  // The area changed since the last call, or null.
  takeDirty(): Rect | null {
    const area = this.dirty;
    this.dirty = null;
    return area;
  }

  private mark(x: number, y: number) {
    const d = this.dirty;
    if (!d) {
      this.dirty = { x, y, width: 1, height: 1 };
      return;
    }
    const right = Math.max(d.x + d.width, x + 1);
    const bottom = Math.max(d.y + d.height, y + 1);
    d.x = Math.min(d.x, x);
    d.y = Math.min(d.y, y);
    d.width = right - d.x;
    d.height = bottom - d.y;
  }
}
