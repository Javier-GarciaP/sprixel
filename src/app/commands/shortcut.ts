import { t, type TranslationKey } from "@/shared/lib/i18n";

export type Platform = "mac" | "other";

export function detectPlatform(): Platform {
  return /mac/i.test(navigator.platform) ? "mac" : "other";
}

// Shortcuts are written once with "Mod" (Control, or Command on macOS):
// "Mod+N", "Mod+Shift+Z", "F2", "B". The "+" key itself is written "+".
function parts(shortcut: string): string[] {
  return shortcut === "+" ? ["+"] : shortcut.split("+");
}

// Key names that have a translation. Ctrl, the macOS symbols and the letters
// are the same in every language.
const translatedKeys: Record<string, TranslationKey> = {
  Shift: "key.shift",
  Enter: "key.enter",
  Delete: "key.delete",
  Space: "key.space",
};

export type KeyName = (key: TranslationKey) => string;

// The key caps to show: ["Ctrl", "Shift", "Z"], with the names translated
// when a `translate` function is given.
export function shortcutKeys(
  shortcut: string,
  platform: Platform,
  translate: KeyName = (key) => t("en", key),
): string[] {
  const symbols: Record<string, string> =
    platform === "mac"
      ? { Mod: "⌘", Shift: "⇧", Alt: "⌥" }
      : { Mod: "Ctrl", Alt: "Alt" };
  return parts(shortcut).map((part) => {
    if (part in symbols) return symbols[part];
    return part in translatedKeys ? translate(translatedKeys[part]) : part;
  });
}

export function formatShortcut(
  shortcut: string,
  platform: Platform,
  translate?: KeyName,
): string {
  return shortcutKeys(shortcut, platform, translate).join(
    platform === "mac" ? "" : "+",
  );
}

type KeyEvent = Pick<
  KeyboardEvent,
  "key" | "ctrlKey" | "metaKey" | "shiftKey" | "altKey"
>;

// Does this key press match the shortcut? Control is "Mod" on Windows and
// Linux, Command on macOS. Shift is ignored for symbol keys (+ , . -), because
// the layout decides whether they need it.
export function matchesShortcut(
  event: KeyEvent,
  shortcut: string,
  platform: Platform,
): boolean {
  const list = parts(shortcut);
  const key = list[list.length - 1];
  const wantsMod = list.includes("Mod");
  const modPressed = platform === "mac" ? event.metaKey : event.ctrlKey;
  const otherMod = platform === "mac" ? event.ctrlKey : event.metaKey;

  if (modPressed !== wantsMod || otherMod || event.altKey) return false;

  const isSymbol = key.length === 1 && !/[a-z0-9]/i.test(key);
  if (!isSymbol && event.shiftKey !== list.includes("Shift")) return false;

  const pressed = event.key === " " ? "Space" : event.key;
  if (key === "+") return pressed === "+" || pressed === "=";
  return pressed.toLowerCase() === key.toLowerCase();
}
