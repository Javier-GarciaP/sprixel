import type { View } from "../render/renderer";

// Where the cursor box goes on the screen, in pixels of the canvas area: the
// box covers the sprite pixel `(x, y)` exactly, so it moves with the zoom and
// the offset of the view.
export function cursorBox(view: View, x: number, y: number) {
  return {
    left: view.x + x * view.zoom,
    top: view.y + y * view.zoom,
    size: view.zoom,
  };
}

type Props = {
  view: View;
  x: number;
  y: number;
  agent: string;
};

// The cursor of an agent while its pixels are replayed: an amber box on the
// pixel it is drawing, with its name above. It lets the clicks through.
export function AgentCursor({ view, x, y, agent }: Props) {
  const box = cursorBox(view, x, y);
  return (
    <div
      data-testid="agent-cursor"
      aria-hidden="true"
      className="pointer-events-none absolute z-10 border-2 border-amber"
      style={{
        left: box.left,
        top: box.top,
        width: box.size,
        height: box.size,
      }}
    >
      <span className="absolute bottom-full left-0 mb-0.5 whitespace-nowrap rounded-sm bg-amber px-1 text-[10px] font-bold leading-4 text-ink">
        {agent}
      </span>
    </div>
  );
}
