// @vitest-environment jsdom
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { AppProviders } from "@/app/AppProviders";
import { defaultPreferences } from "@/shared/lib/preferences";
import { ResizeDialog } from "./ResizeDialog";

afterEach(cleanup);

function renderDialog(resize = vi.fn().mockResolvedValue(undefined)) {
  const onClose = vi.fn();
  render(
    <AppProviders initial={defaultPreferences("en")}>
      <ResizeDialog
        sprite={{ folder: ["Heroes"], name: "goblin", width: 16, height: 16 }}
        resize={resize}
        onClose={onClose}
      />
    </AppProviders>,
  );
  return { resize, onClose };
}

const type = (label: string, value: string) =>
  fireEvent.change(screen.getByLabelText(label), { target: { value } });
const apply = () => screen.getByRole("button", { name: "Change canvas" });
const enabled = (button: HTMLElement) =>
  !(button as HTMLButtonElement).disabled;

describe("Change canvas dialog", () => {
  // Checks that the dialog starts with the current size and the anchor in the
  // center, and that nothing can be applied while the size is the same.
  it("starts with the current size and the center anchor", () => {
    renderDialog();
    expect((screen.getByLabelText("Width") as HTMLInputElement).value).toBe(
      "16",
    );
    expect((screen.getByLabelText("Height") as HTMLInputElement).value).toBe(
      "16",
    );
    expect(
      screen
        .getByRole("radio", { name: "Center" })
        .getAttribute("aria-checked"),
    ).toBe("true");
    expect(enabled(apply())).toBe(false);
  });

  // Checks the spec scenario "Dialog": a new size and the bottom-right anchor
  // are sent to the core with the sprite, and the dialog closes.
  it("sends the size and the anchor, then closes", async () => {
    const { resize, onClose } = renderDialog();

    type("Width", "24");
    type("Height", "8");
    fireEvent.click(screen.getByRole("radio", { name: "Bottom right" }));
    await vi.waitFor(() => expect(enabled(apply())).toBe(true));
    fireEvent.click(apply());

    await vi.waitFor(() =>
      expect(resize).toHaveBeenCalledWith(24, 8, "bottom-right"),
    );
    await vi.waitFor(() => expect(onClose).toHaveBeenCalled());
  });

  // Checks the limits of the spec: a side must be a whole number from 1 to 512,
  // so 0, 513 and 1.5 show an error and block the button.
  it("blocks sizes outside 1 to 512", async () => {
    renderDialog();
    for (const bad of ["0", "513", "1.5", ""]) {
      type("Width", bad);
      await vi.waitFor(() => expect(enabled(apply())).toBe(false));
      expect(screen.getAllByText(/1 to 512/).length).toBeGreaterThan(0);
    }
    type("Width", "512");
    await vi.waitFor(() => expect(enabled(apply())).toBe(true));
  });

  // Checks that the nine places are there and that the warning about cut
  // pixels and anchors is shown only when the new size is smaller.
  it("has nine anchors and warns when it crops", async () => {
    renderDialog();
    expect(screen.getAllByRole("radio")).toHaveLength(9);
    expect(screen.queryByRole("status")).toBeNull();
    type("Width", "8");
    expect(await screen.findByRole("status")).toBeTruthy();
    expect(screen.getByRole("status").textContent).toMatch(/undo/i);
  });

  // Checks that a failure of the core is shown and the dialog stays open.
  it("shows the error of the core and stays open", async () => {
    const { onClose } = renderDialog(
      vi.fn().mockRejectedValue({ code: "bad_size", message: "no way" }),
    );
    type("Width", "20");
    await vi.waitFor(() => expect(enabled(apply())).toBe(true));
    fireEvent.click(apply());

    expect((await screen.findByRole("alert")).textContent).toContain("no way");
    expect(onClose).not.toHaveBeenCalled();
  });
});
