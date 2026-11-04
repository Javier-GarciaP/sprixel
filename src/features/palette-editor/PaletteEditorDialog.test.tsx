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
import type { PaletteEntry } from "./types";

afterEach(cleanup);

async function open(
  colors: PaletteEntry[] = sampleColors,
  language: "en" | "es" = "en",
  agentBusy = false,
) {
  const api = new FakePaletteApi(colors);
  const onClose = vi.fn();
  const onApplied = vi.fn();
  render(
    <I18nProvider language={language}>
      <PaletteEditorDialog
        api={api}
        name="dungeon16"
        agentBusy={agentBusy}
        onClose={onClose}
        onApplied={onApplied}
      />
    </I18nProvider>,
  );
  await screen.findByRole("dialog");
  return { api, onClose, onApplied };
}

const row = (name: string) =>
  screen.getByRole("button", { name: new RegExp(`^${name} `) });
const type = (label: string, value: string) =>
  fireEvent.change(screen.getByLabelText(label), { target: { value } });
const done = () =>
  fireEvent.click(screen.getByRole("button", { name: "Done" }));

describe("Palette editor dialog", () => {
  // Checks that the dialog lists the colors of the palette with their key,
  // name and value, and shows the first color as the selected one.
  it("lists the colors and selects the first", async () => {
    await open();
    expect(screen.getByRole("dialog").getAttribute("aria-label")).toBe(
      "Edit palette · dungeon16",
    );
    expect(row("red").textContent).toContain("#F04545");
    expect((screen.getByLabelText("Name") as HTMLInputElement).value).toBe(
      "black",
    );
  });

  // Checks the "Cancel" scenario: after changing two colors, Cancel closes
  // the dialog and nothing is sent to the core.
  it("discards every change on Cancel", async () => {
    const { api, onClose, onApplied } = await open();
    type("Color (hex)", "#111111");
    fireEvent.click(row("red"));
    type("Name", "crimson");
    fireEvent.click(screen.getByRole("button", { name: "Cancel" }));
    expect(onClose).toHaveBeenCalledTimes(1);
    expect(onApplied).not.toHaveBeenCalled();
    expect(api.applied).toEqual([]);
  });

  // Checks that Escape and the close button also discard the changes.
  it("discards the changes on Escape", async () => {
    const { api, onClose } = await open();
    type("Color (hex)", "#111111");
    fireEvent.keyDown(window, { key: "Escape" });
    expect(onClose).toHaveBeenCalled();
    expect(api.applied).toEqual([]);
  });

  // Checks the "Change a color" scenario: the new value goes to the core on
  // Done, in one call, with the key the color had at the start.
  it("sends the whole edit as one call on Done", async () => {
    const { api, onClose, onApplied } = await open();
    fireEvent.click(row("red"));
    type("Color (hex)", "#d03030");
    type("Name", "crimson");
    done();
    await waitFor(() => expect(onClose).toHaveBeenCalled());
    expect(onApplied).toHaveBeenCalledTimes(1);
    expect(api.applied).toHaveLength(1);
    const sent = api.applied[0];
    expect(sent.name).toBe("dungeon16");
    expect(sent.colors.map((c) => [c.from, c.key, c.name, c.hex])).toEqual([
      ["N", "N", "black", "#000000"],
      ["R", "R", "crimson", "#D03030"],
      ["V", "V", "green", "#4ADE80"],
    ]);
    expect(sent.replacements).toEqual([]);
  });

  // Checks the RGB sliders: moving the red slider changes the hex field and
  // the big preview of the selected color.
  it("changes the color with the RGB sliders", async () => {
    await open();
    fireEvent.click(row("red"));
    fireEvent.change(screen.getByLabelText("Red"), {
      target: { value: "208" },
    });
    expect(
      (screen.getByLabelText("Color (hex)") as HTMLInputElement).value,
    ).toBe("#D04545");
    expect(screen.getByTestId("selected-color").style.background).toBe(
      "rgb(208, 69, 69)",
    );
    expect((screen.getByLabelText("Red") as HTMLInputElement).value).toBe(
      "208",
    );
  });

  // Checks that a hex that is not #RRGGBB shows an error and is not applied.
  it("refuses an invalid hex", async () => {
    const { api } = await open();
    type("Color (hex)", "#12");
    // More time than the default: the machine can be busy (the CI, a stress run).
    const alert = await screen.findByRole("alert", {}, { timeout: 4000 });
    expect(alert.textContent).toContain("#F04545");
    done();
    await waitFor(() => expect(api.applied).toHaveLength(1));
    expect(api.applied[0].colors[0].hex).toBe("#000000");
  });

  // Checks the "Rename the key R to H" scenario: the warning names the old
  // key, and the core receives the old key as `from` so it can rewrite pixels.
  it("warns when a key changes and sends the old key", async () => {
    const { api } = await open();
    expect(screen.queryByText(/text grids/)).toBeNull();
    fireEvent.click(row("red"));
    type("Key (1 letter)", "H");
    expect(
      (await screen.findByText(/Changing the key "R"/)).textContent,
    ).toContain("invalid");
    done();
    await waitFor(() => expect(api.applied).toHaveLength(1));
    expect(api.applied[0].colors[1]).toMatchObject({ from: "R", key: "H" });
  });

  // Checks the "Reserved key" scenario: "." is refused with an explanation
  // that it means transparent, and the color keeps its key.
  it('refuses "." as a key and explains it', async () => {
    const { api } = await open();
    fireEvent.click(row("red"));
    type("Key (1 letter)", ".");
    expect((await screen.findByRole("alert")).textContent).toContain(
      "transparent",
    );
    done();
    await waitFor(() => expect(api.applied).toHaveLength(1));
    expect(api.applied[0].colors[1].key).toBe("R");
  });

  // Checks the "Duplicate key" scenario: a key another color uses shows an
  // error and the change is not applied.
  it("refuses a key that another color uses", async () => {
    const { api } = await open();
    fireEvent.click(row("red"));
    type("Key (1 letter)", "V");
    expect((await screen.findByRole("alert")).textContent).toContain(
      "already uses this key",
    );
    done();
    await waitFor(() => expect(api.applied).toHaveLength(1));
    expect(api.applied[0].colors[1].key).toBe("R");
  });

  // Checks that a name another color has is refused, and an empty one too.
  it("refuses a repeated or empty name", async () => {
    await open();
    fireEvent.click(row("red"));
    type("Name", "Green");
    expect((await screen.findByRole("alert")).textContent).toContain(
      "already has this name",
    );
    type("Name", " ");
    await waitFor(() =>
      expect(screen.getByRole("alert").textContent).toContain("needs a name"),
    );
  });

  // Checks "Add color": the new color gets the next free key, can be edited
  // and is sent with no original key.
  it("adds a color with a free key", async () => {
    const { api } = await open();
    fireEvent.click(screen.getByRole("button", { name: "Add color" }));
    expect(
      (screen.getByLabelText("Key (1 letter)") as HTMLInputElement).value,
    ).toBe("A");
    type("Color (hex)", "#F5C542");
    done();
    await waitFor(() => expect(api.applied).toHaveLength(1));
    expect(api.applied[0].colors[3]).toMatchObject({
      from: null,
      key: "A",
      hex: "#F5C542",
    });
  });

  // Checks the limit: with 93 colors the Add button is off.
  it("cannot add a color to a full palette", async () => {
    const keys = [
      ..."!\"#$%&'()*+,-/0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~",
    ];
    expect(keys).toHaveLength(93);
    await open(
      keys.map((key, i) => ({
        key,
        name: `c${i}`,
        hex: "#000000",
        used: false,
      })),
    );
    expect(
      (screen.getByRole("button", { name: "Add color" }) as HTMLButtonElement)
        .disabled,
    ).toBe(true);
  });

  // Checks that a color no sprite uses is removed at once, with no question.
  it("removes an unused color without a replacement", async () => {
    const { api } = await open();
    fireEvent.click(row("green"));
    fireEvent.click(screen.getByRole("button", { name: "Remove color" }));
    expect(screen.queryByRole("button", { name: /^green / })).toBeNull();
    done();
    await waitFor(() => expect(api.applied).toHaveLength(1));
    expect(api.applied[0].colors.map((c) => c.key)).toEqual(["N", "R"]);
    expect(api.applied[0].replacements).toEqual([]);
  });

  // Checks the "Remove a used color" scenario: the editor asks for a
  // replacement, and the choice is sent with the edit.
  it("asks for a replacement when a used color is removed", async () => {
    const { api } = await open();
    fireEvent.click(row("red"));
    fireEvent.click(screen.getByRole("button", { name: "Remove color" }));
    // Nothing is removed until the replacement is chosen.
    expect(row("red")).toBeTruthy();
    fireEvent.change(screen.getByLabelText("Replace with"), {
      target: { value: "V" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Replace and remove" }));
    expect(screen.queryByRole("button", { name: /^red / })).toBeNull();
    done();
    await waitFor(() => expect(api.applied).toHaveLength(1));
    expect(api.applied[0].replacements).toEqual([{ key: "R", with: "V" }]);
    expect(api.applied[0].colors.map((c) => c.from)).toEqual(["N", "V"]);
  });

  // Checks that the replacement can be transparent and that the question can
  // be cancelled (the color stays).
  it("can replace with transparent, and the question can be cancelled", async () => {
    const { api } = await open();
    fireEvent.click(row("red"));
    fireEvent.click(screen.getByRole("button", { name: "Remove color" }));
    fireEvent.click(screen.getAllByRole("button", { name: "Cancel" })[0]);
    expect(row("red")).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "Remove color" }));
    fireEvent.click(screen.getByRole("button", { name: "Replace and remove" }));
    done();
    await waitFor(() => expect(api.applied).toHaveLength(1));
    expect(api.applied[0].replacements).toEqual([{ key: "R", with: "." }]);
  });

  // Checks that an error of the core is shown in the dialog, in the language
  // of the user, and the dialog stays open so nothing is lost.
  it("shows the error of the core and stays open", async () => {
    const { api, onClose } = await open();
    api.failWith = { code: "duplicate_key" };
    done();
    expect((await screen.findAllByRole("alert"))[0].textContent).toContain(
      "already uses this key",
    );
    expect(onClose).not.toHaveBeenCalled();
  });

  // Checks that the texts come from the translation files (Spanish here).
  it("is translated", async () => {
    await open(sampleColors, "es");
    expect(screen.getByRole("button", { name: "Hecho" })).toBeTruthy();
    expect(screen.getByLabelText("Clave (1 letra)")).toBeTruthy();
  });

  // Checks the "Agent is drawing" scenario: while an agent writes, changing a
  // key shows the message (wait or stop the agent) and the key does not
  // change, but the color value can still be changed.
  it("blocks key changes while an agent writes", async () => {
    const { api } = await open(sampleColors, "en", true);
    fireEvent.click(row("red"));
    type("Key (1 letter)", "H");
    expect((await screen.findByRole("alert")).textContent).toContain(
      "Wait for it to finish or stop it first",
    );
    expect(screen.queryByText(/text grids/)).toBeNull();
    type("Color (hex)", "#D03030");
    done();
    await waitFor(() => expect(api.applied).toHaveLength(1));
    expect(api.applied[0].colors[1]).toMatchObject({
      from: "R",
      key: "R",
      hex: "#D03030",
    });
  });

  // Checks that a used color cannot be removed while an agent writes (its
  // pixels would be rewritten), but an unused one can.
  it("blocks removing a used color while an agent writes", async () => {
    await open(sampleColors, "en", true);
    fireEvent.click(row("red"));
    fireEvent.click(screen.getByRole("button", { name: "Remove color" }));
    expect((await screen.findByRole("alert")).textContent).toContain(
      "Wait for it",
    );
    expect(screen.queryByLabelText("Replace with")).toBeNull();
    fireEvent.click(row("green"));
    fireEvent.click(screen.getByRole("button", { name: "Remove color" }));
    expect(screen.queryByRole("button", { name: /^green / })).toBeNull();
  });

  // Checks that the message of the core is shown too (the agent can start
  // after the dialog opened).
  it("shows the agent message when the core refuses the edit", async () => {
    const { api, onClose } = await open();
    api.failWith = { code: "agent_writing" };
    done();
    expect((await screen.findAllByRole("alert"))[0].textContent).toContain(
      "Wait for it to finish",
    );
    expect(onClose).not.toHaveBeenCalled();
  });
});
