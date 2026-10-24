// @vitest-environment jsdom
import {
  cleanup,
  fireEvent,
  render,
  screen,
  within,
} from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

const api = vi.hoisted(() => ({
  createProject: vi.fn(),
  openProject: vi.fn(),
  closeProject: vi.fn(),
  projectStatus: vi.fn(),
  retrySave: vi.fn(),
  pickFolder: vi.fn(),
  defaultProjectLocation: vi.fn(),
  addRecentProject: vi.fn(),
  removeRecentProject: vi.fn(),
  folderExists: vi.fn(),
  loadPreferences: vi.fn(),
  saveSettings: vi.fn(),
}));

vi.mock("./api/project", async (original) => ({
  ...(await original<typeof import("./api/project")>()),
  createProject: api.createProject,
  openProject: api.openProject,
  closeProject: api.closeProject,
  projectStatus: api.projectStatus,
  retrySave: api.retrySave,
  pickFolder: api.pickFolder,
  defaultProjectLocation: api.defaultProjectLocation,
}));
vi.mock("./api/preferences", async (original) => ({
  ...(await original<typeof import("./api/preferences")>()),
  addRecentProject: api.addRecentProject,
  removeRecentProject: api.removeRecentProject,
  folderExists: api.folderExists,
  loadPreferences: api.loadPreferences,
  saveSettings: api.saveSettings,
}));

vi.mock("@/shared/lib/core", async (original) => ({
  ...(await original<typeof import("@/shared/lib/core")>()),
  projectTree: vi.fn().mockResolvedValue({
    name: "Dungeon Kit",
    revision: 0,
    root: { name: "", path: [], folders: [], sprites: [] },
    palettes: [],
  }),
  onProjectChanged: vi.fn().mockResolvedValue(() => {}),
}));

import App from "./App";

const kit = {
  name: "Dungeon Kit",
  path: "/home/ana/Proyectos/Dungeon Kit",
  damagedFiles: 0,
};
const recents = [{ path: "/p/forest-run", name: "Forest Run" }];

beforeEach(() => {
  for (const fn of Object.values(api)) fn.mockReset();
  api.loadPreferences.mockResolvedValue({
    settings: {},
    recentProjects: recents,
  });
  api.defaultProjectLocation.mockResolvedValue("/home/ana/Proyectos");
  api.projectStatus.mockResolvedValue({ status: "saved" });
  api.addRecentProject.mockResolvedValue([
    { path: kit.path, name: kit.name },
    ...recents,
  ]);
  api.folderExists.mockResolvedValue(true);
  api.closeProject.mockResolvedValue(undefined);
});
afterEach(cleanup);

async function openNewProjectDialog() {
  render(<App />);
  fireEvent.click(await screen.findByRole("button", { name: /New project/ }));
  return screen.findByRole("dialog", { name: "New project" });
}

const create = (name: string) => {
  fireEvent.change(screen.getByLabelText("Name"), { target: { value: name } });
  fireEvent.click(screen.getByRole("button", { name: "Create project" }));
};

const tabDisabled = (name: string) =>
  (screen.getByRole("button", { name }) as HTMLButtonElement).disabled;

