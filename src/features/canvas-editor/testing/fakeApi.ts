import type {
  AgentActivity,
  ApplyResult,
  EditorApi,
  HistoryState,
  PixelChange,
  SpriteInfo,
  SpriteRef,
} from "../api/types";
import type { PaletteColor } from "../types";

// An in-memory stand-in for the Rust core, for the tests. It follows the same
// rules the core has for strokes: one history entry per call, a palette that
// grows with the next free key, `palette_full` at 93 colors and `frame_busy`
// while an agent writes. The real rules are tested in Rust.
const KEY_ORDER =
  "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789" +
  "!\"#$%&'()*+,-/:;<=>?@[\\]^_`{|}~";

type Entry = {
  frame: number;
  before: string;
  after: string;
  added: PaletteColor[];
};

export class FakeEditorApi implements EditorApi {
  frames: string[];
  palette: PaletteColor[];
  calls: { frame: number; changes: PixelChange[] }[] = [];
  private done: Entry[] = [];
  private undone: Entry[] = [];
  private listeners = new Set<(a: AgentActivity | null) => void>();
  activity: AgentActivity | null = null;
  stopped = 0;
  revision = 0;

  constructor(
    public width: number,
    public height: number,
    frameCount = 1,
    palette: PaletteColor[] = [],
  ) {
    this.frames = Array.from({ length: frameCount }, () =>
      ".".repeat(width * height),
    );
    this.palette = [...palette];
  }

  // Sets the pixels of a frame from rows of keys, for example ["R.", ".R"].
  setFrame(frame: number, rows: string[]) {
    this.frames[frame - 1] = rows.join("");
  }

  rows(frame: number): string[] {
    const text = this.frames[frame - 1];
    return Array.from({ length: this.height }, (_, y) =>
      text.slice(y * this.width, (y + 1) * this.width),
    );
  }

  private history(): HistoryState {
    return {
      canUndo: this.done.length > 0,
      canRedo: this.undone.length > 0,
      revision: this.revision,
    };
  }

  async loadSprite(sprite: SpriteRef): Promise<SpriteInfo> {
    void sprite;
    return {
      width: this.width,
      height: this.height,
      frameCount: this.frames.length,
      palette: this.palette.map((c) => ({ ...c })),
      ...this.history(),
    };
  }

  async loadFrame(sprite: SpriteRef, frame: number): Promise<string> {
    void sprite;
    return this.frames[frame - 1];
  }

  async applyChanges(
    sprite: SpriteRef,
    frame: number,
    changes: PixelChange[],
  ): Promise<ApplyResult> {
    if (
      this.activity &&
      this.activity.frame === frame &&
      this.activity.sprite.name === sprite.name
    ) {
      throw { code: "frame_busy" };
    }
    this.calls.push({ frame, changes });
    const palette = this.palette.map((c) => ({ ...c }));
    const added: PaletteColor[] = [];
    const keyOf = (hex: string | null) => {
      if (hex === null) return ".";
      const upper = hex.toUpperCase();
      const found = palette.find((c) => c.hex === upper);
      if (found) return found.key;
      if (palette.length >= 93) throw { code: "palette_full" };
      const key = [...KEY_ORDER].find(
        (k) => !palette.some((c) => c.key === k),
      )!;
      palette.push({ key, hex: upper });
      added.push({ key, hex: upper });
      return key;
    };
    const chars = [...this.frames[frame - 1]];
    for (const change of changes) {
      chars[change.y * this.width + change.x] = keyOf(change.color);
    }
    const entry = {
      frame,
      before: this.frames[frame - 1],
      after: chars.join(""),
      added,
    };
    this.frames[frame - 1] = entry.after;
    this.palette = palette;
    this.done.push(entry);
    this.undone = [];
    this.revision++;
    return { palette: palette.map((c) => ({ ...c })), ...this.history() };
  }

  async undo(): Promise<HistoryState> {
    const entry = this.done.pop();
    if (entry) {
      this.frames[entry.frame - 1] = entry.before;
      const keys = entry.added.map((c) => c.key);
      this.palette = this.palette.filter((c) => !keys.includes(c.key));
      this.undone.push(entry);
      this.revision++;
    }
    return this.history();
  }

  async redo(): Promise<HistoryState> {
    const entry = this.undone.pop();
    if (entry) {
      this.frames[entry.frame - 1] = entry.after;
      this.palette = [...this.palette, ...entry.added];
      this.done.push(entry);
      this.revision++;
    }
    return this.history();
  }

  async agentActivity() {
    return this.activity;
  }

  async stopAgent() {
    this.stopped++;
    this.emit(null);
  }

  async onAgentActivity(callback: (activity: AgentActivity | null) => void) {
    this.listeners.add(callback);
    return () => {
      this.listeners.delete(callback);
    };
  }

  // Simulates an agent that starts (or, with null, ends) a write.
  emit(activity: AgentActivity | null) {
    this.activity = activity;
    this.listeners.forEach((listener) => listener(activity));
  }
}
