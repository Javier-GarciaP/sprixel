import { useT } from "@/shared/lib/i18n";
import { MAX_ZOOM, MIN_ZOOM } from "../view/viewMath";

type Props = {
  // The pixel under the pointer, or null when the pointer is not on the sprite.
  position: { x: number; y: number } | null;
  zoom: number;
  onZoomIn: () => void;
  onZoomOut: () => void;
  onFit: () => void;
};

const button =
  "h-6 min-w-6 rounded border border-border bg-surface px-1.5 text-xs text-text disabled:cursor-not-allowed disabled:opacity-40";

// The bar under the canvas: zoom buttons and the zoom level.
export function CanvasStatusBar({
  position,
  zoom,
  onZoomIn,
  onZoomOut,
  onFit,
}: Props) {
  const t = useT();
  return (
    <div className="flex h-8 shrink-0 items-center justify-between gap-2 border-t border-border bg-bg px-3">
      <span
        className="font-mono text-xs text-text-muted"
        aria-label={t("canvas.coordinates")}
      >
        {t("canvas.position", {
          x: position ? position.x : "–",
          y: position ? position.y : "–",
        })}
      </span>
      <div className="flex items-center gap-2">
        <button
          type="button"
          className={button}
          aria-label={t("canvas.zoomOut")}
          disabled={zoom <= MIN_ZOOM}
          onClick={onZoomOut}
        >
          −
        </button>
        <span
          className="w-12 text-center font-mono text-xs text-text-muted"
          aria-label={t("canvas.zoom")}
        >
          {zoom * 100}%
        </span>
        <button
          type="button"
          className={button}
          aria-label={t("canvas.zoomIn")}
          disabled={zoom >= MAX_ZOOM}
          onClick={onZoomIn}
        >
          +
        </button>
        <button type="button" className={button} onClick={onFit}>
          {t("canvas.fit")}
        </button>
      </div>
    </div>
  );
}
