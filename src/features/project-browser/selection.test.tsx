// @vitest-environment jsdom
import { cleanup, fireEvent, screen, within } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { frameOf, sampleTree } from "./fixtures";
import { card, openFolder, pressUntil, renderBrowser } from "./test-utils";

const core = vi.hoisted(() => ({
  projectTree: vi.fn(),
  spriteFrame: vi.fn(),
  onProjectChanged: vi.fn(),
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
  core.deleteSprites.mockResolvedValue(2);
  core.moveSprites.mockResolvedValue(undefined);
});
afterEach(() => {
  cleanup();
  for (const fn of Object.values(core)) fn.mockReset();
});

const checkbox = (name: string) =>
  screen.findByRole("checkbox", { name: `Select ${name}` });

// Opens Personajes and checks goblin, hero and merchant.
async function selectThree() {
  renderBrowser();
  await openFolder(/^Personajes/);
  for (const name of ["goblin", "hero", "merchant"])
    fireEvent.click(await checkbox(name));
  await screen.findByRole("toolbar", { name: "Selection" });
}

const bar = () => screen.getByRole("toolbar", { name: "Selection" });
const targets = [
  { folder: ["Personajes"], name: "goblin" },
  { folder: ["Personajes"], name: "hero" },
  { folder: ["Personajes"], name: "merchant" },
];

describe("multiple selection", () => {
  // Checks the spec scenario "Select three sprites": the bar shows "3 selected"
  // with Move, Delete and Cancel selection, and the details list the three
  // names and their total frames (4 + 3 + 2).
  it("shows the bar and the combined details for three sprites", async () => {
    await selectThree();

    expect(within(bar()).getByText("3 selected")).toBeTruthy();
    expect(within(bar()).getByRole("button", { name: "Move" })).toBeTruthy();
    expect(within(bar()).getByRole("button", { name: "Delete" })).toBeTruthy();
    expect(
      within(bar()).getByRole("button", { name: "Cancel selection" }),
    ).toBeTruthy();
    const details = screen.getByRole("complementary", { name: "Details" });
    expect(
      within(details).getByRole("heading", { name: "3 sprites" }),
    ).toBeTruthy();
    expect(within(details).getByText("goblin, hero, merchant")).toBeTruthy();
    expect(within(details).getByText("9")).toBeTruthy();
  });

  // Checks that one selected sprite shows no bar (the details have the actions).
  it("shows no bar for a single sprite", async () => {
    renderBrowser();
    await openFolder(/^Personajes/);

    fireEvent.click(await checkbox("goblin"));

    expect(await screen.findByRole("heading", { name: "goblin" })).toBeTruthy();
    expect(screen.queryByRole("toolbar", { name: "Selection" })).toBeNull();
  });

  // Checks that Ctrl+click adds a card to the selection, Ctrl+click again
  // removes it, and a plain click selects only that card.
  it("adds and removes cards with Ctrl and replaces with a plain click", async () => {
    renderBrowser();
    await openFolder(/^Personajes/);
    fireEvent.click(await card(/goblin/));
    fireEvent.click(await card(/hero/), { ctrlKey: true });
    expect(await screen.findByText("2 selected")).toBeTruthy();

    fireEvent.click(await card(/hero/), { ctrlKey: true });
    await vi.waitFor(() => expect(screen.queryByText("2 selected")).toBeNull());

    fireEvent.click(await card(/hero/), { ctrlKey: true });
    await screen.findByText("2 selected");
    fireEvent.click(await card(/merchant/));
    await vi.waitFor(() => expect(screen.queryByText("2 selected")).toBeNull());
    expect(screen.getByRole("heading", { name: "merchant" })).toBeTruthy();
  });

  // Checks that Shift+click selects every card between the first click and this one.
  it("selects a range with Shift", async () => {
    renderBrowser();
    await openFolder(/^Personajes/);
    fireEvent.click(await card(/goblin/));

    fireEvent.click(await card(/merchant/), { shiftKey: true });

    expect(await screen.findByText("3 selected")).toBeTruthy();
  });

  // Checks that Cancel selection clears everything.
  it("clears the selection with Cancel selection", async () => {
    await selectThree();

    fireEvent.click(
      within(bar()).getByRole("button", { name: "Cancel selection" }),
    );

    await vi.waitFor(() =>
      expect(screen.queryByRole("toolbar", { name: "Selection" })).toBeNull(),
    );
    expect(((await checkbox("goblin")) as HTMLInputElement).checked).toBe(
      false,
    );
  });

  // Checks the rule "a group asks": Delete in the bar opens the confirmation
  // that lists the three sprites, with the focus on Cancel, and nothing is
  // deleted until the person confirms.
  it("asks before deleting a group, then moves it to the trash", async () => {
    await selectThree();

    fireEvent.click(within(bar()).getByRole("button", { name: "Delete" }));
    const dialog = await screen.findByRole("dialog", {
      name: "Delete 3 sprites?",
    });

    expect(
      within(dialog).getByText("These 3 sprites will be moved to the trash."),
    ).toBeTruthy();
    expect(within(dialog).getByText("Personajes / goblin")).toBeTruthy();
    await vi.waitFor(() =>
      expect(document.activeElement).toBe(
        within(dialog).getByRole("button", { name: "Cancel" }),
      ),
    );
    expect(core.deleteSprites).not.toHaveBeenCalled();
    fireEvent.click(
      within(dialog).getByRole("button", { name: "Delete 3 sprites" }),
    );

    await vi.waitFor(() =>
      expect(core.deleteSprites).toHaveBeenCalledWith(targets),
    );
    expect(
      await screen.findByText("3 sprites moved to the trash"),
    ).toBeTruthy();
    expect(screen.getByRole("button", { name: "Undo" })).toBeTruthy();
  });

  // Checks that the Delete key asks for a group too.
  it("asks when the Delete key is pressed with several selected", async () => {
    await selectThree();

    await pressUntil("Delete", () =>
      expect(
        screen.getByRole("dialog", { name: "Delete 3 sprites?" }),
      ).toBeTruthy(),
    );

    expect(core.deleteSprites).not.toHaveBeenCalled();
  });

  // Checks that Move in the bar moves all the selected sprites to the chosen folder.
  it("moves the whole selection from the bar", async () => {
    await selectThree();

    fireEvent.click(within(bar()).getByRole("button", { name: "Move" }));
    fireEvent.click(await screen.findByRole("menuitem", { name: "Enemigos" }));

    await vi.waitFor(() =>
      expect(core.moveSprites).toHaveBeenCalledWith(targets, ["Enemigos"]),
    );
  });

  // Checks that the Move button of the details moves the selection too, and
  // that its Delete button reads "Delete 3 sprites".
  it("has the same actions in the details panel", async () => {
    await selectThree();
    const details = screen.getByRole("complementary", { name: "Details" });

    fireEvent.click(
      within(details).getByRole("button", { name: "Move to folder" }),
    );
    fireEvent.click(
      await screen.findByRole("menuitem", { name: "Dungeon Kit" }),
    );
    await vi.waitFor(() =>
      expect(core.moveSprites).toHaveBeenCalledWith(targets, []),
    );

    expect(
      within(details).getByRole("button", { name: "Delete 3 sprites" }),
    ).toBeTruthy();
  });

  // Checks that the menu of a card that is part of the selection acts on all of them.
  it("uses the whole selection in the menu of a selected card", async () => {
    await selectThree();

    fireEvent.click(
      screen.getByRole("button", { name: "More actions for hero" }),
    );
    const menu = await screen.findByRole("menu");

    expect(
      within(menu).getByRole("menuitem", { name: "Delete 3 sprites" }),
    ).toBeTruthy();
    expect(within(menu).queryByRole("menuitem", { name: "Rename" })).toBeNull();
  });
});
