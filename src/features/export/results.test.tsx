// @vitest-environment jsdom
import { cleanup, fireEvent, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import { emptyTree, fakeApi, renderExport, tree } from "./test-utils";

afterEach(cleanup);

const exportButton = (name: string) =>
  screen.findByRole("button", { name }) as Promise<HTMLButtonElement>;

// Exports the folder "Personajes" and waits for the answer.
async function exportFolder(api = fakeApi()) {
  const view = renderExport(api, { folder: ["Personajes"] });
  // The button is off until the core has said what would be exported.
  const button = await exportButton("Export");
  await waitFor(() => expect(button.disabled).toBe(false));
  fireEvent.click(button);
  return { api, ...view };
}

describe("Export tab: success", () => {
  // Checks the spec scenario "Success" (screen 26): a message says "2 files saved"
  // with the folder, and the main button now says "Export again".
  it("shows the green message with the files and the folder", async () => {
    await exportFolder();
    const message = await screen.findByRole("status");
    expect(message.textContent).toContain("2 files saved");
    expect(message.textContent).toContain("assets/sprites");
    expect(await exportButton("Export again")).toBeTruthy();
  });

  // Checks screen 26: after a good export the title of the panel says "Export
  // ready" instead of "Preview".
  it("changes the title to Export ready", async () => {
    await exportFolder();
    await screen.findByRole("heading", { name: "Export ready" });
    expect(screen.queryByRole("heading", { name: "Preview" })).toBeNull();
  });

  // Checks that "Open folder" asks the core to open the output folder.
  it("opens the folder from the message", async () => {
    const { api } = await exportFolder();
    fireEvent.click(await screen.findByRole("button", { name: "Open folder" }));
    expect(api.reveal).toHaveBeenCalledWith("assets/sprites");
  });

  // Checks that the message goes away when the options change, because it
  // would describe an export that no longer matches the screen.
  it("hides the message when an option changes", async () => {
    await exportFolder();
    await screen.findByRole("status");
    fireEvent.click(screen.getByRole("button", { name: "2x" }));
    await waitFor(() => expect(screen.queryByRole("status")).toBeNull());
    expect(await exportButton("Export")).toBeTruthy();
  });
});

describe("Export tab: errors", () => {
  const failing = (code: string, message = "boom") => {
    const api = fakeApi();
    api.run.mockRejectedValue({ code, message });
    return api;
  };

  // Checks the spec scenario "No permission" (screen 27): a red message says
  // there is no permission to write in the folder, the options stay as they were
  // and the main button says "Retry".
  it("explains a missing permission and offers Retry", async () => {
    await exportFolder(failing("no_permission"));
    const alert = await screen.findByRole("alert");
    expect(alert.textContent).toContain("Could not export");
    expect(alert.textContent).toContain(
      "There is no permission to write in assets/sprites",
    );
    expect(await exportButton("Retry")).toBeTruthy();
    expect(
      screen.getByRole("button", { name: "Folder Personajes · 4 sprites" }),
    ).toBeTruthy();
  });

  // Checks that Retry runs the same export again, and that a second answer
  // that works replaces the error with the success message.
  it("retries with the same options", async () => {
    const api = failing("no_permission");
    await exportFolder(api);
    api.run.mockResolvedValueOnce({
      output: "assets/sprites",
      files: ["a.png", "a.json"],
    });
    fireEvent.click(await exportButton("Retry"));
    expect((await screen.findByRole("status")).textContent).toContain(
      "2 files saved",
    );
    expect(api.run).toHaveBeenCalledTimes(2);
    expect(screen.queryByRole("alert")).toBeNull();
  });

  // Checks that "Change folder" lets the person edit the output folder (it was
  // read only) and marks it red while the error is shown.
  it("lets the person change the folder after an error", async () => {
    await exportFolder(failing("no_permission"));
    fireEvent.click(
      await screen.findByRole("button", { name: "Change folder" }),
    );
    const output = screen.getByLabelText(
      "Save inside the project in",
    ) as HTMLInputElement;
    expect(output.readOnly).toBe(false);
    expect(output.getAttribute("aria-invalid")).toBe("true");
    await waitFor(() => expect(document.activeElement).toBe(output));
  });

  // Checks the messages of the other reasons: no space, a folder outside the
  // project, an invalid path and an unknown error (its own text).
  it.each([
    ["no_space", "There is no space left to write in assets/sprites"],
    ["outside_project", "The folder must be inside the project"],
    ["invalid_path", "This folder cannot be used: boom"],
    ["io", "boom"],
  ])("explains %s", async (code, text) => {
    await exportFolder(failing(code));
    expect((await screen.findByRole("alert")).textContent).toContain(text);
  });
});

describe("Export tab: nothing to export", () => {
  // Checks the spec scenario "Empty project" (screen 25): the steps are
  // dimmed, Export is disabled and "Go to project" is offered and works.
  it("explains an empty project and offers Go to project", async () => {
    const { onGoToProject } = renderExport(fakeApi(emptyTree));
    expect(await screen.findByText("Nothing to export yet")).toBeTruthy();
    expect(screen.getByText("Create a sprite and come back.")).toBeTruthy();
    expect(((await exportButton("Export")) as HTMLButtonElement).disabled).toBe(
      true,
    );
    expect(
      screen.getByRole("button", { name: "PNG files" }).matches(":disabled"),
    ).toBe(true);
    fireEvent.click(screen.getByRole("button", { name: "Go to project" }));
    expect(onGoToProject).toHaveBeenCalled();
  });

  // Checks that a scope with no sprites (an empty folder) also disables Export
  // and explains, while the steps stay usable so another scope can be chosen.
  it("explains an empty folder but keeps the steps usable", async () => {
    const withEmptyFolder = {
      ...tree,
      root: {
        ...tree.root,
        folders: [{ name: "Vacia", path: ["Vacia"], folders: [], sprites: [] }],
      },
    };
    renderExport(fakeApi(withEmptyFolder), { folder: ["Vacia"] });
    expect(await screen.findByText("Nothing to export yet")).toBeTruthy();
    expect(((await exportButton("Export")) as HTMLButtonElement).disabled).toBe(
      true,
    );
    expect(
      screen.getByRole("button", { name: "PNG files" }).matches(":disabled"),
    ).toBe(false);
  });
});
