// @vitest-environment jsdom
import {
  act,
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

// The app talks to the agents through this fake core.
const fake = vi.hoisted(() => ({ api: null as unknown }));
vi.mock("@/features/agents", async (original) => {
  const real = await original<typeof import("@/features/agents")>();
  const { FakeAgentsApi } = await import("@/features/agents/testing");
  fake.api = new FakeAgentsApi();
  return { ...real, coreAgentsApi: fake.api };
});
vi.mock("./api/project", async (original) => ({
  ...(await original<typeof import("./api/project")>()),
  openProject: vi.fn().mockResolvedValue({
    name: "Dungeon Kit",
    path: "/p/kit",
    damagedFiles: 0,
  }),
  projectStatus: vi.fn().mockResolvedValue({ status: "saved" }),
  pickFolder: vi.fn().mockResolvedValue("/p/kit"),
}));
vi.mock("./api/preferences", async (original) => ({
  ...(await original<typeof import("./api/preferences")>()),
  loadPreferences: vi
    .fn()
    .mockResolvedValue({ settings: { language: "en" }, recentProjects: [] }),
  addRecentProject: vi.fn().mockResolvedValue([]),
  saveSettings: vi.fn(),
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

import type { FakeAgentsApi as Fake } from "@/features/agents/testing";
import { client } from "@/features/agents/testing";
import App from "./App";

const agents = () => fake.api as Fake;

beforeEach(() => {
  const api = agents();
  api.clientList = [];
  api.calls = [];
});
afterEach(cleanup);

async function openProject() {
  render(<App />);
  fireEvent.click(await screen.findByRole("button", { name: /Open/ }));
  await screen.findByText("No folders yet");
  await waitFor(() => expect(agents().listening).toBe(true));
}

describe("Agents in the app", () => {
  // Checks that the approval prompt is global: it appears over the Project
  // tab, the top bar waits, and Allow ends with the agent connected in the
  // top bar and in the Agents tab.
  it("asks to approve from any tab and connects on Allow", async () => {
    await openProject();
    const api = agents();
    act(() => {
      api.clientList = [client("Claude Code", "pending")];
      api.emit({ type: "pending", name: "Claude Code", version: "1.0" });
      api.emit({ type: "clients" });
    });

    const dialog = await screen.findByRole("dialog", {
      name: "Claude Code wants to connect",
    });
    expect(screen.getByText("Waiting for approval")).toBeTruthy();
    fireEvent.click(within(dialog).getByRole("button", { name: "Allow" }));

    await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
    const header = screen.getByRole("banner");
    await within(header).findByText("Claude Code");
    fireEvent.click(screen.getByRole("button", { name: "Agents" }));
    const card = await screen.findByRole("region", { name: "Claude Code" });
    expect(within(card).getByText("Connected")).toBeTruthy();
  });

  // Checks that `open_project` from an agent is shown like an open by the
  // person: the new project is the open one and enters the recent list.
  it("shows the project an agent opened and adds it to the recent list", async () => {
    await openProject();
    const { addRecentProject } = await import("./api/preferences");
    act(() =>
      agents().emit({
        type: "project-opened",
        project: { name: "Other Kit", path: "/p/other" },
      }),
    );
    await waitFor(() =>
      expect(addRecentProject).toHaveBeenCalledWith("/p/other", "Other Kit"),
    );
    await waitFor(() => expect(document.title).toContain("Other Kit"));
  });

  // Checks that `open_sprite` from an agent shows the sprite in the Editor.
  it("opens the sprite an agent asks for in the Editor", async () => {
    await openProject();
    act(() =>
      agents().emit({
        type: "open-sprite",
        sprite: { folder: ["Heroes"], name: "goblin" },
      }),
    );
    await waitFor(() =>
      expect(
        screen
          .getByRole("button", { name: "Editor" })
          .getAttribute("aria-current"),
      ).toBe("page"),
    );
  });
});
