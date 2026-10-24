// Helpers shared by the tests of this feature. Each test file mocks the core
// itself (vi.mock must be in the file that uses it).
import { fireEvent, render, screen, within } from "@testing-library/react";
import { vi } from "vitest";
import { AppProviders } from "@/app/AppProviders";
import { useShortcuts } from "@/app/commands/useShortcuts";
import { defaultPreferences } from "@/shared/lib/preferences";
import { ProjectBrowser, type Shortcuts } from "./ProjectBrowser";

// The real shortcut listener of the app, on a non-Mac platform (Ctrl).
export const shortcuts: Shortcuts = {
  useHandlers: (handlers) => useShortcuts(handlers, "other"),
  hint: () => undefined,
};

export function renderBrowser(
  props: Partial<Parameters<typeof ProjectBrowser>[0]> = {},
) {
  const handlers = {
    onOpenSprite: vi.fn(),
    onOpenPalette: vi.fn(),
    onNewPalette: vi.fn(),
    shortcuts,
    ...props,
  };
  render(
    <AppProviders initial={defaultPreferences("en")}>
      <ProjectBrowser {...handlers} />
    </AppProviders>,
  );
  return handlers;
}

// Selects the folder from the tree, so its sprites show in the middle.
export async function openFolder(name: RegExp) {
  fireEvent.click(
    within(await screen.findByRole("treeitem", { name })).getByRole("button", {
      name,
    }),
  );
}

// A card of the middle area (the tree has buttons with the same names).
export const card = (name: RegExp) =>
  within(screen.getByRole("region", { name: "Sprites" })).findByRole("button", {
    // The card button starts with the name ("More actions for..." does not).
    name: new RegExp(`^${name.source}`),
  });

// Presses a key until the app reacts: the listeners are added in effects, so
// the first press may come too early.
export const pressUntil = (
  key: string,
  check: () => void,
  init: KeyboardEventInit = {},
) =>
  vi.waitFor(() => {
    fireEvent.keyDown(window, { key, ...init });
    check();
  });

export const type = (label: string, value: string) =>
  fireEvent.change(screen.getByLabelText(label), { target: { value } });

export const enabled = (button: HTMLElement) =>
  !(button as HTMLButtonElement).disabled;

// The core answers like the real one for the sample project: "Enemigos" is a
// folder, "orc" and "goblin" are sprites, empty and "a/b" are not valid.
export function fakeCheckName(
  parent: string[],
  name: string,
  current?: string,
) {
  const text = name.toLowerCase();
  const own = current?.toLowerCase() === text;
  if (!name.trim()) return Promise.resolve("empty");
  if (name.includes("/")) return Promise.resolve("invalid_name");
  if (parent.length === 0 && text === "enemigos" && !own)
    return Promise.resolve("folder_exists");
  if (parent.length === 0 && text === "orc" && !own)
    return Promise.resolve("sprite_exists");
  if (
    parent.join("/") === "Personajes" &&
    ["goblin", "hero", "merchant"].includes(text) &&
    !own
  )
    return Promise.resolve("sprite_exists");
  return Promise.resolve(null);
}
