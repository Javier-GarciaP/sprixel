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

// jsdom has no canvas: the drawing is replaced by a spy that records the frame.
const drawFrame = vi.hoisted(() => vi.fn());
vi.mock("@/shared/lib/render", async (original) => ({
  ...(await original<typeof import("@/shared/lib/render")>()),
  drawFrame,
}));

import { ProjectBrowser } from "./ProjectBrowser";

beforeEach(() => {
  core.projectTree.mockResolvedValue(sampleTree());
  core.spriteFrame.mockResolvedValue(frameOf());
  core.onProjectChanged.mockResolvedValue(() => {});
});
afterEach(() => {
  cleanup();
  drawFrame.mockReset();
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

// Selects the folder from the tree, so its sprites show in the middle.
async function openFolder(name: RegExp) {
  fireEvent.click(
    within(await screen.findByRole("treeitem", { name })).getByRole("button", {
      name,
    }),
  );
}

// A card of the middle area (the tree has buttons with the same names).
const card = (name: RegExp) =>
  within(screen.getByRole("region", { name: "Sprites" })).findByRole("button", {
    name,
  });

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

describe("sprite cards", () => {
  // Checks the spec scenario "Card information": a card shows the name and
  // "8x8 - 4 frames", and the cards end with the "New sprite" card.
  it("shows the card of a sprite with its size and frames", async () => {
    renderBrowser();
    await openFolder(/^Personajes/);

    const goblin = await card(/goblin/);

    expect(within(goblin).getByText("goblin")).toBeTruthy();
    expect(within(goblin).getByText("8x8 - 4 frames")).toBeTruthy();
    expect(screen.getByRole("button", { name: "New sprite" })).toBeTruthy();
  });

  // Checks that a sprite with one frame says "1 frame", not "1 frames".
  it("uses the singular for one frame", async () => {
    renderBrowser();

    expect(await screen.findByText("8x8 - 1 frame")).toBeTruthy();
  });

  // Checks that the first frame is asked from the core and drawn on the card.
  it("draws the first frame of each sprite", async () => {
    renderBrowser();
    await openFolder(/^Personajes/);

    await vi.waitFor(() => expect(drawFrame).toHaveBeenCalled());
    expect(core.spriteFrame).toHaveBeenCalledWith(["Personajes"], "goblin", 1);
    expect(drawFrame.mock.calls[0][1]).toEqual(frameOf());
  });

  // Checks that a changed sprite is drawn again (and a sprite that was not
  // touched is not asked again): the thumbnail follows the document.
  it("draws a sprite again after the core says it changed", async () => {
    let notify: (event: unknown) => void = () => {};
    core.onProjectChanged.mockImplementation(async (listener) => {
      notify = listener;
      return () => {};
    });
    renderBrowser();
    await openFolder(/^Personajes/);
    await vi.waitFor(() =>
      expect(core.spriteFrame).toHaveBeenCalledWith(["Personajes"], "hero", 1),
    );
    await vi.waitFor(() => expect(drawFrame).toHaveBeenCalled());
    core.spriteFrame.mockResolvedValue(frameOf(2));
    core.projectTree.mockResolvedValue(sampleTree(2));
    await vi.waitFor(() => expect(core.onProjectChanged).toHaveBeenCalled());

    notify({
      revision: 2,
      touched: [{ kind: "sprite", path: ["Personajes", "goblin"] }],
    });

    await vi.waitFor(() =>
      expect(drawFrame.mock.calls.some(([, f]) => f.revision === 2)).toBe(true),
    );
    const asked = core.spriteFrame.mock.calls.filter(
      ([, name]) => name === "goblin",
    );
    expect(asked).toHaveLength(2);
    const others = core.spriteFrame.mock.calls.filter(
      ([, name]) => name === "hero",
    );
    expect(others).toHaveLength(1);
  });
});

describe("sprite details", () => {
  // Checks that selecting a card shows its details: size, frames, palette and animations.
  it("shows the details of the selected sprite", async () => {
    renderBrowser();
    await openFolder(/^Personajes/);

    fireEvent.click(await card(/goblin/));

    const details = await screen.findByRole("complementary", {
      name: "Details",
    });
    expect(
      within(details).getByRole("heading", { name: "goblin" }),
    ).toBeTruthy();
    expect(within(details).getByText("8 × 8 px")).toBeTruthy();
    expect(within(details).getByText("dungeon16")).toBeTruthy();
    expect(within(details).getByText("walk")).toBeTruthy();
    expect(
      within(details).getByText("frames 1-4 · 12 fps · repeats"),
    ).toBeTruthy();
  });

  // Checks the spec scenario "Last edited by an AI": the client name and the time show.
  it("says which AI client edited the sprite last", async () => {
    const tree = sampleTree();
    tree.root.folders[0].sprites[0].lastEditor = {
      author: "claude-code",
      atMs: Date.now(),
    };
    core.projectTree.mockResolvedValue(tree);
    renderBrowser();
    await openFolder(/^Personajes/);

    fireEvent.click(await card(/goblin/));

    expect(await screen.findByText(/Edited by claude-code · /)).toBeTruthy();
  });

  // Checks that a sprite nobody edited in this session shows no "Edited by" line.
  it("shows no editor when there is none", async () => {
    renderBrowser();
    await openFolder(/^Personajes/);

    fireEvent.click(await card(/goblin/));

    await screen.findByRole("heading", { name: "goblin" });
    expect(screen.queryByText(/Edited by/)).toBeNull();
  });

  // Checks that a sprite with no animation says so.
  it("says when a sprite has no animations", async () => {
    renderBrowser();
    await openFolder(/^Personajes/);

    fireEvent.click(await card(/hero/));

    expect(await screen.findByText("No animations")).toBeTruthy();
  });

  // Checks that selecting a sprite in the tree selects it in the middle and shows its details.
  it("shows the details when a sprite is chosen in the tree", async () => {
    renderBrowser();
    fireEvent.click(
      await screen.findByRole("button", { name: "Expand Personajes" }),
    );

    fireEvent.click(
      within(
        await screen.findByRole("treeitem", { name: /merchant/ }),
      ).getByRole("button"),
    );

    expect(
      await screen.findByRole("heading", { name: "merchant" }),
    ).toBeTruthy();
    expect(
      screen.getByRole("button", { name: /merchant/, pressed: true }),
    ).toBeTruthy();
  });
});

describe("empty states", () => {
  // Checks the spec scenario "New project": the empty message with one main
  // action, and no details panel.
  it("shows the empty message and no details in a project without sprites", async () => {
    core.projectTree.mockResolvedValue(emptyTree());
    renderBrowser();

    expect(await screen.findByText("No sprites yet")).toBeTruthy();
    expect(screen.getByRole("button", { name: "New sprite" })).toBeTruthy();
    expect(screen.queryByRole("complementary", { name: "Details" })).toBeNull();
  });
});
