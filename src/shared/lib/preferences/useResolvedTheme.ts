import { useEffect, useState } from "react";
import { usePreferences } from ".";
import { resolveTheme } from "./theme";

const query = "(prefers-color-scheme: dark)";

// The theme that is really shown: with System it follows the operating
// system and changes when the system changes.
export function useResolvedTheme(): "light" | "dark" {
  const { preferences } = usePreferences();
  const [systemDark, setSystemDark] = useState(
    () => window.matchMedia?.(query).matches ?? true,
  );

  useEffect(() => {
    const media = window.matchMedia?.(query);
    if (!media) return;
    const onChange = () => setSystemDark(media.matches);
    media.addEventListener("change", onChange);
    return () => media.removeEventListener("change", onChange);
  }, []);

  return resolveTheme(preferences.theme, systemDark);
}
