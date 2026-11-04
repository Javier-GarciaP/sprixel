import type { FramesApi, FramesInfo, Tag } from "@/features/frames-animations";
import type {
  ColorEdit,
  PaletteApi,
  PaletteInfo,
  Preset,
  Replacement,
} from "@/features/palette-editor";
import type { ChangedEvent, FrameData } from "@/shared/lib/core";
import type {
  Anchor,
  AnchorKey,
  Applied,
  Clip,
  OutlineArgs,
  PowerApi,
  Preview,
  RegionAction,
  Review,
  ShadeArgs,
  TileCheck,
  AgentActivity,
  AgentOps,
  ApplyResult,
  EditorApi,
  HistoryState,
  PixelChange,
  SpriteInfo,
  SpriteRef,
} from "../api/types";
import type { PaletteColor, Rect } from "../types";

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

export class FakeEditorApi
  implements EditorApi, FramesApi, PaletteApi, PowerApi
{
  frames: string[];
  palette: PaletteColor[];
  calls: { frame: number; changes: PixelChange[] }[] = [];
  private done: Entry[] = [];
  private undone: Entry[] = [];
  private listeners = new Set<(a: AgentActivity | null) => void>();
  activity: AgentActivity | null = null;
  stopped = 0;
  revision = 0;
  // The name of the palette of the sprite (a sprite can have its own).
  paletteName = "main";
  // The animations of the sprite (the core tests the range rules).
  tags: Tag[] = [];
  private sprite: SpriteRef = { folder: [], name: "goblin" };
  private changed = new Set<(event: ChangedEvent) => void>();

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
    this.sprite = sprite;
    return {
      width: this.width,
      height: this.height,
      frameCount: this.frames.length,
      paletteName: this.paletteName,
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
    this.notify(sprite);
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
      this.notify(this.sprite);
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
      this.notify(this.sprite);
    }
    return this.history();
  }

  async agentActivity() {
    return this.activity;
  }

  // The ops of agent writes and the numbers the editor said it finished.
  private opsListeners = new Set<(ops: AgentOps) => void>();
  playbackDones: number[] = [];

  async onAgentOps(callback: (ops: AgentOps) => void) {
    this.opsListeners.add(callback);
    return () => {
      this.opsListeners.delete(callback);
    };
  }

  get listeningToOps() {
    return this.opsListeners.size > 0;
  }

  // Simulates the event of an agent write.
  emitOps(ops: AgentOps) {
    this.latestWrite = ops.id;
    this.opsListeners.forEach((listener) => listener(ops));
  }

  // Like the core: the frame is free when the editor says the replay of the
  // newest write is done. `latestWrite` is the id of that write.
  latestWrite = 0;
  async playbackDone(id: number) {
    this.playbackDones.push(id);
    if (id === this.latestWrite && this.activity) this.emit(null);
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

  // The palette editor calls (see `PaletteApi`): the edits are only recorded.
  paletteEdits: {
    name: string;
    colors: ColorEdit[];
    replacements: Replacement[];
  }[] = [];

  async loadPalette(name: string): Promise<PaletteInfo> {
    return {
      name,
      colors: this.palette.map((c) => ({ ...c, name: c.key, used: false })),
    };
  }

  async applyEdit(
    name: string,
    colors: ColorEdit[],
    replacements: Replacement[],
  ): Promise<void> {
    this.paletteEdits.push({ name, colors, replacements });
  }

  // The editor does not make palettes; these only complete the interface.
  async listPresets(): Promise<Preset[]> {
    return [];
  }

  async createPalette(name: string): Promise<string> {
    return name;
  }

  async rampPreview(): Promise<string[]> {
    return [];
  }

  async rampApply(): Promise<{ keys: string[] }> {
    return { keys: [] };
  }

  async removeUnused(): Promise<string[]> {
    return [];
  }

  async pickPaletteFile(): Promise<string | null> {
    return null;
  }

  async importPalette(name: string): Promise<string> {
    return name;
  }

  // The frames and animations calls (see `FramesApi`).
  private notify(sprite: SpriteRef) {
    const event: ChangedEvent = {
      revision: this.revision,
      touched: [{ kind: "sprite", path: [...sprite.folder, sprite.name] }],
    };
    this.changed.forEach((listener) => listener(event));
  }

  // The own duration of each frame (100 means none of its own).
  durations: number[] = [];
  durationCalls: { frame: number; ms: number }[] = [];

  async setDuration(sprite: SpriteRef, frame: number, ms: number) {
    this.durationCalls.push({ frame, ms });
    this.durations[frame - 1] = ms;
    this.revision++;
    this.notify(sprite);
  }

  async loadInfo(sprite: SpriteRef): Promise<FramesInfo> {
    void sprite;
    return {
      frameCount: this.frames.length,
      durations: this.frames.map((_, i) => this.durations[i] ?? 100),
      defaultMs: 100,
      tags: this.tags.map((t) => ({ ...t })),
    };
  }

  async loadFrameData(sprite: SpriteRef, frame: number): Promise<FrameData> {
    void sprite;
    return {
      revision: this.revision,
      width: this.width,
      height: this.height,
      rows: this.rows(frame),
      colors: this.palette.map((c) => ({ ...c })),
    };
  }

  async insertFrame(sprite: SpriteRef, at: number, copyOf: number | null) {
    const blank = ".".repeat(this.width * this.height);
    this.frames.splice(at - 1, 0, copyOf ? this.frames[copyOf - 1] : blank);
    this.revision++;
    this.notify(sprite);
    return at;
  }

  async deleteFrame(sprite: SpriteRef, frame: number) {
    if (this.frames.length === 1) throw { code: "last_frame" };
    this.frames.splice(frame - 1, 1);
    this.revision++;
    this.notify(sprite);
  }

  async moveFrame(sprite: SpriteRef, from: number, to: number) {
    const [moved] = this.frames.splice(from - 1, 1);
    this.frames.splice(to - 1, 0, moved);
    this.revision++;
    this.notify(sprite);
  }

  async addTag(sprite: SpriteRef, tag: Tag) {
    this.tags.push({ ...tag });
    this.notify(sprite);
  }

  async setTag(sprite: SpriteRef, name: string, tag: Tag) {
    this.tags = this.tags.map((t) => (t.name === name ? { ...tag } : t));
    this.notify(sprite);
  }

  async removeTag(sprite: SpriteRef, name: string) {
    this.tags = this.tags.filter((t) => t.name !== name);
    this.notify(sprite);
  }

  async onChanged(listener: (event: ChangedEvent) => void) {
    this.changed.add(listener);
    return () => {
      this.changed.delete(listener);
    };
  }

  // The power tools (see `PowerApi`). The calls are recorded; the tests of
  // the core check what they do. `powerFailure` makes the next write fail.
  powerCalls: { name: string; args: unknown[] }[] = [];
  powerFailure: { code: string; message?: string } | null = null;
  clip: Clip = { palette: "main", width: 1, height: 1, keys: "." };
  preview: Preview = { pixels: [] };
  review_: Review = { used: [], unused: [], frames: [], changes: [] };
  tile: TileCheck = { seamless: true, mismatches: [] };
  anchorList: Anchor[] = [];

  private async power<T>(name: string, args: unknown[], value: T): Promise<T> {
    this.powerCalls.push({ name, args });
    const failure = this.powerFailure;
    if (failure && !name.endsWith("Preview")) {
      this.powerFailure = null;
      throw failure;
    }
    return value;
  }

  private applied(): Applied {
    this.revision++;
    return { pixelsChanged: 1, ...this.history(), canUndo: true };
  }

  region(
    sprite: SpriteRef,
    frame: number,
    region: Rect | null,
    action: RegionAction,
  ) {
    return this.power(
      "region",
      [sprite, frame, region, action],
      this.applied(),
    );
  }
  copy(sprite: SpriteRef, frame: number, region: Rect | null) {
    return this.power("copy", [sprite, frame, region], this.clip);
  }
  paste(
    sprite: SpriteRef,
    frame: number,
    clip: Clip,
    at: { x: number; y: number },
    skipTransparent: boolean,
  ) {
    return this.power(
      "paste",
      [sprite, frame, clip, at, skipTransparent],
      this.applied(),
    );
  }
  outlinePreview(sprite: SpriteRef, frame: number, args: OutlineArgs) {
    return this.power("outlinePreview", [sprite, frame, args], this.preview);
  }
  outlineApply(sprite: SpriteRef, frame: number, args: OutlineArgs) {
    return this.power("outlineApply", [sprite, frame, args], this.applied());
  }
  shadePreview(sprite: SpriteRef, frame: number, args: ShadeArgs) {
    return this.power("shadePreview", [sprite, frame, args], this.preview);
  }
  shadeApply(sprite: SpriteRef, frame: number, args: ShadeArgs) {
    return this.power("shadeApply", [sprite, frame, args], this.applied());
  }
  review(sprite: SpriteRef, outlineKey: string | null) {
    return this.power("review", [sprite, outlineKey], this.review_);
  }
  tileCheck(sprite: SpriteRef, frame: number) {
    return this.power("tileCheck", [sprite, frame], this.tile);
  }
  anchors(sprite: SpriteRef) {
    // A read made on every change: not recorded with the actions.
    void sprite;
    return Promise.resolve(this.anchorList.map((a) => ({ ...a })));
  }
  async saveAnchor(
    sprite: SpriteRef,
    anchor: Anchor,
    replaces: AnchorKey | null,
  ) {
    await this.power("saveAnchor", [sprite, anchor, replaces], undefined);
    const same = (a: Anchor, k: AnchorKey) =>
      a.name.toLowerCase() === k.name.toLowerCase() && a.frame === k.frame;
    const old = replaces ?? anchor;
    this.anchorList = this.anchorList.filter((a) => !same(a, old));
    this.anchorList.push({ ...anchor });
    this.revision++;
  }
  async deleteAnchor(sprite: SpriteRef, key: AnchorKey) {
    await this.power("deleteAnchor", [sprite, key], undefined);
    this.anchorList = this.anchorList.filter(
      (a) =>
        !(
          a.name.toLowerCase() === key.name.toLowerCase() &&
          a.frame === key.frame
        ),
    );
    this.revision++;
  }
}
