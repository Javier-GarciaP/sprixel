export type Platform = "mac" | "other";

export function detectPlatform(): Platform {
  return /mac/i.test(navigator.platform) ? "mac" : "other";
}

// Shortcuts are written once with "Mod" (Control, or Command on macOS):
// "Mod+N", "Mod+Shift+Z", "F2", "B".
export function formatShortcut(shortcut: string, platform: Platform): string {
  const names =
    platform === "mac"
      ? { Mod: "⌘", Shift: "⇧", Alt: "⌥" }
      : { Mod: "Ctrl", Shift: "Shift", Alt: "Alt" };
  return shortcut
    .split("+")
    .map((part) => names[part as keyof typeof names] ?? part)
    .join(platform === "mac" ? "" : "+");
}
