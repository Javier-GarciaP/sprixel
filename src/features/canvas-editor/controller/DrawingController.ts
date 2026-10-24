import type { PixelChange } from "../api/types";
import { Draft } from "../tools/draft";
import { floodFill } from "../tools/fill";
import {
  ellipsePoints,
  linePoints,
  rectanglePoints,
  snapLine,
  squareCorner,
} from "../tools/shapes";
import type { ToolId } from "../tools/tools";
import { TRANSPARENT_CODE, type Rect } from "../types";
import type { Point } from "../view/viewMath";
import type { ColorBook } from "./colorBook";

function rectBetween(a: Point, b: Point): Rect {
  return {
    x: Math.min(a.x, b.x),
    y: Math.min(a.y, b.y),
    width: Math.abs(a.x - b.x) + 1,
    height: Math.abs(a.y - b.y) + 1,
  };
}

export type Events = {
  // An area of sprite pixels changed and must be repainted.
  onDirty: (area: Rect) => void;
  // A stroke, fill, shape or move ended: its changed pixels, to send to the
  // core as one operation.
  onCommit: (changes: PixelChange[]) => void;
  // The Eyedropper picked a color: the main one (left button) or the
  // secondary one (right button).
  onPick: (hex: string, button: Button) => void;
};

export type Modifiers = { shift: boolean };

// 0 is the left button and 2 the right one.
export type Button = 0 | 2;

// Handles the pointer on the pixels of the current frame. It lives outside
// React: it writes straight into the typed array and asks for a repaint of
// the changed area only, so drawing never waits for a React render. A stroke
// ends with one list of changed pixels.
export class DrawingController {
  tool: ToolId = "pencil";
  main = "#000000";
  secondary = "#FFFFFF";
  private draft: Draft;
  // Rectangle and Ellipse can be outlined or filled.
  filled = false;
  private stroke: { code: number; last: Point } | null = null;
  // A shape being dragged: where it started and the color it is drawn with.
  private shape: { from: Point; code: number } | null = null;
  // The rectangle chosen with Select, in sprite pixels.
  selection: Rect | null = null;
  private selecting: Point | null = null;
  // A move in progress: where the pointer started, the area that moves and a
  // copy of its pixels.
  private moving: { from: Point; area: Rect; source: Uint8Array } | null = null;

  constructor(
    readonly width: number,
    readonly height: number,
    readonly pixels: Uint8Array,
    readonly colors: ColorBook,
    private events: Events,
  ) {
    this.draft = new Draft(pixels, width, height);
  }

  get drawing() {
    return (
      this.stroke !== null ||
      this.shape !== null ||
      this.selecting !== null ||
      this.moving !== null
    );
  }

  pointerDown(x: number, y: number, button: Button, modifiers: Modifiers) {
    const color = button === 2 ? this.secondary : this.main;
    if (this.tool === "eyedropper") {
      if (this.draft.inside(x, y)) {
        // A transparent pixel has no color: the colors stay as they are.
        const hex = this.colors.hexOf(this.pixels[y * this.width + x]);
        if (hex) this.events.onPick(hex, button);
      }
      return;
    }
    if (this.tool === "select") {
      // A new area starts here; a click without a drag clears the old one.
      this.selecting = { x, y };
      this.setSelection(null);
      return;
    }
    if (this.tool === "move") {
      this.startMove(x, y);
      return;
    }
    if (this.tool === "bucket") {
      // A fill is one click: it starts and ends here.
      floodFill(this.draft, this.pixels, x, y, this.colors.codeFor(color));
      this.flush();
      this.finish();
      return;
    }
    if (
      this.tool === "line" ||
      this.tool === "rectangle" ||
      this.tool === "ellipse"
    ) {
      this.shape = { from: { x, y }, code: this.colors.codeFor(color) };
      this.drawShape({ x, y }, modifiers.shift);
      return;
    }
    if (this.tool === "pencil" || this.tool === "eraser") {
      const code =
        this.tool === "eraser" ? TRANSPARENT_CODE : this.colors.codeFor(color);
      this.stroke = { code, last: { x, y } };
      this.draft.set(x, y, code);
      this.flush();
    }
  }

