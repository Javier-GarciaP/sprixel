// @vitest-environment jsdom
import {
  cleanup,
  fireEvent,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import { fakeApi, renderExport, tree } from "./test-utils";

afterEach(cleanup);

// The button is off until the core has said what would be exported.
async function clickExport() {
  const button = (await screen.findByRole("button", {
    name: "Export",
  })) as HTMLButtonElement;
  await waitFor(() => expect(button.disabled).toBe(false));
  fireEvent.click(button);
}

const card = (name: RegExp) => screen.findByRole("button", { name });

describe("Export tab: what to export", () => {
  // Checks the "Export a folder" scenario: with the folder "Personajes" (4
  // sprites, 14 frames) the card shows its name and count and the tab says
  // "14 frames · 2 files".
  it("shows the counts of a folder", async () => {
    renderExport(fakeApi(), { folder: ["Personajes"] });
    expect(await card(/Folder.*Personajes · 4 sprites/)).toBeTruthy();
    expect(await screen.findByText("14 frames · 2 files")).toBeTruthy();
  });

  // Checks that the project card counts every sprite, and that choosing it
  // changes the counts of the summary.
  it("exports the whole project when it is chosen", async () => {
    renderExport(fakeApi(), { folder: ["Personajes"] });
    fireEvent.click(await card(/Project.*6 sprites/));
    expect(await screen.findByText("18 frames · 2 files")).toBeTruthy();
  });

  // Checks that the selection card is off with nothing selected, and that with a
  // selection it is chosen by default and counts only those sprites.
  it("uses the selection of the Project tab", async () => {
    const api = fakeApi();
    renderExport(api, {
      selection: [{ folder: ["Personajes"], name: "hero" }],
    });
    const chosen = await card(/Selection.*1 chosen/);
    expect(chosen.getAttribute("aria-pressed")).toBe("true");
    expect(await screen.findByText("4 frames · 2 files")).toBeTruthy();
  });

  it("disables the selection card when nothing is selected", async () => {
    renderExport(fakeApi());
    const none = (await card(/Selection.*0 chosen/)) as HTMLButtonElement;
    expect(none.disabled).toBe(true);
  });

  // Checks that a project without folders says so and turns the folder card off.
  it("says when there are no folders", async () => {
    const flat = { ...tree, root: { ...tree.root, folders: [] } };
    renderExport(fakeApi(flat));
    const folder = (await card(/Folder.*No folders/)) as HTMLButtonElement;
    expect(folder.disabled).toBe(true);
  });

  // Checks that the folder can be changed with the list that appears under the cards.
  it("lets the person pick another folder", async () => {
    renderExport(fakeApi(), { folder: ["Personajes"] });
    fireEvent.change(await screen.findByLabelText("Folder to export"), {
      target: { value: "Enemigos" },
    });
    expect(await screen.findByText("3 frames · 2 files")).toBeTruthy();
  });
});

describe("Export tab: format and options", () => {
  // Checks that PNG files lists one file per frame, and that GIF is not built
  // yet (it is off).
  it("switches to PNG files and keeps GIF off", async () => {
    renderExport(fakeApi(), { folder: ["Personajes"] });
    fireEvent.click(await card(/PNG files/));
    expect(await screen.findByText("14 frames · 14 files")).toBeTruthy();
    expect(
      (screen.getByRole("button", { name: /GIF/ }) as HTMLButtonElement)
        .disabled,
    ).toBe(true);
  });

  // Checks that scale, spacing and folder go to the core: with 4x, spacing 2
  // and the folder "out" the Export button sends exactly that.
  it("sends the options when exporting", async () => {
    const api = fakeApi();
    renderExport(api, { folder: ["Personajes"] });
    fireEvent.click(await card(/^4x$/));
    fireEvent.change(screen.getByLabelText("Space between frames"), {
      target: { value: "2" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Change" }));
    fireEvent.change(screen.getByLabelText("Save inside the project in"), {
      target: { value: "out" },
    });
    await clickExport();
    await waitFor(() => expect(api.run).toHaveBeenCalled());
    expect(api.run).toHaveBeenCalledWith(
      { kind: "folder", path: ["Personajes"] },
      "sheet",
      { scale: 4, spacing: 2, naming: null },
      "out",
    );
  });

  // Checks the default output folder of the spec.
  it("starts with the folder assets/sprites and 1x", async () => {
    renderExport(fakeApi(), { folder: ["Personajes"] });
    const output = (await screen.findByLabelText(
      "Save inside the project in",
    )) as HTMLInputElement;
    expect(output.value).toBe("assets/sprites");
    expect(output.readOnly).toBe(true);
    expect((await card(/^1x$/)).getAttribute("aria-pressed")).toBe("true");
  });

  // Checks that the spacing is kept between 0 and 16.
  it("keeps the spacing between 0 and 16", async () => {
    renderExport(fakeApi(), { folder: ["Personajes"] });
    const spacing = (await screen.findByLabelText(
      "Space between frames",
    )) as HTMLInputElement;
    fireEvent.change(spacing, { target: { value: "40" } });
    expect(spacing.value).toBe("16");
    fireEvent.change(spacing, { target: { value: "-3" } });
    expect(spacing.value).toBe("0");
  });

  // Checks that the naming pattern is under "More options" and is sent when
  // changed.
  it("sends a naming pattern from More options", async () => {
    const api = fakeApi();
    renderExport(api, { folder: ["Personajes"] });
    fireEvent.click(
      await screen.findByRole("button", { name: /More options/ }),
    );
    fireEvent.change(screen.getByLabelText("Naming pattern"), {
      target: { value: "{sprite}-{frame}" },
    });
    await clickExport();
    await waitFor(() => expect(api.run).toHaveBeenCalled());
    expect(api.run.mock.calls[0][2].naming).toBe("{sprite}-{frame}");
  });
});

describe("Export tab: preview and files", () => {
  // Checks the preview title line and the file list of the design (screen 24):
  // the size of the sheet, the frames and the two files with their meaning.
  it("shows the sheet size and the files", async () => {
    renderExport(fakeApi(), { folder: ["Personajes"] });
    expect(
      await screen.findByText("sheet of 71 × 17 px · 14 frames"),
    ).toBeTruthy();
    const files = await screen.findByRole("region", { name: "Files" });
    expect(within(files).getByText("personajes.png")).toBeTruthy();
    expect(within(files).getByText("Sheet with 14 frames")).toBeTruthy();
    expect(within(files).getByText("personajes.json")).toBeTruthy();
  });

  // Checks that the preview is asked from the core with the current scope.
  it("asks the core for the preview", async () => {
    const api = fakeApi();
    renderExport(api, { folder: ["Personajes"] });
    await waitFor(() => expect(api.preview).toHaveBeenCalled());
    expect(api.preview.mock.calls[0][0]).toEqual({
      kind: "folder",
      path: ["Personajes"],
    });
  });
});
