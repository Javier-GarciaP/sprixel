// @vitest-environment jsdom
import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { I18nProvider } from "@/shared/lib/i18n";
import { PaletteEditorDialog } from "./PaletteEditorDialog";
import { FakePaletteApi, sampleColors } from "./testing";

afterEach(cleanup);

async function open() {
  const api = new FakePaletteApi(sampleColors.map((c) => ({ ...c })));
  const onApplied = vi.fn();
  render(
    <I18nProvider language="en">
      <PaletteEditorDialog
        api={api}
        name="dungeon16"
        onClose={vi.fn()}
        onApplied={onApplied}
      />
    </I18nProvider>,
  );
  await screen.findByRole("dialog");
  return { api, onApplied };
}

const section = () => screen.getByRole("region", { name: "Generate ramp" });

describe("Generate ramp", () => {
  // Checks that the base starts with the selected color, that the preview
  // shows one tone per step, and that nothing is written yet.
  it("previews the tones and writes nothing", async () => {
    const { api } = await open();
    expect(
      (screen.getByLabelText("Base color") as HTMLInputElement).value,
    ).toBe("#000000");
    await waitFor(() =>
      expect(section().querySelectorAll("li")).toHaveLength(5),
    );
    expect(api.rampCalls).toHaveLength(0);
    fireEvent.change(screen.getByLabelText("Steps"), {
      target: { value: "7" },
    });
    await waitFor(() =>
      expect(section().querySelectorAll("li")).toHaveLength(7),
    );
  });

  // Checks "Generate ramp": the button sends the base, steps and hue shift,
  // the palette is read again (the new tones are listed) and the editor is
  // told, so the sprite is read again too.
  it("adds the ramp and reads the palette again", async () => {
    const { api, onApplied } = await open();
    fireEvent.change(screen.getByLabelText("Base color"), {
      target: { value: "#3366cc" },
    });
    fireEvent.change(screen.getByLabelText("Hue shift"), {
      target: { value: "20" },
    });
    await waitFor(() =>
      expect(
        (screen.getByRole("button", { name: "Add ramp" }) as HTMLButtonElement)
          .disabled,
      ).toBe(false),
    );
    fireEvent.click(screen.getByRole("button", { name: "Add ramp" }));
    await waitFor(() => expect(api.rampCalls).toHaveLength(1));
    expect(api.rampCalls[0]).toEqual({
      name: "dungeon16",
      base: "#3366CC",
      steps: 5,
      hueShift: 20,
    });
    await waitFor(() => expect(onApplied).toHaveBeenCalled());
    expect(
      await screen.findByRole("button", { name: /^tone0 a/ }),
    ).toBeTruthy();
  });

  // Checks that an edit not saved yet turns the tool off (the ramp goes into
  // the saved palette, and reading it again would lose the edit).
  it("waits for Done when the palette was edited", async () => {
    const { api } = await open();
    fireEvent.change(screen.getByLabelText("Name"), {
      target: { value: "ink" },
    });
    await screen.findByText(/Press Done first/);
    const add = screen.getByRole("button", {
      name: "Add ramp",
    }) as HTMLButtonElement;
    expect(add.disabled).toBe(true);
    expect(api.rampCalls).toHaveLength(0);
  });

  // Checks that a bad base color is shown as an error and the tool stays off.
  it("refuses a base that is not a color", async () => {
    await open();
    fireEvent.change(screen.getByLabelText("Base color"), {
      target: { value: "blue" },
    });
    expect(
      (screen.getByRole("button", { name: "Add ramp" }) as HTMLButtonElement)
        .disabled,
    ).toBe(true);
  });
});
