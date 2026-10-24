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
import { emptyTree, frameOf, sampleTree } from "./fixtures";

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

async function openNewSprite() {
  fireEvent.click(
    (await screen.findAllByRole("button", { name: "New sprite" }))[0],
  );
  return screen.findByRole("dialog", { name: "New sprite" });
}

const type = (label: string, value: string) =>
  fireEvent.change(screen.getByLabelText(label), { target: { value } });

const createButton = () =>
  screen.getByRole("button", { name: "Create sprite" });
const enabled = (button: HTMLElement) =>
  !(button as HTMLButtonElement).disabled;

describe("New sprite dialog", () => {
  // Checks the spec scenario "Custom size": 120 x 30 creates a sprite of that
  // size in the chosen folder, and the sprite is opened in the editor.
  it("creates a sprite with a custom size and opens it", async () => {
    core.createSprite.mockResolvedValue({ folder: [], name: "goblin" });
    const { onOpenSprite } = renderBrowser();
    await openNewSprite();

    type("Name", "goblin");
    type("Width", "120");
    type("Height", "30");
    await vi.waitFor(() => expect(enabled(createButton())).toBe(true));
    fireEvent.click(createButton());

    await vi.waitFor(() =>
      expect(core.createSprite).toHaveBeenCalledWith(
        [],
        "goblin",
        120,
        30,
        null,
      ),
    );
    await vi.waitFor(() =>
      expect(onOpenSprite).toHaveBeenCalledWith({ folder: [], name: "goblin" }),
    );
    await vi.waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
  });

  // Checks the spec scenario "Preset fills the fields": choosing 16x16 shows 16 and 16.
  it("fills the size fields with a preset", async () => {
    renderBrowser();
    await openNewSprite();

    fireEvent.click(screen.getByRole("button", { name: "32x32" }));

    expect((screen.getByLabelText("Width") as HTMLInputElement).value).toBe(
      "32",
    );
    expect((screen.getByLabelText("Height") as HTMLInputElement).value).toBe(
      "32",
    );
    expect(
      screen.getByRole("button", { name: "32x32", pressed: true }),
    ).toBeTruthy();
  });

  // Checks the rule "typing a size clears the selected preset".
  it("clears the preset when a size is typed", async () => {
    renderBrowser();
    await openNewSprite();
    fireEvent.click(screen.getByRole("button", { name: "8x8" }));

    type("Width", "20");

    expect(
      screen.queryByRole("button", { pressed: true, name: /\dx\d/ }),
    ).toBeNull();
  });

  // Checks the spec scenario "A 256x256 sprite": the warning appears and the
  // create button stays enabled. A small size has no warning.
  it("warns about large sizes but still allows them", async () => {
    renderBrowser();
    await openNewSprite();
    type("Name", "boss");
    expect(screen.queryByText(/Large sizes/)).toBeNull();

    type("Width", "256");
    type("Height", "256");

    expect(
      await screen.findByText(
        /Large sizes: drawing with an AI is less accurate/,
      ),
    ).toBeTruthy();
    await vi.waitFor(() => expect(enabled(createButton())).toBe(true));
  });

  // Checks that 127 does not warn but 128 does (the limit is "128 or more").
  it("starts warning at 128", async () => {
    renderBrowser();
    await openNewSprite();

    type("Width", "127");
    expect(screen.queryByText(/Large sizes/)).toBeNull();
    type("Width", "128");

    expect(await screen.findByText(/Large sizes/)).toBeTruthy();
  });

  // Checks that a size above 512, below 1 or not a whole number is refused in the dialog.
  it("refuses sizes outside 1 to 512", async () => {
    renderBrowser();
    await openNewSprite();
    type("Name", "goblin");
    await vi.waitFor(() => expect(enabled(createButton())).toBe(true));

    for (const bad of ["513", "0", "1.5", ""]) {
      type("Width", bad);
      expect(
        await screen.findByText("Use a whole number from 1 to 512"),
      ).toBeTruthy();
      expect(enabled(createButton())).toBe(false);
    }
    type("Width", "512");
    await vi.waitFor(() => expect(enabled(createButton())).toBe(true));
  });

  // Checks the spec "Name validation": a name used by a sprite shows the
  // error and keeps the button disabled; an empty name shows its error only
  // after typing.
  it("shows the name error and disables the button", async () => {
    renderBrowser();
    await openNewSprite();
    expect(screen.queryByText("Type a name")).toBeNull();
    expect(enabled(createButton())).toBe(false);

    type("Name", "ORC");
    expect(
      await screen.findByText("A sprite with that name already exists"),
    ).toBeTruthy();
    expect(enabled(createButton())).toBe(false);
    expect(screen.getByLabelText("Name").getAttribute("aria-invalid")).toBe(
      "true",
    );

    type("Name", "");
    expect(await screen.findByText("Type a name")).toBeTruthy();
  });

  // Checks that a name with a character that is not allowed is refused.
  it("refuses a name with an invalid character", async () => {
    renderBrowser();
    await openNewSprite();

    type("Name", "a/b");

    expect(await screen.findByText(/This name cannot be used/)).toBeTruthy();
    expect(enabled(createButton())).toBe(false);
  });

  // Checks that the folder list offers the project root and every folder,
  // starts on the folder that is selected, and that the choice is sent.
  it("creates the sprite in the chosen folder", async () => {
    core.createSprite.mockResolvedValue({ folder: ["Enemigos"], name: "bat" });
    renderBrowser();
    fireEvent.click(
      within(
        await screen.findByRole("treeitem", { name: /Personajes/ }),
      ).getByRole("button", { name: /^Personajes/ }),
    );
    await openNewSprite();
    const folder = screen.getByLabelText("Folder") as HTMLSelectElement;
    expect(folder.value).toBe("Personajes");

    fireEvent.change(folder, { target: { value: "Enemigos" } });
    type("Name", "bat");
    await vi.waitFor(() => expect(enabled(createButton())).toBe(true));
    fireEvent.click(createButton());

    await vi.waitFor(() =>
      expect(core.createSprite).toHaveBeenCalledWith(
        ["Enemigos"],
        "bat",
        16,
        16,
        null,
      ),
    );
  });

  // Checks that "No palette" is the default and a project palette can be chosen.
  it("lets the person choose a palette of the project", async () => {
    core.createSprite.mockResolvedValue({ folder: [], name: "x" });
    renderBrowser();
    await openNewSprite();
    const palette = screen.getByLabelText("Palette") as HTMLSelectElement;
    expect(palette.value).toBe("");

    fireEvent.change(palette, { target: { value: "dungeon16" } });
    type("Name", "x");
    await vi.waitFor(() => expect(enabled(createButton())).toBe(true));
    fireEvent.click(createButton());

    await vi.waitFor(() =>
      expect(core.createSprite).toHaveBeenCalledWith(
        [],
        "x",
        16,
        16,
        "dungeon16",
      ),
    );
  });

  // Checks that an error of the core on create (for example the name was
  // taken a moment before) shows in the dialog, which stays open.
  it("shows an error from the core and stays open", async () => {
    core.createSprite.mockRejectedValue({
      code: "name_exists",
      message: "taken",
    });
    renderBrowser();
    await openNewSprite();
    type("Name", "goblin");
    await vi.waitFor(() => expect(enabled(createButton())).toBe(true));

    fireEvent.click(createButton());

    expect(await screen.findByRole("alert")).toBeTruthy();
    expect(screen.getByRole("dialog", { name: "New sprite" })).toBeTruthy();
  });

  // Checks that the empty project offers New sprite as its main action.
  it("opens from the empty message of a project without sprites", async () => {
    core.projectTree.mockResolvedValue(emptyTree());
    renderBrowser();

    fireEvent.click(await screen.findByRole("button", { name: "New sprite" }));

    expect(
      await screen.findByRole("dialog", { name: "New sprite" }),
    ).toBeTruthy();
  });
});
