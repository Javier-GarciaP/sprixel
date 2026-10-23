// @vitest-environment jsdom
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { defaultPreferences } from "@/shared/lib/preferences";
import { AppProviders } from "../AppProviders";
import { NewProjectDialog, projectFolder } from "./NewProjectDialog";

afterEach(cleanup);

function show(overrides: Partial<Parameters<typeof NewProjectDialog>[0]> = {}) {
  const props = {
    defaultLocation: "/home/ana/Proyectos",
    pickFolder: vi.fn().mockResolvedValue(null),
    onCreate: vi.fn().mockResolvedValue(null),
    onClose: vi.fn(),
    ...overrides,
  };
  render(
    <AppProviders initial={defaultPreferences("en")}>
      <NewProjectDialog {...props} />
    </AppProviders>,
  );
  return props;
}

const type = (label: string, value: string) =>
  fireEvent.change(screen.getByLabelText(label), { target: { value } });

describe("NewProjectDialog", () => {
  // Checks that the dialog has the fields of the design: name, location with
  // the default folder, a palette list with only "No palette", and both buttons.
  it("shows the name, location and palette fields", () => {
    show();
    expect((screen.getByLabelText("Save in") as HTMLInputElement).value).toBe(
      "/home/ana/Proyectos",
    );
    const palette = screen.getByLabelText(
      "Starting palette",
    ) as HTMLSelectElement;
    expect([...palette.options].map((o) => o.text)).toEqual([
      "No palette (empty)",
    ]);
    expect(screen.getByRole("button", { name: "Choose..." })).toBeTruthy();
    expect(screen.getByRole("button", { name: "Cancel" })).toBeTruthy();
  });

  // Checks that the line under the fields shows the folder that will be
  // created and follows the name.
  it("shows the folder that will be created", () => {
    show();
    type("Name", "Dungeon Kit");
    expect(
      screen.getByText(
        "It will be created in /home/ana/Proyectos/Dungeon Kit/",
      ),
    ).toBeTruthy();
  });

  // Checks that the preview uses the separator of the location (Windows
  // paths get backslashes) and does not double a trailing separator.
  it("builds the path with the separator of the location", () => {
    expect(projectFolder("C:\\Users\\Ana\\Proyectos", "Kit")).toBe(
      "C:\\Users\\Ana\\Proyectos\\Kit\\",
    );
    expect(projectFolder("/home/ana/", "Kit")).toBe("/home/ana/Kit/");
  });

  // Checks that Create project stays disabled until the name has text.
  it("disables Create project until there is a name", () => {
    show();
    const create = screen.getByRole("button", {
      name: "Create project",
    }) as HTMLButtonElement;
    expect(create.disabled).toBe(true);
    type("Name", "   ");
    expect(create.disabled).toBe(true);
    type("Name", "Kit");
    expect(create.disabled).toBe(false);
  });

  // Checks that Choose... uses the native picker and puts the folder in the field.
  it("takes the location from the folder picker", async () => {
    show({ pickFolder: vi.fn().mockResolvedValue("/mnt/games") });
    fireEvent.click(screen.getByRole("button", { name: "Choose..." }));
    await vi.waitFor(() =>
      expect((screen.getByLabelText("Save in") as HTMLInputElement).value).toBe(
        "/mnt/games",
      ),
    );
  });

  // Checks that a successful create sends location and name and closes the dialog.
  it("creates the project and closes on success", async () => {
    const props = show();
    type("Name", "Dungeon Kit");
    fireEvent.click(screen.getByRole("button", { name: "Create project" }));
    await vi.waitFor(() => expect(props.onClose).toHaveBeenCalled());
    expect(props.onCreate).toHaveBeenCalledWith(
      "/home/ana/Proyectos",
      "Dungeon Kit",
    );
  });

  // Checks the spec scenario: when the folder exists the dialog shows the
  // translated error and stays open.
  it("shows the folder exists error and stays open", async () => {
    const props = show({
      onCreate: vi.fn().mockResolvedValue({ code: "folder_exists" }),
    });
    type("Name", "Kit");
    fireEvent.click(screen.getByRole("button", { name: "Create project" }));
    expect(
      await screen.findByText("A folder with this name already exists here."),
    ).toBeTruthy();
    expect(props.onClose).not.toHaveBeenCalled();
  });

  // Checks that an invalid name shows its translated reason, and that any
  // other failure shows the system message.
  it("shows the invalid name and io errors", async () => {
    const props = show({
      onCreate: vi
        .fn()
        .mockResolvedValueOnce({ code: "invalid_name", message: "x" })
        .mockResolvedValueOnce({ code: "io", message: "disk is full" }),
    });
    type("Name", "a/b");
    fireEvent.click(screen.getByRole("button", { name: "Create project" }));
    expect(
      await screen.findByText("This name cannot be used as a folder name."),
    ).toBeTruthy();

    type("Name", "ab");
    fireEvent.click(screen.getByRole("button", { name: "Create project" }));
    expect((await screen.findByRole("alert")).textContent).toContain(
      "disk is full",
    );
    expect(props.onClose).not.toHaveBeenCalled();
  });
});
