import { useEffect, useRef } from "react";
import type { Scene, View } from "../render/renderer";
import { Surface } from "../render/surface";

type Props = {
  scene: Scene;
  view: View;
  // Called when the area changes size.
  onSize?: (width: number, height: number) => void;
};

// The pixel canvas. It fills its container and paints the scene with the
// view it is given. The canvas background stays transparent outside the
// sprite, so the theme color of the area shows around it.
export function PixelCanvas({ scene, view, onSize }: Props) {
  const holder = useRef<HTMLDivElement>(null);
  const canvas = useRef<HTMLCanvasElement>(null);
  const surface = useRef<Surface | null>(null);
  const latest = useRef({ scene, view });
  latest.current = { scene, view };
  const report = useRef(onSize);
  report.current = onSize;

  useEffect(() => {
    const element = canvas.current;
    const box = holder.current;
    if (!element || !box) return;
    const target = new Surface(element);
    surface.current = target;
    const fit = () => {
      target.resize(box.clientWidth, box.clientHeight);
      report.current?.(box.clientWidth, box.clientHeight);
      target.draw(latest.current.scene, latest.current.view);
    };
    fit();
    const observer =
      typeof ResizeObserver === "function" ? new ResizeObserver(fit) : null;
    observer?.observe(box);
    return () => {
      observer?.disconnect();
      surface.current = null;
    };
  }, []);

  useEffect(() => {
    surface.current?.draw(scene, view);
  }, [scene, view]);

  return (
    <div ref={holder} className="relative h-full w-full overflow-hidden">
      <canvas
        ref={canvas}
        className="absolute inset-0 block [image-rendering:pixelated]"
      />
    </div>
  );
}
