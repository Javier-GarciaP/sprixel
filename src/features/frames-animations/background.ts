import type { CSSProperties } from "react";

// What the preview shows behind a sprite with transparent pixels: the gray
// checkerboard (the same in both themes), or a flat light or dark color to
// see how the sprite looks on a bright or a dark ground in the game.
export type PreviewBackground = "checker" | "light" | "dark";

export const previewBackgrounds: PreviewBackground[] = [
  "checker",
  "light",
  "dark",
];

export function nextBackground(now: PreviewBackground): PreviewBackground {
  return previewBackgrounds[
    (previewBackgrounds.indexOf(now) + 1) % previewBackgrounds.length
  ];
}

export function backgroundStyle(background: PreviewBackground): CSSProperties {
  if (background === "light") return { backgroundColor: "#F4F4F5" };
  if (background === "dark") return { backgroundColor: "#16161A" };
  return {
    backgroundColor: "#3a3a42",
    backgroundImage:
      "conic-gradient(#2e2e35 25%, transparent 25% 50%, #2e2e35 50% 75%, transparent 75%)",
    backgroundSize: "12px 12px",
  };
}
