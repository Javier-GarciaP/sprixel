// @vitest-environment jsdom
import { cleanup, render, screen, within } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import { ShortcutsPage } from "@/features/settings";
import { t } from "@/shared/lib/i18n";
import { shortcutGroups } from "./shortcutGroups";

afterEach(cleanup);

describe("shortcuts page", () => {
  // Checks the spec scenario: the page lists the shortcuts grouped by area,
  // for example Undo with Ctrl+Z in "Editor and view" and the Pencil key B.
  it("lists the shortcuts of the registry by area", () => {
    const groups = shortcutGroups("other").map((group) => ({
      title: t("en", group.titleKey),
      rows: group.rows.map((r) => ({
        id: r.id,
        label: t("en", r.labelKey),
        keys: r.keys,
      })),
    }));
    render(<ShortcutsPage groups={groups} />);

    const editor = screen.getByRole("region", { name: "Editor and view" });
    const undo = within(editor).getByText("Undo").closest("li") as HTMLElement;
    expect(undo.textContent).toBe("UndoCtrlZ");

    const tools = screen.getByRole("region", { name: "Tools" });
    const pencil = within(tools)
      .getByText("Pencil")
      .closest("li") as HTMLElement;
    expect(pencil.textContent).toBe("PencilB");
    expect(screen.getByRole("region", { name: "Project" })).toBeTruthy();
  });
});

describe("shortcut rows", () => {
  const render_ = (language: "en" | "es") => {
    const groups = shortcutGroups("other", (key) => t(language, key)).map(
      (group) => ({
        title: t(language, group.titleKey),
        rows: group.rows.map((r) => ({
          id: r.id,
          label: t(language, r.labelKey),
          keys: r.keys,
          alternative: r.alternative,
        })),
      }),
    );
    render(<ShortcutsPage groups={groups} />);
  };

  // Checks that Zoom in shows "+ or Ctrl wheel" (and the Spanish words),
  // like the design.
  it("shows Zoom in with the Ctrl and wheel alternative", () => {
    render_("en");
    const row = screen.getByText("Zoom in").closest("li") as HTMLElement;
    expect(row.textContent).toBe("Zoom in+orCtrlwheel");

    cleanup();
    render_("es");
    const fila = screen.getByText("Acercar").closest("li") as HTMLElement;
    expect(fila.textContent).toBe("Acercar+oCtrlrueda");
  });

  // Checks that the names of the page have no trailing ellipsis while the
  // project menu keeps them (Open... and Export...).
  it("shows Open and Export without an ellipsis", () => {
    render_("en");
    expect(screen.getByText("Open")).toBeTruthy();
    expect(screen.getByText("Export")).toBeTruthy();
    expect(screen.queryByText(/\.\.\.|…/)).toBeNull();
    expect(t("en", "command.project.open")).toBe("Open...");
  });
});
