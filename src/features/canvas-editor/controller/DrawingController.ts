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
    return this.stroke !== null || this.shape !== null;
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
    if (!this.stroke && !this.shape) return;
    this.stroke = null;
    this.shape = null;
    this.finish();
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
