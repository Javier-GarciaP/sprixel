// @vitest-environment jsdom
import { afterEach, describe, expect, it, vi } from "vitest";

const { invoke, listen } = vi.hoisted(() => ({
  invoke: vi.fn(),
  listen: vi.fn(),
}));
vi.mock("@tauri-apps/api/core", () => ({ invoke }));
vi.mock("@tauri-apps/api/event", () => ({ listen }));

import {
  asCoreFailure,
  checkName,
  createFolder,
  createSprite,
  deleteFolder,
  deleteSprites,
  duplicateSprite,
  moveSprites,
  onProjectChanged,
  projectRedo,
  projectTree,
  projectUndo,
  renameFolder,
  renameSprite,
  spriteFrame,
} from ".";

afterEach(() => {
  delete (window as unknown as Record<string, unknown>).__TAURI_INTERNALS__;
  invoke.mockReset();
  listen.mockReset();
});

const insideApp = () =>
  ((window as unknown as Record<string, unknown>).__TAURI_INTERNALS__ = {});

describe("core api", () => {
  // Checks that the calls use the command and argument names of the Rust
  // commands (project_tree, sprite_frame with folder, name and frame).
  it("calls the core commands with their arguments", async () => {
    insideApp();
    invoke.mockResolvedValue({});

    await projectTree();
    await spriteFrame(["Personajes"], "goblin", 1);

    expect(invoke).toHaveBeenNthCalledWith(1, "project_tree", undefined);
    expect(invoke).toHaveBeenNthCalledWith(2, "sprite_frame", {
      folder: ["Personajes"],
      name: "goblin",
      frame: 1,
    });
  });

  // Checks that the calls that create things use the argument names of the
  // Rust commands, and that "no palette" is sent as null.
  it("calls the commands that create sprites and folders", async () => {
    insideApp();
    invoke.mockResolvedValue(null);

    await checkName(["A"], "x");
    await createSprite(["A"], "goblin", 16, 8, null);
    await createFolder([], "Enemigos");

    expect(invoke).toHaveBeenNthCalledWith(1, "check_name", {
      parent: ["A"],
      name: "x",
      current: null,
    });
    expect(invoke).toHaveBeenNthCalledWith(2, "create_sprite", {
      folder: ["A"],
      name: "goblin",
      width: 16,
      height: 8,
      palette: null,
    });
    expect(invoke).toHaveBeenNthCalledWith(3, "create_folder", {
      parent: [],
      name: "Enemigos",
    });
  });

  // Checks that rename and duplicate use the argument names of the Rust
  // commands (Tauri turns new_name into newName).
  it("calls rename and duplicate", async () => {
    insideApp();
    invoke.mockResolvedValue(null);

    await renameSprite(["A"], "goblin", "orc");
    await renameFolder(["A"], "B");
    await duplicateSprite(["A"], "goblin");

    expect(invoke).toHaveBeenNthCalledWith(1, "rename_sprite", {
      folder: ["A"],
      name: "goblin",
      newName: "orc",
    });
    expect(invoke).toHaveBeenNthCalledWith(2, "rename_folder", {
      path: ["A"],
      newName: "B",
    });
    expect(invoke).toHaveBeenNthCalledWith(3, "duplicate_sprite", {
      folder: ["A"],
      name: "goblin",
    });
  });

  // Checks the delete, undo and redo commands and their arguments.
  it("calls delete, undo and redo", async () => {
    insideApp();
    invoke.mockResolvedValue(null);

    await deleteSprites([{ folder: ["A"], name: "goblin" }]);
    await projectUndo();
    await projectRedo();
    await moveSprites([{ folder: [], name: "orc" }], ["Enemigos"]);
    await deleteFolder(["Personajes"]);

    expect(invoke).toHaveBeenNthCalledWith(4, "move_sprites", {
      sprites: [{ folder: [], name: "orc" }],
      to: ["Enemigos"],
    });
    expect(invoke).toHaveBeenNthCalledWith(5, "delete_folder", {
      path: ["Personajes"],
    });
    expect(invoke).toHaveBeenNthCalledWith(1, "delete_sprites", {
      sprites: [{ folder: ["A"], name: "goblin" }],
    });
    expect(invoke).toHaveBeenNthCalledWith(2, "project_undo", undefined);
    expect(invoke).toHaveBeenNthCalledWith(3, "project_redo", undefined);
  });

  // Checks that outside the app a call fails with `no_core` and does not touch the core.
  it("fails with no_core in a plain browser", async () => {
    await expect(projectTree()).rejects.toMatchObject({ code: "no_core" });
    expect(invoke).not.toHaveBeenCalled();
  });

  // Checks that the listener gets the payload of `project://changed`, and that stopping works.
  it("listens to project changes and can stop", async () => {
    insideApp();
    const stop = vi.fn();
    listen.mockResolvedValue(stop);
    const seen = vi.fn();

    const unlisten = await onProjectChanged(seen);
    const [name, handler] = listen.mock.calls[0];
    handler({ payload: { revision: 3, touched: [] } });
    unlisten();

    expect(name).toBe("project://changed");
    expect(seen).toHaveBeenCalledWith({ revision: 3, touched: [] });
    expect(stop).toHaveBeenCalled();
  });

  // Checks that errors with a code pass through and anything else becomes an `io` error.
  it("turns rejections into core failures", () => {
    expect(asCoreFailure({ code: "name_exists", message: "x" })).toEqual({
      code: "name_exists",
      message: "x",
    });
    expect(asCoreFailure("boom")).toEqual({ code: "io", message: "boom" });
  });
});
