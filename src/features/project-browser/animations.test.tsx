// @vitest-environment jsdom
import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { FramesApi } from "@/features/frames-animations";
import { I18nProvider } from "@/shared/lib/i18n";
import { sprite } from "./fixtures";
import { DetailsPanel } from "./DetailsPanel";
import { FrameCache } from "@/shared/lib/render";

afterEach(cleanup);

function show(
  tags = [{ name: "walk", from: 1, to: 4, fps: 12, mode: "loop" as const }],
) {
  const api = {
    addTag: vi.fn().mockResolvedValue(undefined),
    setTag: vi.fn().mockResolvedValue(undefined),
    removeTag: vi.fn().mockResolvedValue(undefined),
  };
  const goblin = sprite(["Characters"], "goblin", { frames: 7, tags });
  render(
    <I18nProvider language="en">
      <DetailsPanel
        sprite={goblin}
        cache={new FrameCache(() => Promise.reject(new Error("no core")))}
        revision={1}
        onOpen={vi.fn()}
        onRename={vi.fn()}
        onDuplicate={vi.fn()}
        onDelete={vi.fn()}
        moveItems={[]}
        framesApi={api as unknown as FramesApi}
      />
    </I18nProvider>,
  );
  return api;
}

describe("Animations in the sprite details", () => {
  // Checks that an animation shows as its name and "frames 1-4 · 12 fps ·
  // repeats".
  it("lists the animations of the sprite", () => {
    show();
    expect(screen.getByText("walk")).toBeTruthy();
    expect(screen.getByText("frames 1-4 · 12 fps · repeats")).toBeTruthy();
  });

  // Checks the "Create an animation" scenario: "+" opens the dialog, and "walk"
  // from frame 1 to 4 at 12 fps repeating is sent to the core for this sprite.
  it("creates an animation with the plus button", async () => {
    const api = show([]);
    fireEvent.click(screen.getByRole("button", { name: "New animation" }));
    fireEvent.change(await screen.findByLabelText("Name"), {
      target: { value: "idle" },
    });
    fireEvent.change(screen.getByLabelText("Last frame"), {
      target: { value: "2" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Create animation" }));
    await waitFor(() =>
      expect(api.addTag).toHaveBeenCalledWith(
        { folder: ["Characters"], name: "goblin" },
        { name: "idle", from: 1, to: 2, fps: 12, mode: "loop" },
      ),
    );
  });

  // Checks the dialog uses the frames of this sprite: the end frame can be 7
  // and not 8, and the names of the other animations are taken.
  it("checks the animation against the frames and names of the sprite", async () => {
    show();
    fireEvent.click(screen.getByRole("button", { name: "New animation" }));
    fireEvent.change(await screen.findByLabelText("Name"), {
      target: { value: "WALK" },
    });
    expect(await screen.findByText(/already has an animation/)).toBeTruthy();
    fireEvent.change(screen.getByLabelText("Last frame"), {
      target: { value: "8" },
    });
    expect(await screen.findByText("Use a frame from 1 to 7")).toBeTruthy();
  });

  // Checks editing: the dialog starts with the animation, and the changed
  // animation replaces the old one by its name.
  it("edits an animation", async () => {
    const api = show();
    fireEvent.click(screen.getByRole("button", { name: "Edit walk" }));
    const speed = (await screen.findByLabelText(
      "Speed (fps)",
    )) as HTMLInputElement;
    expect(speed.value).toBe("12");
    fireEvent.change(speed, { target: { value: "24" } });
    fireEvent.click(screen.getByRole("button", { name: "Save" }));
    await waitFor(() =>
      expect(api.setTag).toHaveBeenCalledWith(
        { folder: ["Characters"], name: "goblin" },
        "walk",
        { name: "walk", from: 1, to: 4, fps: 24, mode: "loop" },
      ),
    );
  });

  // Checks deleting: the button asks the core to remove that animation.
  it("deletes an animation", async () => {
    const api = show();
    fireEvent.click(screen.getByRole("button", { name: "Delete walk" }));
    await waitFor(() =>
      expect(api.removeTag).toHaveBeenCalledWith(
        { folder: ["Characters"], name: "goblin" },
        "walk",
      ),
    );
  });
});
