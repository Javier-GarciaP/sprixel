import type { View } from "../render/renderer";

export const MIN_ZOOM = 1;
export const MAX_ZOOM = 32;

export type Point = { x: number; y: number };

const clamp = (zoom: number) => Math.min(MAX_ZOOM, Math.max(MIN_ZOOM, zoom));

// Zoom changes by one whole step (100%) and stays between 100% and 3200%.
export function stepZoom(zoom: number, direction: 1 | -1): number {
  return clamp(zoom + direction);
}

// The largest whole zoom at which the sprite fits in the area.
export function fitZoom(
  width: number,
  height: number,
  areaWidth: number,
  areaHeight: number,
): number {
  return clamp(Math.floor(Math.min(areaWidth / width, areaHeight / height)));
}

// The sprite in the middle of the area, on whole screen pixels.
export function centeredView(
  width: number,
  height: number,
  zoom: number,
  areaWidth: number,
  areaHeight: number,
): View {
  return {
    zoom,
    x: Math.floor((areaWidth - width * zoom) / 2),
    y: Math.floor((areaHeight - height * zoom) / 2),
  };
}

// The sprite pixel under a screen point (it can be outside the sprite).
export function pixelAt(view: View, point: Point): Point {
  return {
    x: Math.floor((point.x - view.x) / view.zoom),
    y: Math.floor((point.y - view.y) / view.zoom),
  };
}

// Changes the zoom and moves the sprite so that the pixel under `pointer`
// stays under it. The new position is the closest whole number to the exact
// one, kept inside the range that leaves the same pixel under the pointer.
export function zoomAt(view: View, zoom: number, pointer: Point): View {
  const next = clamp(zoom);
  const place = (offset: number, at: number) => {
    const exact = (at - offset) / view.zoom;
    const pixel = Math.floor(exact);
    const ideal = Math.round(at - exact * next);
    const low = at - (pixel + 1) * next + 1;
    const high = at - pixel * next;
    return Math.min(high, Math.max(low, ideal));
  };
  return {
    zoom: next,
    x: place(view.x, pointer.x),
    y: place(view.y, pointer.y),
  };
}
