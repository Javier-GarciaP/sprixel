// Small icons for the Project tab, on the same 24x24 grid and thin stroke as
// the shared icons. They live here, not in `shared/`, because only this
// feature uses them.
const paths = {
  folder: "M3 7h6l2 2h10v10H3z",
  sprite: "M4 5h16v14H4zM4 15l5-5 4 4 3-3 4 4",
  palette:
    "M12 3a9 9 0 100 18c1.5 0 2-1 1.5-2-.6-1.2.2-2.5 1.5-2.5H17a4 4 0 004-4c0-5-4-9.5-9-9.5zM7.5 12h.01M9.5 8h.01M14 7.5h.01",
  plus: "M12 5v14M5 12h14",
  chevron: "M9 6l6 6-6 6",
  more: "M5 12h.01M12 12h.01M19 12h.01",
  trash: "M4 7h16M9 7V4h6v3M6 7l1 13h10l1-13M10 11v6M14 11v6",
  copy: "M8 8h12v12H8zM4 16V4h12",
  search: "M11 4a7 7 0 100 14 7 7 0 000-14zM21 21l-5-5",
  check: "M5 12l5 5 9-10",
};

export type BrowserIconName = keyof typeof paths;

export function BrowserIcon({
  name,
  className = "h-4 w-4",
}: {
  name: BrowserIconName;
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
      aria-hidden="true"
      className={className}
    >
      <path d={paths[name]} />
    </svg>
  );
}
