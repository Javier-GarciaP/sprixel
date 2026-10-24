import type { ToolId } from "../tools/tools";

// Icons of the editor, on a 24x24 grid with a thin stroke. They use
// currentColor, so the text color sets the icon color.
const paths: Record<string, string> = {
  pencil: "M4 20l1-4L16 5l3 3L8 19zM14 7l3 3",
  eraser: "M20 20H9L4 15a2 2 0 010-3l8-8a2 2 0 013 0l5 5a2 2 0 010 3z",
  bucket: "M6 12l6-6 7 7-6 6zM19 15c1 2 2 3 2 4a2 2 0 01-4 0c0-1 1-2 2-4z",
  eyedropper:
    "M20 4a2 2 0 00-3 0l-3 3-1-1-2 2 8 8 2-2-1-1 3-3a2 2 0 000-3zM11 10l-7 7v3h3l7-7",
  line: "M5 19L19 5",
  rectangle: "M5 6h14v12H5z",
  ellipse: "M12 5a8 7 0 100 14 8 7 0 000-14z",
  select: "M5 5h14v14H5z",
  move: "M12 3v18M3 12h18M12 3l-3 3M12 3l3 3M12 21l-3-3M12 21l3-3M3 12l3-3M3 12l3 3M21 12l-3-3M21 12l-3 3",
  undo: "M9 14L4 9l5-5M4 9h10a6 6 0 010 12h-3",
  redo: "M15 14l5-5-5-5M20 9H10a6 6 0 000 12h3",
  swap: "M7 4L3 8l4 4M3 8h14M17 20l4-4-4-4M21 16H7",
};

export type EditorIconName = ToolId | "undo" | "redo" | "swap";

export function EditorIcon({
  name,
  className = "h-5 w-5",
}: {
  name: EditorIconName;
  className?: string;
}) {
  return (
    <svg
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth={2}
      strokeLinecap="round"
      strokeLinejoin="round"
      strokeDasharray={name === "select" ? "3 3" : undefined}
      aria-hidden="true"
      className={className}
    >
      <path d={paths[name]} />
    </svg>
  );
}