describe("creating a project", () => {
  // Checks the spec scenario end to end in the UI: the dialog creates the
  // project in the chosen folder, closes, opens it in the Project tab with
  // its name and save status in the top bar, enables the tabs and adds it to
  // the recent list.
  it("creates, closes the dialog and opens the project", async () => {
    api.createProject.mockResolvedValue(kit);
    await openNewProjectDialog();
    create("Dungeon Kit");

    expect(
      await screen.findByRole("button", { name: /Dungeon Kit/ }),
    ).toBeTruthy();
    expect(api.createProject).toHaveBeenCalledWith(
      "/home/ana/Proyectos",
      "Dungeon Kit",
    );
    expect(screen.queryByRole("dialog")).toBeNull();
    expect(screen.getByText("Saved")).toBeTruthy();
    // The Project tab shows the project from the core (here an empty one).
    expect(await screen.findByText("No folders yet")).toBeTruthy();
    for (const tab of ["Editor", "Export", "Agents"])
      expect(tabDisabled(tab)).toBe(false);
    expect(api.addRecentProject).toHaveBeenCalledWith(kit.path, kit.name);
  });

  // Checks the spec scenario: when the folder exists the dialog shows the
  // error, stays open, and no project is opened (tabs stay disabled).
  it("shows the folder exists error and opens nothing", async () => {
    api.createProject.mockRejectedValue({ code: "folder_exists" });
    await openNewProjectDialog();
    create("Dungeon Kit");

    expect(
      await screen.findByText("A folder with this name already exists here."),
    ).toBeTruthy();
    expect(screen.getByRole("dialog")).toBeTruthy();
    expect(tabDisabled("Editor")).toBe(true);
    expect(api.addRecentProject).not.toHaveBeenCalled();
  });

  // Checks that the window title becomes "<name> - Sprixel" once opened.
  it("sets the window title to the project name", async () => {
    api.createProject.mockResolvedValue(kit);
    await openNewProjectDialog();
    // The title is set in an effect, which runs just after the screen changes,
    // so the checks wait for it instead of reading it at once.
    await vi.waitFor(() => expect(document.title).toBe("Sprixel"));
    create("Dungeon Kit");
    await vi.waitFor(() =>
      expect(document.title).toBe("Dungeon Kit - Sprixel"),
    );
  });
});

describe("opening a project", () => {
  // Checks that Open project asks for a folder with the native picker and
  // opens what the core returns.
  it("opens the folder chosen in the picker", async () => {
    api.pickFolder.mockResolvedValue(kit.path);
    api.openProject.mockResolvedValue(kit);
    render(<App />);
    fireEvent.click(
      await screen.findByRole("button", { name: "Open project" }),
    );

    expect(
      await screen.findByRole("button", { name: /Dungeon Kit/ }),
    ).toBeTruthy();
    expect(api.openProject).toHaveBeenCalledWith(kit.path);
  });

  // Checks the spec scenario: a folder without sprixel.json shows the message
  // and nothing is opened.
  it("shows a message when the folder is not a project", async () => {
    api.pickFolder.mockResolvedValue("/tmp/other");
    api.openProject.mockRejectedValue({ code: "not_a_project" });
    render(<App />);
    fireEvent.click(
      await screen.findByRole("button", { name: "Open project" }),
    );

    expect((await screen.findByRole("status")).textContent).toBe(
      "This folder is not a Sprixel project",
    );
    expect(tabDisabled("Editor")).toBe(true);
    expect(api.addRecentProject).not.toHaveBeenCalled();
  });

  // Checks that nothing happens when the user cancels the folder picker.
  it("does nothing when the picker is cancelled", async () => {
    api.pickFolder.mockResolvedValue(null);
    render(<App />);
    fireEvent.click(
      await screen.findByRole("button", { name: "Open project" }),
    );
    await vi.waitFor(() => expect(api.pickFolder).toHaveBeenCalled());
    expect(api.openProject).not.toHaveBeenCalled();
  });

  // Checks the spec scenario left open in task 5.1: clicking a recent project
  // opens it.
  it("opens a recent project when it is clicked", async () => {
    api.openProject.mockResolvedValue({
      name: "Forest Run",
      path: "/p/forest-run",
      damagedFiles: 0,
    });
    render(<App />);
    fireEvent.click(await screen.findByText("Forest Run"));

    expect(
      await screen.findByRole("button", { name: /Forest Run/ }),
    ).toBeTruthy();
    expect(api.openProject).toHaveBeenCalledWith("/p/forest-run");
  });

  // Checks that a recent project whose folder was moved is not opened and
  // keeps the "folder not found" message of the Welcome screen.
  it("keeps the moved folder message", async () => {
    api.folderExists.mockResolvedValue(false);
    render(<App />);
    fireEvent.click(await screen.findByText("Forest Run"));

    expect((await screen.findByRole("alert")).textContent).toContain(
      "Folder not found: /p/forest-run",
    );
    expect(api.openProject).not.toHaveBeenCalled();
  });

  // Checks that the Open recent submenu of the project menu opens a recent project.
  it("opens a recent project from the project menu", async () => {
    api.createProject.mockResolvedValue(kit);
    api.openProject.mockResolvedValue({
      name: "Forest Run",
      path: "/p/forest-run",
      damagedFiles: 0,
    });
    await openNewProjectDialog();
    create("Dungeon Kit");
    fireEvent.click(await screen.findByRole("button", { name: /Dungeon Kit/ }));
    fireEvent.click(screen.getByRole("menuitem", { name: /Open recent/ }));
    fireEvent.click(
      within(screen.getAllByRole("menu")[1]).getByText("Forest Run"),
    );

    expect(
      await screen.findByRole("button", { name: /Forest Run/ }),
    ).toBeTruthy();
    expect(api.openProject).toHaveBeenCalledWith("/p/forest-run");
  });
});

