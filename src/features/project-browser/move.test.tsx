// @vitest-environment jsdom
import { cleanup, fireEvent, screen, within } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { frameOf, sampleTree } from "./fixtures";
import {
  card,
  enabled,
  fakeCheckName,
  openFolder,
  renderBrowser,
  type,
} from "./test-utils";

const core = vi.hoisted(() => ({
  projectTree: vi.fn(),
  spriteFrame: vi.fn(),
  onProjectChanged: vi.fn(),
  checkName: vi.fn(),
  moveSprites: vi.fn(),
  createFolder: vi.fn(),
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
  core.moveSprites.mockResolvedValue(undefined);
});
afterEach(() => {
  cleanup();
  for (const fn of Object.values(core)) fn.mockReset();
});

const goblin = { folder: ["Personajes"], name: "goblin" };

async function openMoveMenu() {
  renderBrowser();
  await openFolder(/^Personajes/);
  await card(/goblin/);
  fireEvent.click(
    screen.getByRole("button", { name: "More actions for goblin" }),
  );
  fireEvent.click(
    await screen.findByRole("menuitem", { name: "Move to folder" }),
  );
}

describe("move to a folder", () => {
  // Checks the spec scenario "Move to a folder": choosing "Enemigos" in the
  // "Move to folder" menu asks the core to move the sprite there.
  it("moves a sprite from the menu", async () => {
    await openMoveMenu();

    fireEvent.click(await screen.findByRole("menuitem", { name: "Enemigos" }));

    await vi.waitFor(() =>
      expect(core.moveSprites).toHaveBeenCalledWith([goblin], ["Enemigos"]),
    );
  });

  // Checks that the list offers the project root and the other folders, but
  // not the folder the sprite is already in.
  it("offers every folder except the one the sprite is in", async () => {
    await openMoveMenu();

    expect(
      await screen.findByRole("menuitem", { name: "Dungeon Kit" }),
    ).toBeTruthy();
    expect(screen.getByRole("menuitem", { name: "Enemigos" })).toBeTruthy();
    expect(
      screen.getByRole("menuitem", { name: "New folder..." }),
    ).toBeTruthy();
    expect(screen.queryByRole("menuitem", { name: "Personajes" })).toBeNull();
  });

  // Checks the rule "moving fails with a clear message if the name already
  // exists in the destination".
  it("shows a clear message when the name exists in the destination", async () => {
    core.moveSprites.mockRejectedValue({ code: "name_exists", message: "x" });
    await openMoveMenu();

    fireEvent.click(await screen.findByRole("menuitem", { name: "Enemigos" }));

    expect(
      await screen.findByText(
        'Could not move: a sprite or folder with the same name already exists in "Enemigos".',
      ),
    ).toBeTruthy();
  });

  // Checks that other errors of the core are shown too.
  it("shows other errors", async () => {
    core.moveSprites.mockRejectedValue({ code: "io", message: "disk full" });
    await openMoveMenu();

    fireEvent.click(await screen.findByRole("menuitem", { name: "Enemigos" }));

    expect(await screen.findByText(/disk full/)).toBeTruthy();
  });

  // Checks that "New folder..." makes the folder first and then moves the sprite into it.
  it("creates a folder and moves the sprite into it", async () => {
    core.createFolder.mockResolvedValue(["Items"]);
    await openMoveMenu();

    fireEvent.click(
      await screen.findByRole("menuitem", { name: "New folder..." }),
    );
    await screen.findByRole("dialog", { name: "New folder" });
    type("Name", "Items");
    const create = screen.getByRole("button", { name: "Create folder" });
    await vi.waitFor(() => expect(enabled(create)).toBe(true));
    fireEvent.click(create);

    await vi.waitFor(() =>
      expect(core.createFolder).toHaveBeenCalledWith(["Personajes"], "Items"),
    );
    await vi.waitFor(() =>
      expect(core.moveSprites).toHaveBeenCalledWith([goblin], ["Items"]),
    );
  });
});

// A stand-in for the data of a drag: jsdom does not make one.
function fakeDataTransfer() {
  const data: Record<string, string> = {};
  return {
    data,
    types: [] as string[],
    effectAllowed: "",
    setData(type: string, value: string) {
      data[type] = value;
      if (!this.types.includes(type)) this.types.push(type);
    },
    getData: (type: string) => data[type] ?? "",
  };
}

describe("drag and drop", () => {
  const dragGoblinTo = async (folder: RegExp) => {
    renderBrowser();
    await openFolder(/^Personajes/);
    const article = (await card(/goblin/)).closest("article") as HTMLElement;
    const transfer = fakeDataTransfer();
    fireEvent.dragStart(article, { dataTransfer: transfer });
    const row = within(await screen.findByRole("tree")).getByRole("treeitem", {
      name: folder,
    });
    fireEvent.dragOver(row, { dataTransfer: transfer });
    fireEvent.drop(row, { dataTransfer: transfer });
  };

  // Checks that dragging a card onto a folder of the tree moves the sprite there.
  it("moves a sprite dropped on a folder", async () => {
    await dragGoblinTo(/^Enemigos/);

    await vi.waitFor(() =>
      expect(core.moveSprites).toHaveBeenCalledWith([goblin], ["Enemigos"]),
    );
  });

  // Checks that dropping on the folder the sprite is already in does nothing.
  it("ignores a drop on the same folder", async () => {
    await dragGoblinTo(/^Personajes/);

    expect(core.moveSprites).not.toHaveBeenCalled();
  });
});
