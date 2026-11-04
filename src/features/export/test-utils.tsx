// Helpers shared by the tests of this feature: a fake core and a project tree.
import { render } from "@testing-library/react";
import { vi } from "vitest";
import { AppProviders } from "@/app/AppProviders";
import type {
  FolderNode,
  ProjectTree,
  SpriteSummary,
  SpriteTarget,
} from "@/shared/lib/core";
import { defaultPreferences } from "@/shared/lib/preferences";
import { ExportScreen } from "./ExportScreen";
import type { ExportApi, Format, Scope, Summary } from "./types";

export const sprite = (
  folder: string[],
  name: string,
  frames = 1,
): SpriteSummary => ({
  name,
  folder,
  width: 8,
  height: 8,
  frames,
  palette: "main",
  tags: [],
  lastEditor: null,
});

const folder = (name: string, sprites: SpriteSummary[]): FolderNode => ({
  name,
  path: [name],
  folders: [],
  sprites,
});

// "Dungeon Kit": the folder "Personajes" with 4 sprites (14 frames) and the
// folder "Enemigos" with 1 sprite, plus 1 sprite in the root. 6 in all.
export const tree: ProjectTree = {
  name: "Dungeon Kit",
  revision: 1,
  root: {
    name: "",
    path: [],
    sprites: [sprite([], "tree")],
    folders: [
      folder("Personajes", [
        sprite(["Personajes"], "hero", 4),
        sprite(["Personajes"], "mage", 4),
        sprite(["Personajes"], "elf", 4),
        sprite(["Personajes"], "kid", 2),
      ]),
      folder("Enemigos", [sprite(["Enemigos"], "orc", 3)]),
    ],
  },
  palettes: [],
};

export const emptyTree: ProjectTree = {
  ...tree,
  root: { name: "", path: [], sprites: [], folders: [] },
};

const countFrames = (scope: Scope, from: ProjectTree) => {
  const all = (f: FolderNode): SpriteSummary[] => [
    ...f.sprites,
    ...f.folders.flatMap(all),
  ];
  const everything = all(from.root);
  const inScope =
    scope.kind === "project"
      ? everything
      : scope.kind === "folder"
        ? everything.filter((s) => s.folder[0] === scope.path[0])
        : everything.filter((s) =>
            scope.sprites.some(
              (t) => t.name === s.name && t.folder.join() === s.folder.join(),
            ),
          );
  return {
    sprites: inScope.length,
    frames: inScope.reduce((sum, s) => sum + s.frames, 0),
  };
};

// A core that answers like the real one for the sprites of `from`: counts
// by scope, 2 files for the sheet and one file per frame for PNG files.
export function fakeApi(from: ProjectTree = tree): ExportApi & {
  summary: ReturnType<typeof vi.fn>;
  preview: ReturnType<typeof vi.fn>;
  run: ReturnType<typeof vi.fn>;
  reveal: ReturnType<typeof vi.fn>;
} {
  return {
    loadTree: vi.fn(async () => from),
    summary: vi.fn(async (scope: Scope, format: Format): Promise<Summary> => {
      const { sprites, frames } = countFrames(scope, from);
      const base = scope.kind === "folder" ? scope.path[0].toLowerCase() : "x";
      return {
        sprites,
        frames,
        files:
          sprites === 0
            ? []
            : format === "sheet"
              ? [`${base}.png`, `${base}.json`]
              : Array.from({ length: frames }, (_, i) => `f_${i + 1}.png`),
        sheetWidth: 71,
        sheetHeight: 17,
      };
    }),
    preview: vi.fn(async () => ({
      width: 2,
      height: 1,
      rgba: new Uint8ClampedArray(8),
    })),
    run: vi.fn(async () => ({
      output: "assets/sprites",
      files: ["personajes.png", "personajes.json"],
    })),
    reveal: vi.fn(async () => undefined),
    onChanged: vi.fn(async () => () => {}),
  };
}

export function renderExport(
  api: ExportApi,
  props: { folder?: string[]; selection?: SpriteTarget[] } = {},
) {
  const onGoToProject = vi.fn();
  render(
    <AppProviders initial={defaultPreferences("en")}>
      <ExportScreen
        api={api}
        folder={props.folder ?? []}
        selection={props.selection ?? []}
        onGoToProject={onGoToProject}
      />
    </AppProviders>,
  );
  return { onGoToProject };
}
