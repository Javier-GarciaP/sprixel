// @vitest-environment jsdom
import { afterEach, describe, expect, it, vi } from "vitest";

const invoke = vi.fn();
const open = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({ invoke }));
vi.mock("@tauri-apps/plugin-dialog", () => ({ open }));

import {
  asProjectFailure,
  createProject,
  openProject,
  pickFolder,
  projectStatus,
} from "./project";

afterEach(() => {
  delete (window as unknown as Record<string, unknown>).__TAURI_INTERNALS__;
  invoke.mockReset();
  open.mockReset();
});

const insideApp = () =>
  ((window as unknown as Record<string, unknown>).__TAURI_INTERNALS__ = {});

describe("project api", () => {
  // Checks that the calls use the command names and argument names of the
  // Rust commands (parent, name, path).
  it("calls the core commands with their arguments", async () => {
    insideApp();
    invoke.mockResolvedValue({ name: "Kit", path: "/p/Kit", damagedFiles: 0 });

    await createProject("/p", "Kit");
    await openProject("/p/Kit");
    await projectStatus();

    expect(invoke).toHaveBeenNthCalledWith(1, "create_project", {
      parent: "/p",
      name: "Kit",
    });
    expect(invoke).toHaveBeenNthCalledWith(2, "open_project", {
      path: "/p/Kit",
    });
    expect(invoke).toHaveBeenNthCalledWith(3, "project_status", undefined);
  });

  // Checks that errors with a code pass through, and anything else becomes
  // an `io` error with its text.
  it("turns rejections into project failures", () => {
    expect(asProjectFailure({ code: "folder_exists" })).toEqual({
      code: "folder_exists",
    });
    expect(asProjectFailure("boom")).toEqual({ code: "io", message: "boom" });
  });

  // Checks that the folder picker asks for a directory and returns its path,
  // or null when the user cancels.
  it("picks a folder", async () => {
    insideApp();
    open.mockResolvedValueOnce("/chosen");
    expect(await pickFolder()).toBe("/chosen");
    expect(open).toHaveBeenCalledWith({ directory: true });

    open.mockResolvedValueOnce(null);
    expect(await pickFolder()).toBeNull();
  });

  // Checks that outside the app a call fails with an io error and the picker
  // gives nothing, instead of crashing.
  it("fails clearly outside the app", async () => {
    await expect(openProject("/x")).rejects.toMatchObject({ code: "io" });
    expect(await pickFolder()).toBeNull();
  });
});
