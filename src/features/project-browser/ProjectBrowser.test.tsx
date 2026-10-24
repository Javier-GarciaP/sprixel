// @vitest-environment jsdom
import {
  cleanup,
  fireEvent,
  render,
  screen,
  within,
} from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { AppProviders } from "@/app/AppProviders";
import { defaultPreferences } from "@/shared/lib/preferences";
import { emptyTree, frameOf, sampleTree } from "./fixtures";

const core = vi.hoisted(() => ({
  projectTree: vi.fn(),
  spriteFrame: vi.fn(),
  onProjectChanged: vi.fn(),
}));
vi.mock("@/shared/lib/core", async (original) => ({
  ...(await original<typeof import("@/shared/lib/core")>()),
  ...core,
}));

import { ProjectBrowser } from "./ProjectBrowser";

beforeEach(() => {
  core.projectTree.mockResolvedValue(sampleTree());
  core.spriteFrame.mockResolvedValue(frameOf());
  core.onProjectChanged.mockResolvedValue(() => {});
});
afterEach(() => {
  cleanup();
  for (const fn of Object.values(core)) fn.mockReset();
});

function renderBrowser(
  props: Partial<Parameters<typeof ProjectBrowser>[0]> = {},
) {
  const handlers = {
    onOpenSprite: vi.fn(),
    onOpenPalette: vi.fn(),
    onNewPalette: vi.fn(),
    ...props,
  };
  render(
    <AppProviders initial={defaultPreferences("en")}>
      <ProjectBrowser {...handlers} />
    </AppProviders>,
  );
  return handlers;
}

const folderRow = (name: RegExp) => screen.findByRole("treeitem", { name });

describe("folders tree", () => {
  // Checks that every folder shows the number of items next to its name.
  it("shows each folder with its number of items", async () => {
    renderBrowser();

    expect(await folderRow(/Personajes\s*3/)).toBeTruthy();
    expect(await folderRow(/Enemigos\s*1/)).toBeTruthy();
  });

  // Checks the spec scenario "Browse folders": selecting a folder highlights
  // it in the tree and the middle area shows its name.
  it("highlights the selected folder and shows it in the middle", async () => {
    renderBrowser();

    fireEvent.click(
      within(await folderRow(/Personajes/)).getByRole("button", {
        name: /^Personajes/,
      }),
    );

    await vi.waitFor(() =>
      expect(
        screen
          .getByRole("treeitem", { name: /Personajes/ })
          .getAttribute("aria-selected"),
      ).toBe("true"),
    );
    expect(
      within(screen.getByRole("region", { name: "Sprites" })).getByRole(
        "heading",
        {
          name: "Personajes",
        },
      ),
    ).toBeTruthy();
  });

  // Checks that a folder opens and closes with its arrow, and that the
  // sprites inside show their number of frames.
  it("expands and collapses a folder, with frames next to sprites", async () => {
    renderBrowser();
    await folderRow(/Personajes/);

    fireEvent.click(screen.getByRole("button", { name: "Expand Personajes" }));

    expect(
      await screen.findByRole("treeitem", { name: /goblin\s*4/ }),
    ).toBeTruthy();
    expect(screen.getByRole("treeitem", { name: /hero\s*3/ })).toBeTruthy();

    fireEvent.click(
      screen.getByRole("button", { name: "Collapse Personajes" }),
    );

    await vi.waitFor(() =>
      expect(screen.queryByRole("treeitem", { name: /goblin/ })).toBeNull(),
    );
  });

  // Checks the spec scenario "New project": no folders gives the "No folders yet" text.
  it("says there are no folders yet in an empty project", async () => {
    core.projectTree.mockResolvedValue(emptyTree());
    renderBrowser();

    expect(await screen.findByText("No folders yet")).toBeTruthy();
  });

  // Checks that the tree is read again after a `project://changed` event
  // (the UI never changes the tree by itself).
  it("reloads the tree when the core says the project changed", async () => {
    let notify: (event: unknown) => void = () => {};
    core.onProjectChanged.mockImplementation(async (listener) => {
      notify = listener;
      return () => {};
    });
    renderBrowser();
    await folderRow(/Personajes/);
    const tree = sampleTree(2);
    tree.root.folders.push({
      name: "Items",
      path: ["Items"],
      folders: [],
      sprites: [],
    });
    core.projectTree.mockResolvedValue(tree);

    await vi.waitFor(() => expect(core.onProjectChanged).toHaveBeenCalled());
    notify({ revision: 2, touched: [{ kind: "folder", path: ["Items"] }] });

    expect(await folderRow(/Items\s*0/)).toBeTruthy();
  });

  // Checks that a selected folder that disappears (deleted, renamed) falls
  // back to the project root instead of showing a ghost.
  it("falls back to the root when the selected folder is gone", async () => {
    let notify: (event: unknown) => void = () => {};
    core.onProjectChanged.mockImplementation(async (listener) => {
      notify = listener;
      return () => {};
    });
    renderBrowser();
    fireEvent.click(
      within(await folderRow(/Enemigos/)).getByRole("button", {
        name: /^Enemigos/,
      }),
    );
    const tree = sampleTree(2);
    tree.root.folders.pop();
    core.projectTree.mockResolvedValue(tree);
    await vi.waitFor(() => expect(core.onProjectChanged).toHaveBeenCalled());

    notify({ revision: 2, touched: [{ kind: "folder", path: ["Enemigos"] }] });

    await vi.waitFor(() =>
      expect(
        within(screen.getByRole("region", { name: "Sprites" })).getByRole(
          "heading",
        ).textContent,
      ).toBe("Dungeon Kit"),
    );
  });
});

describe("palettes list", () => {
  // Checks the spec scenario "Open a palette": clicking it asks the app to open its editor.
  it("opens a palette when it is clicked", async () => {
    const handlers = renderBrowser();

    fireEvent.click(await screen.findByRole("button", { name: /dungeon16/ }));

    expect(handlers.onOpenPalette).toHaveBeenCalledWith("dungeon16");
  });

  // Checks that the plus button asks the app to create a palette.
  it("has a button to create a palette", async () => {
    const handlers = renderBrowser();

    fireEvent.click(await screen.findByRole("button", { name: "New palette" }));

    expect(handlers.onNewPalette).toHaveBeenCalled();
  });
});
