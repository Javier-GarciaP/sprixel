// @vitest-environment jsdom
import { cleanup, fireEvent, screen, within } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { frameOf, sampleTree } from "./fixtures";
import {
  card,
  enabled,
  fakeCheckName,
  openFolder,
  pressUntil,
  renderBrowser,
  type,
} from "./test-utils";

const core = vi.hoisted(() => ({
  projectTree: vi.fn(),
  spriteFrame: vi.fn(),
  onProjectChanged: vi.fn(),
  checkName: vi.fn(),
  renameSprite: vi.fn(),
  renameFolder: vi.fn(),
  duplicateSprite: vi.fn(),
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
});
afterEach(() => {
  cleanup();
  for (const fn of Object.values(core)) fn.mockReset();
});

const renameButton = () => screen.getByRole("button", { name: "Rename" });

async function selectGoblin() {
  renderBrowser();
  await openFolder(/^Personajes/);
  fireEvent.click(await card(/goblin/));
  await screen.findByRole("heading", { name: "goblin" });
}

describe("rename", () => {
  // Checks the spec scenario "Rename with F2": a sprite is selected, F2 opens
  // the dialog with its name, the new name is sent and the dialog closes.
  it("renames the selected sprite with F2", async () => {
    core.renameSprite.mockResolvedValue({
      folder: ["Personajes"],
      name: "orc2",
    });
    await selectGoblin();

    await pressUntil("F2", () =>
      expect(
        screen.getByRole("dialog", { name: "Rename sprite" }),
      ).toBeTruthy(),
    );
    expect((screen.getByLabelText("Name") as HTMLInputElement).value).toBe(
      "goblin",
    );
    type("Name", "orc2");
    await vi.waitFor(() => expect(enabled(renameButton())).toBe(true));
    fireEvent.click(renameButton());

    await vi.waitFor(() =>
      expect(core.renameSprite).toHaveBeenCalledWith(
        ["Personajes"],
        "goblin",
        "orc2",
      ),
    );
    await vi.waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
  });

  // Checks that the rename is also in the "..." menu of the card.
  it("renames from the menu of the card", async () => {
    await selectGoblin();

    fireEvent.click(
      screen.getByRole("button", { name: "More actions for goblin" }),
    );
    fireEvent.click(await screen.findByRole("menuitem", { name: "Rename" }));

    expect(
      await screen.findByRole("dialog", { name: "Rename sprite" }),
    ).toBeTruthy();
  });

  // Checks that double-clicking the name in the details renames the sprite.
  it("renames when the name in the details is double-clicked", async () => {
    await selectGoblin();

    fireEvent.doubleClick(screen.getByRole("heading", { name: "goblin" }));

    expect(
      await screen.findByRole("dialog", { name: "Rename sprite" }),
    ).toBeTruthy();
  });

  // Checks that the new name follows the same rules as creating: a name in
  // use shows the error and disables the button; the same name does nothing.
  it("validates the new name like when creating", async () => {
    await selectGoblin();
    fireEvent.doubleClick(screen.getByRole("heading", { name: "goblin" }));
    await screen.findByRole("dialog", { name: "Rename sprite" });
    await vi.waitFor(() => expect(core.checkName).toHaveBeenCalled());
    expect(enabled(renameButton())).toBe(false);

    type("Name", "hero");

    expect(
      await screen.findByText("A sprite with that name already exists"),
    ).toBeTruthy();
    expect(enabled(renameButton())).toBe(false);
    type("Name", "goblin2");
    await vi.waitFor(() => expect(enabled(renameButton())).toBe(true));
  });

  // Checks that the name can change only in case (the core says it is free for itself).
  it("allows a change of case", async () => {
    await selectGoblin();
    fireEvent.doubleClick(screen.getByRole("heading", { name: "goblin" }));
    await screen.findByRole("dialog", { name: "Rename sprite" });

    type("Name", "Goblin");

    await vi.waitFor(() => expect(enabled(renameButton())).toBe(true));
    expect(core.checkName).toHaveBeenLastCalledWith(
      ["Personajes"],
      "Goblin",
      "goblin",
    );
  });

  // Checks that the details follow the sprite to its new name after the core
  // says the project changed.
  it("keeps the renamed sprite selected", async () => {
    let notify: (event: unknown) => void = () => {};
    core.onProjectChanged.mockImplementation(async (listener) => {
      notify = listener;
      return () => {};
    });
    core.renameSprite.mockResolvedValue({
      folder: ["Personajes"],
      name: "orc2",
    });
    await selectGoblin();
    fireEvent.doubleClick(screen.getByRole("heading", { name: "goblin" }));
    await screen.findByRole("dialog", { name: "Rename sprite" });
    type("Name", "orc2");
    await vi.waitFor(() => expect(enabled(renameButton())).toBe(true));
    const renamed = sampleTree(2);
    renamed.root.folders[0].sprites[0].name = "orc2";
    core.projectTree.mockResolvedValue(renamed);
    await vi.waitFor(() => expect(core.onProjectChanged).toHaveBeenCalled());

    fireEvent.click(renameButton());
    await vi.waitFor(() => expect(core.renameSprite).toHaveBeenCalled());
    notify({ revision: 2, touched: [] });

    expect(await screen.findByRole("heading", { name: "orc2" })).toBeTruthy();
  });

  // Checks that a folder is renamed by double-clicking it in the tree, and the
  // sprites area follows the folder to its new name.
  it("renames a folder with a double-click in the tree", async () => {
    core.renameFolder.mockResolvedValue(["Heroes"]);
    renderBrowser();
    await openFolder(/^Personajes/);

    fireEvent.doubleClick(
      within(
        await screen.findByRole("treeitem", { name: /Personajes/ }),
      ).getByRole("button", { name: /^Personajes/ }),
    );
    await screen.findByRole("dialog", { name: "Rename folder" });
    type("Name", "Heroes");
    await vi.waitFor(() => expect(enabled(renameButton())).toBe(true));
    fireEvent.click(renameButton());

    await vi.waitFor(() =>
      expect(core.renameFolder).toHaveBeenCalledWith(["Personajes"], "Heroes"),
    );
  });

  // Checks that F2 with no sprite selected renames the folder that is open,
  // and does nothing at the project root.
  it("renames the open folder with F2 when no sprite is selected", async () => {
    renderBrowser();
    await screen.findByRole("treeitem", { name: /Personajes/ });
    fireEvent.keyDown(window, { key: "F2" });
    expect(screen.queryByRole("dialog")).toBeNull();

    await openFolder(/^Enemigos/);
    await pressUntil("F2", () =>
      expect(
        screen.getByRole("dialog", { name: "Rename folder" }),
      ).toBeTruthy(),
    );

    expect((screen.getByLabelText("Name") as HTMLInputElement).value).toBe(
      "Enemigos",
    );
  });

  // Checks that an error of the core shows in the dialog and keeps it open.
  it("shows an error from the core", async () => {
    core.renameSprite.mockRejectedValue({
      code: "name_exists",
      message: "taken",
    });
    await selectGoblin();
    fireEvent.doubleClick(screen.getByRole("heading", { name: "goblin" }));
    await screen.findByRole("dialog", { name: "Rename sprite" });
    type("Name", "other");
    await vi.waitFor(() => expect(enabled(renameButton())).toBe(true));

    fireEvent.click(renameButton());

    expect(await screen.findByRole("alert")).toBeTruthy();
  });
});

describe("duplicate", () => {
  // Checks the spec scenario "Duplicate": the button of the details asks the
  // core to duplicate the selected sprite.
  it("duplicates from the details button", async () => {
    core.duplicateSprite.mockResolvedValue({
      folder: ["Personajes"],
      name: "goblin copy",
    });
    await selectGoblin();

    fireEvent.click(screen.getByRole("button", { name: "Duplicate" }));

    await vi.waitFor(() =>
      expect(core.duplicateSprite).toHaveBeenCalledWith(
        ["Personajes"],
        "goblin",
      ),
    );
  });

  // Checks that the "..." menu of the card duplicates too.
  it("duplicates from the menu of the card", async () => {
    core.duplicateSprite.mockResolvedValue({
      folder: ["Personajes"],
      name: "goblin copy",
    });
    await selectGoblin();

    fireEvent.click(
      screen.getByRole("button", { name: "More actions for goblin" }),
    );
    fireEvent.click(await screen.findByRole("menuitem", { name: "Duplicate" }));

    await vi.waitFor(() =>
      expect(core.duplicateSprite).toHaveBeenCalledWith(
        ["Personajes"],
        "goblin",
      ),
    );
  });

  // Checks that Ctrl+D duplicates the selected sprite.
  it("duplicates with Ctrl+D", async () => {
    core.duplicateSprite.mockResolvedValue({
      folder: ["Personajes"],
      name: "goblin copy",
    });
    await selectGoblin();

    await pressUntil(
      "d",
      () =>
        expect(core.duplicateSprite).toHaveBeenCalledWith(
          ["Personajes"],
          "goblin",
        ),
      { ctrlKey: true },
    );
  });

  // Checks that a failed duplicate shows a message instead of failing silently.
  it("shows a message when the duplicate fails", async () => {
    core.duplicateSprite.mockRejectedValue({
      code: "io",
      message: "disk full",
    });
    await selectGoblin();

    fireEvent.click(screen.getByRole("button", { name: "Duplicate" }));

    expect(await screen.findByRole("status")).toBeTruthy();
    expect(screen.getByText(/disk full/)).toBeTruthy();
  });
});
