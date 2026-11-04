import type { View } from "../render/renderer";

type Props = {
  view: View;
  anchors: { name: string; x: number; y: number }[];
};

// The names of the anchors, over the canvas next to their crosses. The
// panel lists the same anchors for the keyboard and screen readers, so this
// layer is only a picture and lets every click through.
export function AnchorLabels({ view, anchors }: Props) {
  return (
    <div
      aria-hidden="true"
      className="pointer-events-none absolute inset-0 z-10"
    >
      {anchors.map(({ name, x, y }) => (
        <span
          key={name}
          data-testid="anchor-label"
          className="absolute whitespace-nowrap rounded-sm bg-black/70 px-1 text-[10px] leading-4 text-white"
          style={{
            left: view.x + (x + 0.5) * view.zoom + 6,
            top: view.y + (y + 0.5) * view.zoom + 6,
          }}
        >
          {name}
        </span>
      ))}
    </div>
  );
}
