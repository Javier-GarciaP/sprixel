import {
  useEffect,
  useRef,
  useState,
  type PointerEvent,
  type Ref,
} from "react";
import type { Scene, View } from "../render/renderer";
import { pixelAt, type Point } from "../view/viewMath";
import type { Button } from "../controller/DrawingController";
import { PixelCanvas, type PixelCanvasHandle } from "./PixelCanvas";

type Props = {
  scene: Scene;
  view: View;
  onSize: (width: number, height: number) => void;
  // Zoom one step (Control and the wheel), keeping `at` fixed.
  onZoom: (direction: 1 | -1, at: Point) => void;
  // Move the view so the sprite starts at this screen position.
  onMove: (x: number, y: number) => void;
  // The sprite pixel under the pointer, or null when it is outside the sprite.
  onHover?: (pixel: Point | null) => void;
  // The pointer drew: the pixel can be outside the sprite (a stroke may leave
  // it and come back). The tool decides what to do.
  onStrokeStart?: (pixel: Point, button: Button, shift: boolean) => void;
  onStrokeMove?: (pixel: Point, shift: boolean) => void;
  onStrokeEnd?: () => void;
  // Lets the drawing controller repaint small areas.
  canvasRef?: Ref<PixelCanvasHandle>;
};

const typingInField = (target: EventTarget | null) =>
  !!(target as HTMLElement | null)?.closest?.(
    "input, textarea, select, [contenteditable]",
  );

// The area around the canvas. It reads the pointer: the wheel with Control
// zooms, and holding Space and dragging, or dragging with the middle button,
// moves the view.
export function CanvasArea({
  scene,
  view,
  onSize,
  onZoom,
  onMove,
  onHover,
  onStrokeStart,
  onStrokeMove,
  onStrokeEnd,
  canvasRef,
}: Props) {
  const holder = useRef<HTMLDivElement>(null);
  const [spaceHeld, setSpaceHeld] = useState(false);
  const [panning, setPanning] = useState(false);
  const drag = useRef<{ from: Point; start: View } | null>(null);
  const stroking = useRef(false);

  useEffect(() => {
    const down = (event: KeyboardEvent) => {
      if (event.key !== " " || typingInField(event.target)) return;
      event.preventDefault();
      setSpaceHeld(true);
    };
    const up = (event: KeyboardEvent) => {
      if (event.key === " ") setSpaceHeld(false);
    };
    window.addEventListener("keydown", down);
    window.addEventListener("keyup", up);
    return () => {
      window.removeEventListener("keydown", down);
      window.removeEventListener("keyup", up);
    };
  }, []);

  // A point of the area, in screen pixels.
  const local = (event: { clientX: number; clientY: number }): Point => {
    const box = holder.current?.getBoundingClientRect();
    return {
      x: event.clientX - (box?.left ?? 0),
      y: event.clientY - (box?.top ?? 0),
    };
  };

  // A native listener, not onWheel: React's wheel handler is passive, so it
  // cannot stop the webview from zooming the whole page. The handler is kept
  // in a ref so the listener is added once and always uses the latest props.
  const zoomWithWheel = useRef((event: globalThis.WheelEvent) => {
    void event;
  });
  zoomWithWheel.current = (event) => {
    if (!event.ctrlKey && !event.metaKey) return;
    event.preventDefault();
    onZoom(event.deltaY < 0 ? 1 : -1, local(event));
  };
  useEffect(() => {
    const element = holder.current;
    if (!element) return;
    const listener = (event: globalThis.WheelEvent) =>
      zoomWithWheel.current(event);
    element.addEventListener("wheel", listener, { passive: false });
    return () => element.removeEventListener("wheel", listener);
  }, []);

  const onPointerDown = (event: PointerEvent) => {
    const pan = event.button === 1 || (event.button === 0 && spaceHeld);
    const draw = event.button === 0 || event.button === 2;
    if (!pan && !draw) return;
    event.preventDefault();
    holder.current?.setPointerCapture?.(event.pointerId);
    if (pan) {
      drag.current = { from: local(event), start: view };
      setPanning(true);
      return;
    }
    stroking.current = true;
    onStrokeStart?.(
      pixelAt(view, local(event)),
      event.button as Button,
      event.shiftKey,
    );
  };

  // Tells the parent which pixel is under the pointer, only when it changes.
  const hovered = useRef<Point | null>(null);
  const hover = (pixel: Point | null) => {
    const before = hovered.current;
    if (before?.x === pixel?.x && before?.y === pixel?.y) return;
    hovered.current = pixel;
    onHover?.(pixel);
  };

  const onPointerMove = (event: PointerEvent) => {
    const pixel = pixelAt(view, local(event));
    const inside =
      pixel.x >= 0 &&
      pixel.y >= 0 &&
      pixel.x < scene.width &&
      pixel.y < scene.height;
    hover(inside ? pixel : null);
    if (stroking.current) onStrokeMove?.(pixel, event.shiftKey);
    if (!drag.current) return;
    const now = local(event);
    onMove(
      drag.current.start.x + now.x - drag.current.from.x,
      drag.current.start.y + now.y - drag.current.from.y,
    );
  };

  const release = () => {
    drag.current = null;
    setPanning(false);
    if (stroking.current) {
      stroking.current = false;
      onStrokeEnd?.();
    }
  };

  const cursor = panning ? "grabbing" : spaceHeld ? "grab" : "crosshair";
  return (
    <div
      ref={holder}
      data-testid="canvas-area"
      className="min-h-0 min-w-0 flex-1"
      style={{ cursor }}
      onPointerDown={onPointerDown}
      onPointerMove={onPointerMove}
      onPointerUp={release}
      onPointerCancel={release}
      onPointerLeave={() => hover(null)}
      onAuxClick={(event) => event.preventDefault()}
      onContextMenu={(event) => event.preventDefault()}
    >
      <PixelCanvas ref={canvasRef} scene={scene} view={view} onSize={onSize} />
    </div>
  );
}
