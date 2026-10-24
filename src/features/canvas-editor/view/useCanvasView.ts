import { useCallback, useRef, useState } from "react";
import type { View } from "../render/renderer";
import {
  centeredView,
  fitZoom,
  stepZoom,
  zoomAt,
  type Point,
} from "./viewMath";

export type Area = { width: number; height: number };

// The zoom and position of the sprite on the screen. Until the user zooms or
// moves the view, the sprite stays fitted and centered, also when the window
// changes size. "Fit" goes back to that.
export function useCanvasView(spriteWidth: number, spriteHeight: number) {
  const [area, setArea] = useState<Area>({ width: 0, height: 0 });
  const [manual, setManual] = useState<View | null>(null);

  const view: View =
    manual ??
    (area.width > 0
      ? centeredView(
          spriteWidth,
          spriteHeight,
          fitZoom(spriteWidth, spriteHeight, area.width, area.height),
          area.width,
          area.height,
        )
      : { zoom: 1, x: 0, y: 0 });
  // The latest view and area, for handlers that run later than the render.
  const latest = useRef({ view, area });
  latest.current = { view, area };

  const center = (): Point => ({
    x: latest.current.area.width / 2,
    y: latest.current.area.height / 2,
  });

  // `pointer` is a point of the canvas area; the buttons and keys use its center.
  const zoomBy = useCallback((direction: 1 | -1, pointer?: Point) => {
    const { view: current, area: size } = latest.current;
    const at = pointer ?? { x: size.width / 2, y: size.height / 2 };
    setManual(zoomAt(current, stepZoom(current.zoom, direction), at));
  }, []);

  return {
    view,
    area,
    setArea,
    zoomIn: useCallback((pointer?: Point) => zoomBy(1, pointer), [zoomBy]),
    zoomOut: useCallback((pointer?: Point) => zoomBy(-1, pointer), [zoomBy]),
    fit: useCallback(() => setManual(null), []),
    moveTo: useCallback(
      (x: number, y: number) =>
        setManual({
          ...latest.current.view,
          x: Math.round(x),
          y: Math.round(y),
        }),
      [],
    ),
    center,
  };
}
