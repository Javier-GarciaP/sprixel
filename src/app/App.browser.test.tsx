// @vitest-environment jsdom
import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

const api = vi.hoisted(() => ({
  openProject: vi.fn(),
  closeProject: vi.fn(),
  projectStatus: vi.fn(),
  loadPreferences: vi.fn(),
  addRecentProject: vi.fn(),
  folderExists: vi.fn(),
  pickFolder: vi.fn(),
}));
const core = vi.hoisted(() => ({
  projectTree: vi.fn(),
  spriteFrame: vi.fn(),
  onProjectChanged: vi.fn(),
}));

const paletteApi = vi.hoisted(() => ({
  loadPalette: vi.fn(),
  applyEdit: vi.fn(),
  listPresets: vi.fn(),
  createPalette: vi.fn(),
  rampPreview: vi.fn(),
  rampApply: vi.fn(),
  removeUnused: vi.fn(),
}));

vi.mock("./api/project", async (original) => ({
  ...(await original<typeof import("./api/project")>()),
  openProject: api.openProject,
  closeProject: api.closeProject,
  projectStatus: api.projectStatus,
  pickFolder: api.pickFolder,
}));
vi.mock("./api/preferences", async (original) => ({
  ...(await original<typeof import("./api/preferences")>()),
  loadPreferences: api.loadPreferences,
  addRecentProject: api.addRecentProject,
  folderExists: api.folderExists,
}));
vi.mock("@/shared/lib/core", async (original) => ({
  ...(await original<typeof import("@/shared/lib/core")>()),
  ...core,
}));
vi.mock("@/shared/lib/render", async (original) => ({
  ...(await original<typeof import("@/shared/lib/render")>()),
  drawFrame: vi.fn(),
}));

vi.mock("@/features/palette-editor", async (original) => ({
  ...(await original<typeof import("@/features/palette-editor")>()),
  corePaletteApi: paletteApi,
}));

import App from "./App";

const sprite = (name: string) => ({
  name,
  folder: ["Personajes"],
  width: 8,
  height: 8,
  frames: 4,
  palette: "dungeon16",
  tags: [],
  lastEditor: null,
});

beforeEach(() => {
  api.loadPreferences.mockResolvedValue({ settings: {}, recentProjects: [] });
  api.projectStatus.mockResolvedValue({ status: "saved" });
  api.addRecentProject.mockResolvedValue([]);
  api.closeProject.mockResolvedValue(undefined);
  api.pickFolder.mockResolvedValue("/p/Kit");
  api.openProject.mockResolvedValue({
    name: "Kit",
    path: "/p/Kit",
    damagedFiles: 0,
  });
  core.onProjectChanged.mockResolvedValue(() => {});
  core.spriteFrame.mockResolvedValue({
    revision: 1,
    width: 1,
    height: 1,
    rows: ["."],
    colors: [],
  });
  core.projectTree.mockResolvedValue({
    name: "Kit",
    revision: 1,
    root: {
      name: "",
      path: [],
      sprites: [],
      folders: [
        {
          name: "Personajes",
          path: ["Personajes"],
          folders: [],
          sprites: [sprite("goblin")],
        },
      ],
    },
    palettes: [],
  });
});
afterEach(() => {
  cleanup();
  for (const fn of [
    ...Object.values(api),
    ...Object.values(core),
    ...Object.values(paletteApi),
  ])
    fn.mockReset();
});

