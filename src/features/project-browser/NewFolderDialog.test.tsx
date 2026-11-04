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
import { frameOf, sampleTree } from "./fixtures";

const core = vi.hoisted(() => ({
  projectTree: vi.fn(),
  spriteFrame: vi.fn(),
  onProjectChanged: vi.fn(),
  checkName: vi.fn(),
  createSprite: vi.fn(),
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

import { ProjectBrowser } from "./ProjectBrowser";

// The core answers like the real one for the names of the sample project:
// "Enemigos" is a folder, "orc" is a sprite, empty and "a/b" are not valid.
function fakeCheckName(parent: string[], name: string) {
  const text = name.toLowerCase();
  if (!name.trim()) return Promise.resolve("empty");
  if (name.includes("/")) return Promise.resolve("invalid_name");
  if (parent.length === 0 && text === "enemigos")
    return Promise.resolve("folder_exists");
  if (parent.length === 0 && text === "orc")
    return Promise.resolve("sprite_exists");
  return Promise.resolve(null);
}

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

function renderBrowser() {
  const onOpenSprite = vi.fn();
  render(
    <AppProviders initial={defaultPreferences("en")}>
      <ProjectBrowser
        onOpenSprite={onOpenSprite}
        onOpenPalette={vi.fn()}
        onNewPalette={vi.fn()}
      />
    </AppProviders>,
  );
  return { onOpenSprite };
}

const type = (label: string, value: string) =>
  fireEvent.change(screen.getByLabelText(label), { target: { value } });

const enabled = (button: HTMLElement) =>
  !(button as HTMLButtonElement).disabled;

describe("New folder dialog", () => {
  const openNewFolder = async () => {
    fireEvent.click(await screen.findByRole("button", { name: "New folder" }));
    return screen.findByRole("dialog", { name: "New folder" });
  };
  const createFolderButton = () =>
    screen.getByRole("button", { name: "Create folder" });

  // Checks the spec scenario "Name already exists": the field turns red with
  // the message and the create button is disabled.
  it("shows the red field and disables the button for a name in use", async () => {
    renderBrowser();
    await openNewFolder();

    type("Name", "Enemigos");

    expect(
      await screen.findByText("A folder with that name already exists"),
    ).toBeTruthy();
    expect(screen.getByLabelText("Name").getAttribute("aria-invalid")).toBe(
      "true",
    );
    expect(enabled(createFolderButton())).toBe(false);
  });

  // Checks the spec scenario "Create a folder": a good name creates it in the
  // chosen parent, and the dialog closes.
  it("creates a folder in the root", async () => {
    core.createFolder.mockResolvedValue(["Items"]);
    renderBrowser();
    await openNewFolder();
    fireEvent.change(screen.getByLabelText("Folder"), {
      target: { value: "" },
    });

    type("Name", "Items");
    await vi.waitFor(() => expect(enabled(createFolderButton())).toBe(true));
    fireEvent.click(createFolderButton());

    await vi.waitFor(() =>
      expect(core.createFolder).toHaveBeenCalledWith([], "Items"),
    );
    await vi.waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
  });

  // Checks that the new folder can be put inside the folder that is selected.
  it("starts inside the selected folder", async () => {
    renderBrowser();
    fireEvent.click(
      within(
        await screen.findByRole("treeitem", { name: /Personajes/ }),
      ).getByRole("button", { name: /^Personajes/ }),
    );
    await openNewFolder();

    expect((screen.getByLabelText("Folder") as HTMLSelectElement).value).toBe(
      "Personajes",
    );
  });

  // Checks that an error of the core shows in the dialog and keeps it open.
  it("shows an error from the core", async () => {
    core.createFolder.mockRejectedValue({ code: "io", message: "disk full" });
    renderBrowser();
    await openNewFolder();
    type("Name", "Items");
    await vi.waitFor(() => expect(enabled(createFolderButton())).toBe(true));

    fireEvent.click(createFolderButton());

    expect(await screen.findByRole("alert")).toBeTruthy();
  });
});
