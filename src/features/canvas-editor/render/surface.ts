import type { Rect } from "../types";
import { renderScene, type Scene, type View } from "./renderer";

// The screen area (in screen pixels) covered by an area of sprite pixels.
export function screenRect(area: Rect, view: View): Rect {
  return {
    x: view.x + area.x * view.zoom,
    y: view.y + area.y * view.zoom,
    width: area.width * view.zoom,
    height: area.height * view.zoom,
  };
}

// Owns one <canvas> and its RGBA buffer. It paints with `renderScene` and
// sends only the changed area to the screen with `putImageData`. It lives
// outside React state: React only creates it and gives it a size.
export class Surface {
  private ctx: CanvasRenderingContext2D | null;
  private buffer = new Uint8ClampedArray(0);
  private image: ImageData | null = null;
  width = 0;
  height = 0;

  constructor(private canvas: HTMLCanvasElement) {
    this.ctx = canvas.getContext("2d");
  }

  resize(width: number, height: number) {
    this.width = width;
    this.height = height;
    this.canvas.width = width;
    this.canvas.height = height;
    this.buffer = new Uint8ClampedArray(width * height * 4);
    this.image =
      typeof ImageData === "function" && width > 0 && height > 0
        ? new ImageData(this.buffer, width, height)
        : null;
  }

  // Paints the whole scene, or only `dirty` (an area of sprite pixels).
  draw(scene: Scene, view: View, dirty?: Rect) {
    if (!this.width || !this.height) return;
    renderScene(this.buffer, this.width, this.height, scene, view, dirty);
    if (!this.ctx || !this.image) return;
    if (!dirty) {
      this.ctx.putImageData(this.image, 0, 0);
      return;
    }
    const area = screenRect(dirty, view);
    const x = Math.max(0, area.x);
    const y = Math.max(0, area.y);
    const right = Math.min(this.width, area.x + area.width);
    const bottom = Math.min(this.height, area.y + area.height);
    if (right > x && bottom > y) {
      this.ctx.putImageData(this.image, 0, 0, x, y, right - x, bottom - y);
    }
  }
}