describe("Project tab in the app", () => {
  // Checks the spec scenario "Double-click": double-clicking the card
  // "goblin" switches to the Editor tab with that sprite.
  it("opens the Editor tab with the sprite when its card is double-clicked", async () => {
    render(<App />);
    fireEvent.click(
      await screen.findByRole("button", { name: "Open project" }),
    );
    fireEvent.click(
      within(
        await screen.findByRole("treeitem", { name: /Personajes/ }),
      ).getByRole("button", { name: /^Personajes/ }),
    );

    fireEvent.doubleClick(
      await within(screen.getByRole("region", { name: "Sprites" })).findByRole(
        "button",
        { name: /^goblin/ },
      ),
    );

    // The editor opens that sprite. There is no core in this test, so it
    // says that the sprite could not be read.
    expect((await screen.findByRole("alert")).textContent).toContain(
      "The sprite could not be opened",
    );
    expect(
      screen
        .getByRole("button", { name: "Editor" })
        .getAttribute("aria-current"),
    ).toBe("page");
  });

  // Checks that the menu of a card shows the shortcuts of the list of
  // commands of the app (F2, Ctrl+D, Delete): one list for menus and keys.
  it("shows the shortcuts of the commands in the menu of a card", async () => {
    render(<App />);
    fireEvent.click(
      await screen.findByRole("button", { name: "Open project" }),
    );
    fireEvent.click(
      within(
        await screen.findByRole("treeitem", { name: /Personajes/ }),
      ).getByRole("button", { name: /^Personajes/ }),
    );
    await within(screen.getByRole("region", { name: "Sprites" })).findByRole(
      "button",
      {
        name: /^goblin/,
      },
    );

    fireEvent.click(
      screen.getByRole("button", { name: "More actions for goblin" }),
    );

    const menu = await screen.findByRole("menu");
    // The shortcut is the small mono text at the end of the row.
    const hint = (item: string) =>
      within(menu)
        .getByRole("menuitem", { name: new RegExp(`^${item}`) })
        .querySelector(".font-mono")?.textContent;
    expect(hint("Rename")).toBe("F2");
    expect(hint("Duplicate")).toBe("Ctrl+D");
    expect(hint("Delete")).toBe("Delete");
    expect(hint("Open in editor")).toBe("Enter");
  });
});

describe("Palettes in the Project tab", () => {
  async function openProject() {
    core.projectTree.mockResolvedValue({
      name: "Kit",
      revision: 1,
      root: { name: "", path: [], sprites: [], folders: [] },
      palettes: [{ name: "dungeon16", colors: 2 }],
    });
    render(<App />);
    fireEvent.click(
      await screen.findByRole("button", { name: "Open project" }),
    );
  }

  // Checks task 3.1 in the app: the "+" of the palettes list opens the New
  // palette dialog with the presets, and creating from "Farm 32" asks the
  // core for a palette with that preset.
  it("creates a palette from a preset with the plus button", async () => {
    paletteApi.listPresets.mockResolvedValue([
      {
        id: "farm-32",
        name: "Farm 32",
        colors: [{ key: "A", hex: "#000000" }],
      },
    ]);
    paletteApi.createPalette.mockResolvedValue("Farm 32");
    await openProject();
    fireEvent.click(await screen.findByRole("button", { name: "New palette" }));
    fireEvent.click(await screen.findByRole("radio", { name: /Farm 32/ }));
    fireEvent.click(screen.getByRole("button", { name: "Create" }));
    await waitFor(() =>
      expect(paletteApi.createPalette).toHaveBeenCalledWith(
        "Farm 32",
        "farm-32",
      ),
    );
    await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
  });

  // Checks that clicking a palette of the list opens the palette editor for it.
  it("opens the palette editor when a palette is clicked", async () => {
    paletteApi.loadPalette.mockResolvedValue({
      name: "dungeon16",
      colors: [{ key: "R", name: "red", hex: "#F04545", used: false }],
    });
    paletteApi.rampPreview.mockResolvedValue([]);
    await openProject();
    fireEvent.click(await screen.findByRole("button", { name: /dungeon16/ }));
    const dialog = await screen.findByRole("dialog");
    expect(dialog.getAttribute("aria-label")).toBe("Edit palette · dungeon16");
    expect(paletteApi.loadPalette).toHaveBeenCalledWith("dungeon16");
  });
});
