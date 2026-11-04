import { Tooltip } from "@/shared/components";

type Props = {
  label: string;
  shortcut?: string;
  onClick: () => void;
  // The 24x24 path of the icon (drawn with a thin stroke).
  path: string;
};

// An icon button of the frames column. It looks like the buttons of the top
// bar of the editor (same size, hover and focus ring) and has the same
// tooltip. The bubble opens upwards because the buttons sit at the bottom.
export function IconButton({ label, shortcut, onClick, path }: Props) {
  return (
    <Tooltip label={label} shortcut={shortcut} side="top" align="start">
      <button
        type="button"
        aria-label={label}
        onClick={onClick}
        className="flex h-8 w-8 items-center justify-center rounded-md border border-border bg-surface text-text-muted outline-none hover:bg-surface-2 hover:text-text focus-visible:ring-2 focus-visible:ring-amber"
      >
        <svg
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          strokeWidth={2}
          strokeLinecap="round"
          strokeLinejoin="round"
          aria-hidden="true"
          className="h-4 w-4"
        >
          <path d={path} />
        </svg>
      </button>
    </Tooltip>
  );
}