  pointerMove(x: number, y: number, modifiers: Modifiers) {
    if (this.selecting) {
      const from = this.selecting;
      if (x !== from.x || y !== from.y) {
        this.setSelection(this.clip(rectBetween(from, { x, y })));
      }
      return;
    }
    if (this.moving) {
      this.moveBy(x - this.moving.from.x, y - this.moving.from.y);
      return;
    }
    if (this.shape) {
      this.drawShape({ x, y }, modifiers.shift);
      return;
    }
    const stroke = this.stroke;
    if (!stroke) return;
    // Join the last position and this one, so a fast move leaves no gaps.
    for (const point of linePoints(stroke.last, { x, y })) {
      this.draft.set(point.x, point.y, stroke.code);
    }
    stroke.last = { x, y };
    this.flush();
  }

  pointerUp() {
    this.selecting = null;
    const wasMoving = this.moving !== null;
    this.moving = null;
    if (!this.stroke && !this.shape && !wasMoving) return;
    this.stroke = null;
    this.shape = null;
    this.finish();
  }

  // Changes the selection and asks to repaint the old and the new area.
  private setSelection(next: Rect | null) {
    const old = this.selection;
    this.selection = next;
    for (const area of [old, next]) if (area) this.events.onDirty(area);
  }

  // The part of a rectangle that is inside the sprite, or null.
  private clip(area: Rect): Rect | null {
    const x0 = Math.max(0, area.x);
    const y0 = Math.max(0, area.y);
    const x1 = Math.min(this.width, area.x + area.width);
    const y1 = Math.min(this.height, area.y + area.height);
    return x1 > x0 && y1 > y0
      ? { x: x0, y: y0, width: x1 - x0, height: y1 - y0 }
      : null;
  }

  // Picks up the selected pixels, or the whole frame when nothing is
  // selected.
  private startMove(x: number, y: number) {
    const area = this.selection ?? {
      x: 0,
      y: 0,
      width: this.width,
      height: this.height,
    };
    const source = new Uint8Array(area.width * area.height);
    for (let j = 0; j < area.height; j++) {
      for (let i = 0; i < area.width; i++) {
        source[j * area.width + i] =
          this.pixels[(area.y + j) * this.width + area.x + i];
      }
    }
    this.moving = { from: { x, y }, area, source };
  }

  // Shows the picked-up pixels `dx` and `dy` away from where they were. The
  // old place becomes transparent and what leaves the sprite is cut off. The
  // colors are not changed. Transparent pixels of the moved piece do not
  // erase what is under them.
  private moveBy(dx: number, dy: number) {
    const { area, source } = this.moving!;
    this.draft.revert();
    for (let j = 0; j < area.height; j++) {
      for (let i = 0; i < area.width; i++) {
        this.draft.set(area.x + i, area.y + j, TRANSPARENT_CODE);
      }
    }
    for (let j = 0; j < area.height; j++) {
      for (let i = 0; i < area.width; i++) {
        const code = source[j * area.width + i];
        if (code !== TRANSPARENT_CODE) {
          this.draft.set(area.x + dx + i, area.y + dy + j, code);
        }
      }
    }
    this.flush();
    if (this.selection) {
      this.setSelection(this.clip({ ...area, x: area.x + dx, y: area.y + dy }));
    }
  }

  // The live preview of a shape: the pixels of the last position are taken
  // back and the shape is drawn again up to the pointer. Nothing is sent
  // until the pointer is released.
  private drawShape(to: Point, shift: boolean) {
    const shape = this.shape!;
    this.draft.revert();
    const from = shape.from;
    let points;
    if (this.tool === "line") {
      points = linePoints(from, shift ? snapLine(from, to) : to);
    } else {
      const corner = shift ? squareCorner(from, to) : to;
      points =
        this.tool === "rectangle"
          ? rectanglePoints(from, corner, this.filled)
          : ellipsePoints(from, corner, this.filled);
    }
    for (const point of points) this.draft.set(point.x, point.y, shape.code);
    this.flush();
  }

  private flush() {
    const area = this.draft.takeDirty();
    if (area) this.events.onDirty(area);
  }

  // Sends what changed as one action.
  private finish() {
    const changes = this.draft.changes();
    this.draft.close();
    if (changes.length === 0) return;
    this.events.onCommit(
      changes.map(({ x, y, code }) => ({
        x,
        y,
        color: this.colors.hexOf(code),
      })),
    );
  }
}
