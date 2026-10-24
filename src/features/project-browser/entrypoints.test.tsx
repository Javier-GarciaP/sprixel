// @vitest-environment jsdom
import { cleanup, fireEvent, screen, within } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { frameOf, sampleTree } from "./fixtures";
import {
  card,
  fakeCheckName,
  openFolder,
  pressUntil,
  renderBrowser,
} from "./test-utils";

const core = vi.hoisted(() => ({
  projectTree: vi.fn(),
  spriteFrame: vi.fn(),
  onProjectChanged: vi.fn(),
  checkName: vi.fn(),
  duplicateSprite: vi.fn(),
  deleteSprites: vi.fn(),
  moveSprites: vi.fn(),
}));
vi.mock("@/shared/lib/core", async (original) => ({
  ...(await original<typeof import("@/shared/lib/core")>()),
  ...core,
}));
vi.mock("@/shared/lib/render", async (original) => ({
  ...(await original<typeof import("@/shared/lib/render")>()),
  drawFrame: vi.fn(),
}));

beforeEach(() => {
  core.projectTree.mockResolvedValue(sampleTree());
  core.spriteFrame.mockResolvedValue(frameOf());
  core.onProjectChanged.mockResolvedValue(() => {});
  core.checkName.mockImplementation(fakeCheckName);
  core.duplicateSprite.mockResolvedValue({
    folder: ["Personajes"],
    name: "goblin copy",
  });
  core.deleteSprites.mockResolvedValue(2);
  core.moveSprites.mockResolvedValue(undefined);
});
afterEach(() => {
  cleanup();
  for (const fn of Object.values(core)) fn.mockReset();
});

const goblin = { folder: ["Personajes"], name: "goblin" };

type App = ReturnType<typeof renderBrowser>;

// Every way to start an action on the sprite "goblin". Each one is a function
// that does it; the tests then check that the result is the same.
async function prepare(): Promise<App> {
  const app = renderBrowser();
  await openFolder(/^Personajes/);
  fireEvent.click(await card(/goblin/));
  await screen.findByRole("heading", { name: "goblin" });
  return app;
}

const viaMenuButton = async (item: string) => {
  fireEvent.click(
    screen.getByRole("button", { name: "More actions for goblin" }),
  );
  fireEvent.click(await screen.findByRole("menuitem", { name: item }));
};
const viaRightClick = async (item: string) => {
  fireEvent.contextMenu(await card(/goblin/), { clientX: 40, clientY: 50 });
  fireEvent.click(await screen.findByRole("menuitem", { name: item }));
};
const viaDetails = async (button: string) => {
  const details = screen.getByRole("complementary", { name: "Details" });
  fireEvent.click(within(details).getByRole("button", { name: button }));
};

describe("right-click menu", () => {
  // Checks the spec scenario "Right-click": a menu opens next to the pointer
  // with Open in editor, Rename, Duplicate, Move to folder and Delete.
  it("opens next to the card with the five actions", async () => {
    renderBrowser();
    await openFolder(/^Personajes/);

    fireEvent.contextMenu(await card(/goblin/), { clientX: 120, clientY: 80 });

    const menu = await screen.findByRole("menu");
    for (const name of [
      "Open in editor",
      "Rename",
      "Duplicate",
      "Move to folder",
      "Delete",
    ])
      expect(within(menu).getByRole("menuitem", { name })).toBeTruthy();
    const place = menu.parentElement as HTMLElement;
    expect([place.style.left, place.style.top]).toEqual(["120px", "80px"]);
  });

  // Checks that a right click on a card that was not selected selects it.
  it("selects the card that was right-clicked", async () => {
    renderBrowser();
    await openFolder(/^Personajes/);

    fireEvent.contextMenu(await card(/hero/));

    expect(await screen.findByRole("heading", { name: "hero" })).toBeTruthy();
  });

  // Checks that a right click on a folder of the tree gives Rename and Delete folder.
  it("opens a menu for a folder", async () => {
    renderBrowser();

    fireEvent.contextMenu(
      await screen.findByRole("treeitem", { name: /Enemigos/ }),
    );

    const menu = await screen.findByRole("menu");
    expect(within(menu).getByRole("menuitem", { name: "Rename" })).toBeTruthy();
    expect(
      within(menu).getByRole("menuitem", { name: "Delete folder" }),
    ).toBeTruthy();
  });

  // Checks that Escape closes the menu.
  it("closes with Escape", async () => {
    renderBrowser();
    await openFolder(/^Personajes/);
    fireEvent.contextMenu(await card(/goblin/));
    await screen.findByRole("menu");

    fireEvent.keyDown(window, { key: "Escape" });

    await vi.waitFor(() => expect(screen.queryByRole("menu")).toBeNull());
  });
});

