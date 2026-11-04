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
import { NewPaletteDialog } from "./NewPaletteDialog";
import { FakePaletteApi, sampleColors } from "./testing";

afterEach(cleanup);

function open(language: "en" | "es" = "en") {
  const api = new FakePaletteApi(sampleColors);
  const onClose = vi.fn();
  const onCreated = vi.fn();
  render(
    <I18nProvider language={language}>
      <NewPaletteDialog
        api={api}
        presets={api.presets}
        onCreated={onCreated}
        onClose={onClose}
      />
    </I18nProvider>,
  );
  return { api, onClose, onCreated };
}

const create = () =>
  fireEvent.click(screen.getByRole("button", { name: "Create" }));

describe("New palette dialog", () => {
  // Checks that an empty palette is the first choice and is created with the
  // name the user typed.
  it("creates an empty palette", async () => {
    const { api, onCreated, onClose } = open();
    fireEvent.change(screen.getByLabelText("Name"), {
      target: { value: "Blank" },
    });
    create();
    await waitFor(() => expect(onClose).toHaveBeenCalled());
    expect(api.created).toEqual([{ name: "Blank", preset: null }]);
    expect(onCreated).toHaveBeenCalledWith("Blank");
  });

  // Checks the "Create from a preset" scenario: both presets are offered with
  // their number of colors, and the one chosen is sent with the name that
  // follows the choice until the user types one.
  it("creates a palette from a preset", async () => {
    const { api, onClose } = open();
    expect(screen.getByText("16 colors")).toBeTruthy();
    expect(screen.getByText("32 colors")).toBeTruthy();
    fireEvent.click(screen.getByRole("radio", { name: /Farm 32/ }));
    expect((screen.getByLabelText("Name") as HTMLInputElement).value).toBe(
      "Farm 32",
    );
    create();
    await waitFor(() => expect(onClose).toHaveBeenCalled());
    expect(api.created).toEqual([{ name: "Farm 32", preset: "farm-32" }]);
  });

  // Checks that a name the user typed is not replaced when the choice changes.
  it("keeps the name the user typed", async () => {
    const { api, onClose } = open();
    fireEvent.change(screen.getByLabelText("Name"), {
      target: { value: "Mine" },
    });
    fireEvent.click(screen.getByRole("radio", { name: /Pocket RPG 16/ }));
    create();
    await waitFor(() => expect(onClose).toHaveBeenCalled());
    expect(api.created).toEqual([{ name: "Mine", preset: "pocket-rpg-16" }]);
  });

  // Checks that a name that is already used shows the error and the dialog
  // stays open, and that Create is off with an empty name.
  it("shows the error of the core and stays open", async () => {
    const { api, onClose } = open();
    api.createFailWith = { code: "name_exists" };
    create();
    expect(await screen.findByText(/already exists/)).toBeTruthy();
    expect(onClose).not.toHaveBeenCalled();
    fireEvent.change(screen.getByLabelText("Name"), { target: { value: " " } });
    expect(
      (screen.getByRole("button", { name: "Create" }) as HTMLButtonElement)
        .disabled,
    ).toBe(true);
  });

  // Checks that Cancel creates nothing, and the texts are translated.
  it("is cancelled and translated", () => {
    const { api, onClose } = open("es");
    fireEvent.click(screen.getByRole("button", { name: "Cancelar" }));
    expect(onClose).toHaveBeenCalled();
    expect(api.created).toEqual([]);
    expect(screen.getByText("Paleta vacía")).toBeTruthy();
  });

  // Checks the "Import a hex file" scenario on the screen: the file the user
  // picks is imported with a name that follows the file name, and the preset
  // choice is left.
  it("imports a file chosen with the file picker", async () => {
    const { api, onClose, onCreated } = open();
    fireEvent.click(
      screen.getByRole("button", { name: /Import a .gpl or .hex/ }),
    );
    expect(
      await screen.findByRole("button", { name: /File: sixteen.hex/ }),
    ).toBeTruthy();
    expect((screen.getByLabelText("Name") as HTMLInputElement).value).toBe(
      "sixteen",
    );
    create();
    await waitFor(() => expect(onClose).toHaveBeenCalled());
    expect(api.imported).toEqual([
      { name: "sixteen", path: "/colors/sixteen.hex" },
    ]);
    expect(api.created).toEqual([]);
    expect(onCreated).toHaveBeenCalledWith("sixteen");
  });

  // Checks that a cancelled picker changes nothing, and that choosing a
  // preset after a file goes back to creating, not importing.
  it("goes back to a preset after a file, and ignores a cancelled picker", async () => {
    const { api, onClose } = open();
    api.pickedFile = null;
    fireEvent.click(
      screen.getByRole("button", { name: /Import a .gpl or .hex/ }),
    );
    expect(screen.getByRole("radio", { name: "Empty palette" })).toHaveProperty(
      "checked",
      true,
    );
    api.pickedFile = "C:\\colors\\mine.gpl";
    fireEvent.click(
      screen.getByRole("button", { name: /Import a .gpl or .hex/ }),
    );
    await screen.findByRole("button", { name: /File: mine.gpl/ });
    fireEvent.click(screen.getByRole("radio", { name: /Farm 32/ }));
    create();
    await waitFor(() => expect(onClose).toHaveBeenCalled());
    expect(api.imported).toEqual([]);
    expect(api.created).toEqual([{ name: "Farm 32", preset: "farm-32" }]);
  });

  // Checks that the error of a bad file is shown in the dialog (with the
  // line the core names) and that nothing closes.
  it("shows why a file could not be imported", async () => {
    const { api, onClose } = open();
    fireEvent.click(
      screen.getByRole("button", { name: /Import a .gpl or .hex/ }),
    );
    await screen.findByRole("button", { name: /File: sixteen.hex/ });
    api.createFailWith = {
      code: "bad_import",
      message: "line 2: expected one color",
    };
    create();
    expect(await screen.findByText(/line 2: expected one color/)).toBeTruthy();
    expect(onClose).not.toHaveBeenCalled();
  });
});