describe("shortcuts and closing", () => {
  // Checks that Ctrl+N opens the New project dialog and Ctrl+O the folder picker.
  it("Ctrl+N opens the dialog and Ctrl+O the picker", async () => {
    api.pickFolder.mockResolvedValue(null);
    render(<App />);
    await screen.findByRole("heading", { level: 1 });

    // The key listeners are attached in effects that run just after the screen
    // changes. On a slow machine a key can arrive before them, so each key is
    // sent again until its effect is visible.
    await vi.waitFor(() => {
      fireEvent.keyDown(window, { key: "n", ctrlKey: true });
      expect(screen.getByRole("dialog", { name: "New project" })).toBeTruthy();
    });
    await vi.waitFor(() => {
      fireEvent.keyDown(window, { key: "Escape" });
      expect(screen.queryByRole("dialog")).toBeNull();
    });

    fireEvent.keyDown(window, { key: "o", ctrlKey: true });
    await vi.waitFor(() => expect(api.pickFolder).toHaveBeenCalled());
  });

  // Checks that Close project in the menu closes it in the core and returns
  // to the Welcome screen with the tabs disabled and the plain window title.
  it("closes the project and returns to the Welcome screen", async () => {
    api.createProject.mockResolvedValue(kit);
    await openNewProjectDialog();
    create("Dungeon Kit");
    fireEvent.click(await screen.findByRole("button", { name: /Dungeon Kit/ }));
    fireEvent.click(screen.getByRole("menuitem", { name: "Close project" }));

    expect(await screen.findByRole("heading", { level: 1 })).toBeTruthy();
    expect(api.closeProject).toHaveBeenCalledTimes(1);
    expect(tabDisabled("Editor")).toBe(true);
    await vi.waitFor(() => expect(document.title).toBe("Sprixel"));
  });
});

describe("save status", () => {
  // Checks the spec scenario: when saving fails the top bar shows "Error
  // saving", the message shows the reason, and Retry asks the core to save again.
  it("shows the save error with its reason and retries", async () => {
    api.createProject.mockResolvedValue(kit);
    api.projectStatus.mockResolvedValue({
      status: "error",
      reason: "disk is full",
    });
    api.retrySave.mockResolvedValue({ status: "saved" });
    await openNewProjectDialog();
    create("Dungeon Kit");

    expect(await screen.findByText("Error saving")).toBeTruthy();
    expect(screen.getByRole("status").textContent).toContain("disk is full");

    fireEvent.click(screen.getAllByRole("button", { name: "Retry" })[0]);
    await vi.waitFor(() => expect(api.retrySave).toHaveBeenCalledTimes(1));
    expect(await screen.findByText("Saved")).toBeTruthy();
  });
});