describe("one action list for every entry point", () => {
  // Checks the spec scenario for each entry point: the "..." button, the
  // right-click menu, the details panel and the keyboard all call the core
  // with the same sprite.
  it.each([
    ["the ... button", () => viaMenuButton("Duplicate")],
    ["the right-click menu", () => viaRightClick("Duplicate")],
    ["the details panel", () => viaDetails("Duplicate")],
    [
      "Ctrl+D",
      () =>
        pressUntil("d", () => expect(core.duplicateSprite).toHaveBeenCalled(), {
          ctrlKey: true,
        }),
    ],
  ])("duplicate from %s", async (_name, start) => {
    await prepare();

    await start();

    await vi.waitFor(() =>
      expect(core.duplicateSprite).toHaveBeenCalledWith(
        ["Personajes"],
        "goblin",
      ),
    );
  });

  it.each([
    ["the ... button", () => viaMenuButton("Delete")],
    ["the right-click menu", () => viaRightClick("Delete")],
    ["the details panel", () => viaDetails("Delete")],
    [
      "the Delete key",
      () =>
        pressUntil("Delete", () =>
          expect(core.deleteSprites).toHaveBeenCalled(),
        ),
    ],
  ])("delete from %s", async (_name, start) => {
    await prepare();

    await start();

    await vi.waitFor(() =>
      expect(core.deleteSprites).toHaveBeenCalledWith([goblin]),
    );
  });

  it.each([
    ["the ... button", () => viaMenuButton("Rename")],
    ["the right-click menu", () => viaRightClick("Rename")],
    ["the details panel", () => viaDetails("Rename")],
    [
      "F2",
      () =>
        pressUntil("F2", () =>
          expect(
            screen.getByRole("dialog", { name: "Rename sprite" }),
          ).toBeTruthy(),
        ),
    ],
    [
      "a double-click on the name",
      async () =>
        fireEvent.doubleClick(screen.getByRole("heading", { name: "goblin" })),
    ],
  ])("rename from %s", async (_name, start) => {
    await prepare();

    await start();

    const dialog = await screen.findByRole("dialog", { name: "Rename sprite" });
    expect(
      (within(dialog).getByLabelText("Name") as HTMLInputElement).value,
    ).toBe("goblin");
  });

  const moveByMenu = async (open: () => Promise<unknown>) => {
    await open();
    fireEvent.click(await screen.findByRole("menuitem", { name: "Enemigos" }));
  };
  it.each([
    [
      "the ... button",
      async () => {
        fireEvent.click(
          screen.getByRole("button", { name: "More actions for goblin" }),
        );
        await moveByMenu(async () =>
          fireEvent.click(
            await screen.findByRole("menuitem", { name: "Move to folder" }),
          ),
        );
      },
    ],
    [
      "the right-click menu",
      async () => {
        fireEvent.contextMenu(await card(/goblin/));
        await moveByMenu(async () =>
          fireEvent.click(
            await screen.findByRole("menuitem", { name: "Move to folder" }),
          ),
        );
      },
    ],
    [
      "the details panel",
      async () => {
        const details = screen.getByRole("complementary", { name: "Details" });
        await moveByMenu(async () =>
          fireEvent.click(
            within(details).getByRole("button", { name: "Move to folder" }),
          ),
        );
      },
    ],
  ])("move from %s", async (_name, start) => {
    await prepare();

    await start();

    await vi.waitFor(() =>
      expect(core.moveSprites).toHaveBeenCalledWith([goblin], ["Enemigos"]),
    );
  });

  it.each([
    ["the ... button", () => viaMenuButton("Open in editor")],
    ["the right-click menu", () => viaRightClick("Open in editor")],
    ["the details panel", () => viaDetails("Open in editor")],
    [
      "the Enter key",
      (app: App) =>
        pressUntil("Enter", () => expect(app.onOpenSprite).toHaveBeenCalled()),
    ],
    [
      "a double-click on the card",
      async () => fireEvent.doubleClick(await card(/goblin/)),
    ],
  ])("open from %s", async (_name, start) => {
    const app = await prepare();

    await (start as (app: App) => Promise<unknown>)(app);

    await vi.waitFor(() =>
      expect(app.onOpenSprite).toHaveBeenCalledWith(
        expect.objectContaining(goblin),
      ),
    );
  });
});
