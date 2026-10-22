import type { Theme } from ".";

// "system" follows the operating system; the other two are fixed.
export function resolveTheme(
  theme: Theme,
  systemPrefersDark: boolean,
): "light" | "dark" {
  if (theme === "system") return systemPrefersDark ? "dark" : "light";
  return theme;
}
