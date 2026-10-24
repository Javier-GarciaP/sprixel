// @vitest-environment jsdom
import { cleanup, fireEvent, screen, within } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { frameOf, sampleTree } from "./fixtures";
import { renderBrowser } from "./test-utils";

const core = vi.hoisted(() => ({
  projectTree: vi.fn(),
  spriteFrame: vi.fn(),
  onProjectChanged: vi.fn(),
  deleteFolder: vi.fn(),
  projectUndo: vi.fn(),
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
  core.deleteFolder.mockResolvedValue(2);
  core.projectUndo.mockResolvedValue(undefined);
});
afterEach(() => {
  cleanup();
  for (const fn of Object.values(core)) fn.mockReset();
});

async function askToDelete(folder: RegExp) {
  renderBrowser();
  await screen.findByRole("treeitem", { name: folder });
  const row = screen.getByRole("treeitem", { name: folder });
  const label = folder.source.replace("^", "");
  fireEvent.click(
    within(row).getByRole("button", { name: `More actions for ${label}` }),
  );
  fireEvent.click(
    await screen.findByRole("menuitem", { name: "Delete folder" }),
  );
  return screen.findByRole("dialog");
}

describe("delete a folder", () => {
  // Checks the spec scenario "Delete a folder": the dialog asks "Delete the
  // folder 'Personajes'?" and says that it and its 3 sprites go to the trash,
  // listing them.
  it("asks first and lists what goes to the trash", async () => {
    const dialog = await askToDelete(/Personajes/);

    expect(
      within(dialog).getByRole("heading", {
        name: "Delete the folder 'Personajes'?",
      }),
    ).toBeTruthy();
    expect(
      within(dialog).getByText(
        "The folder and its 3 sprites will be moved to the trash.",
      ),
    ).toBeTruthy();
    for (const name of ["goblin", "hero", "merchant"])
      expect(within(dialog).getByText(name)).toBeTruthy();
    expect(core.deleteFolder).not.toHaveBeenCalled();
  });

  // Checks the rule "with the focus on Cancel": pressing Enter by mistake cancels.
  it("starts with the focus on Cancel", async () => {
    const dialog = await askToDelete(/Personajes/);

    await vi.waitFor(() =>
      expect(document.activeElement).toBe(
        within(dialog).getByRole("button", { name: "Cancel" }),
      ),
    );
  });

  // Checks that Cancel and Escape close the dialog without deleting anything.
  it("does nothing on Cancel or Escape", async () => {
    const dialog = await askToDelete(/Personajes/);
    fireEvent.click(within(dialog).getByRole("button", { name: "Cancel" }));
    await vi.waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());

    fireEvent.click(
      within(screen.getByRole("treeitem", { name: /Personajes/ })).getByRole(
        "button",
        {
          name: "More actions for Personajes",
        },
      ),
    );
    fireEvent.click(
      await screen.findByRole("menuitem", { name: "Delete folder" }),
    );
    await screen.findByRole("dialog");
    fireEvent.keyDown(window, { key: "Escape" });

    await vi.waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
    expect(core.deleteFolder).not.toHaveBeenCalled();
  });

  // Checks that confirming sends the folder to the trash and offers Undo.
  it("deletes the folder when confirmed and offers Undo", async () => {
    const dialog = await askToDelete(/Personajes/);

    fireEvent.click(within(dialog).getByRole("button", { name: "Delete" }));

    await vi.waitFor(() =>
      expect(core.deleteFolder).toHaveBeenCalledWith(["Personajes"]),
    );
    expect(
      await screen.findByText("Folder Personajes moved to the trash"),
    ).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "Undo" }));
    await vi.waitFor(() => expect(core.projectUndo).toHaveBeenCalled());
  });

  // Checks the wording for a folder with one sprite and for an empty folder.
  it("words the message for one sprite and for an empty folder", async () => {
    const dialog = await askToDelete(/Enemigos/);
    expect(
      within(dialog).getByText(
        "The folder and its 1 sprite will be moved to the trash.",
      ),
    ).toBeTruthy();
    cleanup();

    const tree = sampleTree();
    tree.root.folders[1].sprites = [];
    core.projectTree.mockResolvedValue(tree);
    const empty = await askToDelete(/Enemigos/);
    expect(
      within(empty).getByText("The empty folder will be moved to the trash."),
    ).toBeTruthy();
  });

  // Checks that Delete with a folder open (and no sprite selected) asks too.
  it("asks when Delete is pressed with a folder open", async () => {
    renderBrowser();
    fireEvent.click(
      within(
        await screen.findByRole("treeitem", { name: /Enemigos/ }),
      ).getByRole("button", {
        name: /^Enemigos/,
      }),
    );

    await vi.waitFor(() => {
      fireEvent.keyDown(window, { key: "Delete" });
      expect(screen.getByRole("dialog")).toBeTruthy();
    });

    expect(
      screen.getByRole("heading", { name: "Delete the folder 'Enemigos'?" }),
    ).toBeTruthy();
  });
});
