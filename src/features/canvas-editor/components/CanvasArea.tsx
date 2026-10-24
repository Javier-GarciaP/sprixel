import {
  useEffect,
  useRef,
  useState,
  type PointerEvent,
  type WheelEvent,
} from "react";
import type { Scene, View } from "../render/renderer";
import type { Point } from "../view/viewMath";
import { PixelCanvas } from "./PixelCanvas";

type Props = {
  scene: Scene;
  view: View;
  onSize: (width: number, height: number) => void;
  // Zoom one step (Control and the wheel), keeping `at` fixed.
  onZoom: (direction: 1 | -1, at: Point) => void;
  // Move the view so the sprite starts at this screen position.
  onMove: (x: number, y: number) => void;
};

const typingInField = (target: EventTarget | null) =>
  !!(target as HTMLElement | null)?.closest?.(
    "input, textarea, select, [contenteditable]",
  );

// The area around the canvas. It reads the pointer: the wheel with Control
// zooms, and holding Space and dragging, or dragging with the middle button,
// moves the view.
export function CanvasArea({ scene, view, onSize, onZoom, onMove }: Props) {
  const holder = useRef<HTMLDivElement>(null);
  const [spaceHeld, setSpaceHeld] = useState(false);
  const [panning, setPanning] = useState(false);
  const drag = useRef<{ from: Point; start: View } | null>(null);

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

  const onWheel = (event: WheelEvent) => {
    if (!event.ctrlKey && !event.metaKey) return;
    event.preventDefault();
    onZoom(event.deltaY < 0 ? 1 : -1, local(event));
  };

  const onPointerDown = (event: PointerEvent) => {
    const pan = event.button === 1 || (event.button === 0 && spaceHeld);
    if (!pan) return;
    event.preventDefault();
    drag.current = { from: local(event), start: view };
    setPanning(true);
    holder.current?.setPointerCapture?.(event.pointerId);
  };

  const onPointerMove = (event: PointerEvent) => {
    if (!drag.current) return;
    const now = local(event);
    onMove(
      drag.current.start.x + now.x - drag.current.from.x,
      drag.current.start.y + now.y - drag.current.from.y,
    );
  };

  const stopPan = () => {
    drag.current = null;
    setPanning(false);
  };

  const cursor = panning ? "grabbing" : spaceHeld ? "grab" : "crosshair";
  return (
    <div
      ref={holder}
      data-testid="canvas-area"
      className="min-h-0 min-w-0 flex-1"
      style={{ cursor }}
      onWheel={onWheel}
      onPointerDown={onPointerDown}
      onPointerMove={onPointerMove}
      onPointerUp={stopPan}
      onPointerCancel={stopPan}
      onAuxClick={(event) => event.preventDefault()}
    >
      <PixelCanvas scene={scene} view={view} onSize={onSize} />
    </div>
  );
}
