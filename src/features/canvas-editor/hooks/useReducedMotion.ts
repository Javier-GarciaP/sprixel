import { useEffect, useState } from "react";

const QUERY = "(prefers-reduced-motion: reduce)";

// True when the system asks for less motion. The app has no setting of its own
// for it: it follows the one of the operating system.
export function useReducedMotion(): boolean {
  const read = () =>
    typeof window !== "undefined" &&
    typeof window.matchMedia === "function" &&
    window.matchMedia(QUERY).matches;
  const [reduced, setReduced] = useState(read);
  useEffect(() => {
    if (typeof window.matchMedia !== "function") return;
    const media = window.matchMedia(QUERY);
    const change = () => setReduced(media.matches);
    change();
    media.addEventListener?.("change", change);
    return () => media.removeEventListener?.("change", change);
  }, []);
  return reduced;
}
