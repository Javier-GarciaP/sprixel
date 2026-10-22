import { describe, expect, it } from "vitest";
import { t } from "@/shared/lib/i18n";
import { nativeMenuCommands, toAccelerator } from "./nativeMenu";
import { projectMenuCommands } from "./registry";

describe("native menu", () => {
  // Checks the spec: the native menu has the same commands, in the same
  // order and with the same names, as the project menu.
  it("has the same commands as the project menu", () => {
    const native = nativeMenuCommands((key) => t("en", key), {});

    expect(native.map((c) => c.id)).toEqual(
      projectMenuCommands.map((c) => c.id),
    );
    expect(native.map((c) => c.label)).toEqual(
      projectMenuCommands.map((c) => t("en", c.labelKey)),
    );
  });

  // Checks that labels follow the language (the menu is rebuilt on a switch).
  it("uses the language of the translate function", () => {
    const native = nativeMenuCommands((key) => t("es", key), {});
    expect(native[0].label).toBe("Nuevo proyecto");
  });

  // Checks that shortcuts become Tauri accelerators and that a command with
  // no handler is sent disabled, like in the project menu.
  it("converts shortcuts and disables commands without a handler", () => {
    expect(toAccelerator("Mod+Shift+Z")).toBe("CmdOrCtrl+Shift+Z");
    const native = nativeMenuCommands((key) => t("en", key), {
      "project.close": () => {},
    });
    expect(native.find((c) => c.id === "project.new")?.accelerator).toBe(
      "CmdOrCtrl+N",
    );
    expect(native.find((c) => c.id === "project.close")?.enabled).toBe(true);
    expect(native.find((c) => c.id === "project.export")?.enabled).toBe(false);
  });
});
