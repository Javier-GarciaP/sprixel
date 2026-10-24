// @vitest-environment jsdom
import { act, cleanup, fireEvent, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { frameOf, sampleTree } from "./fixtures";
import { UNDO_MS } from "./ProjectBrowser";
import { card, openFolder, pressUntil, renderBrowser } from "./test-utils";

const core = vi.hoisted(() => ({
  projectTree: vi.fn(),
  spriteFrame: vi.fn(),
  onProjectChanged: vi.fn(),
  deleteSprites: vi.fn(),
  projectUndo: vi.fn(),
  projectRedo: vi.fn(),
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
  core.projectUndo.mockResolvedValue(undefined);
  core.projectRedo.mockResolvedValue(undefined);
});
afterEach(() => {
  vi.useRealTimers();
  cleanup();
  for (const fn of Object.values(core)) fn.mockReset();
});

async function selectGoblin() {
  renderBrowser();
  await openFolder(/^Personajes/);
  fireEvent.click(await card(/goblin/));
  await screen.findByRole("heading", { name: "goblin" });
}

const goblin = { folder: ["Personajes"], name: "goblin" };

describe("delete a sprite", () => {
  // Checks the spec scenario "Delete and undo": Delete does not ask, the core
  // moves the sprite to the trash, and the message offers Undo.
  it("deletes with the Delete key without asking and offers Undo", async () => {
    await selectGoblin();

    await pressUntil("Delete", () =>
      expect(core.deleteSprites).toHaveBeenCalledWith([goblin]),
    );

    expect(await screen.findByText("goblin moved to the trash")).toBeTruthy();
    expect(screen.getByRole("button", { name: "Undo" })).toBeTruthy();
    expect(screen.queryByRole("dialog")).toBeNull();
  });

  // Checks the other entry points: the Delete button of the details and the
  // "..." menu of the card ask the core to delete the same sprite.
  it("deletes from the details button and from the menu", async () => {
    await selectGoblin();

    fireEvent.click(screen.getByRole("button", { name: "Delete" }));
    await vi.waitFor(() => expect(core.deleteSprites).toHaveBeenCalledTimes(1));
    fireEvent.click(
      screen.getByRole("button", { name: "More actions for goblin" }),
    );
    fireEvent.click(await screen.findByRole("menuitem", { name: "Delete" }));

    await vi.waitFor(() => expect(core.deleteSprites).toHaveBeenCalledTimes(2));
    expect(core.deleteSprites).toHaveBeenLastCalledWith([goblin]);
  });

  // Checks that pressing Undo in the message asks the core to undo, and the message goes away.
  it("undoes from the message", async () => {
    await selectGoblin();
    fireEvent.click(screen.getByRole("button", { name: "Delete" }));
    fireEvent.click(await screen.findByRole("button", { name: "Undo" }));

    await vi.waitFor(() => expect(core.projectUndo).toHaveBeenCalledTimes(1));
    await vi.waitFor(() =>
      expect(screen.queryByText(/moved to the trash/)).toBeNull(),
    );
  });

  // Checks that Ctrl+Z also undoes (and Ctrl+Shift+Z redoes), as the spec says.
  it("undoes with Ctrl+Z and redoes with Ctrl+Shift+Z", async () => {
    await selectGoblin();

    await pressUntil("z", () => expect(core.projectUndo).toHaveBeenCalled(), {
      ctrlKey: true,
    });
    await pressUntil("Z", () => expect(core.projectRedo).toHaveBeenCalled(), {
      ctrlKey: true,
      shiftKey: true,
    });
  });

  // Checks that "nothing to undo" is not shown as an error, but another error is.
  it("ignores nothing to undo and shows other errors", async () => {
    core.projectUndo.mockRejectedValueOnce({
      code: "nothing_to_undo",
      message: "x",
    });
    await selectGoblin();

    await pressUntil(
      "z",
      () => expect(core.projectUndo).toHaveBeenCalledTimes(1),
      {
        ctrlKey: true,
      },
    );
    core.projectUndo.mockRejectedValueOnce({
      code: "io",
      message: "disk full",
    });
    fireEvent.keyDown(window, { key: "z", ctrlKey: true });

    expect(await screen.findByText(/disk full/)).toBeTruthy();
    expect(screen.queryByText(/nothing_to_undo/)).toBeNull();
  });

  // Checks that a failed delete shows the error and no Undo message.
  it("shows an error when the delete fails", async () => {
    core.deleteSprites.mockRejectedValue({
      code: "not_found",
      message: "gone",
    });
    await selectGoblin();

    fireEvent.click(screen.getByRole("button", { name: "Delete" }));

    expect(await screen.findByText(/gone/)).toBeTruthy();
    expect(screen.queryByRole("button", { name: "Undo" })).toBeNull();
  });

  // Checks that the message stays for 8 seconds and then goes (fake timers: no real waiting).
  it("hides the Undo message after 8 seconds", async () => {
    await selectGoblin();
    vi.useFakeTimers({ toFake: ["setTimeout", "clearTimeout"] });

    await act(async () => {
      fireEvent.click(screen.getByRole("button", { name: "Delete" }));
      await vi.advanceTimersByTimeAsync(0);
    });
    expect(screen.getByText("goblin moved to the trash")).toBeTruthy();
    await act(async () => {
      await vi.advanceTimersByTimeAsync(UNDO_MS - 1);
    });
    expect(screen.getByText("goblin moved to the trash")).toBeTruthy();
    await act(async () => {
      await vi.advanceTimersByTimeAsync(1);
    });

    expect(screen.queryByText("goblin moved to the trash")).toBeNull();
  });

  // Checks that the message goes away when anything else changes the project,
  // because Undo would then undo that change and not the delete.
  it("hides the Undo message when the project changes again", async () => {
    let notify: (event: unknown) => void = () => {};
    core.onProjectChanged.mockImplementation(async (listener) => {
      notify = listener;
      return () => {};
    });
    await selectGoblin();
    fireEvent.click(screen.getByRole("button", { name: "Delete" }));
    await screen.findByText("goblin moved to the trash");

    core.projectTree.mockResolvedValue(sampleTree(5));
    notify({ revision: 5, touched: [] });

    await vi.waitFor(() =>
      expect(screen.queryByText("goblin moved to the trash")).toBeNull(),
    );
  });
});
