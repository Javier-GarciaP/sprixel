import { useCallback, useState } from "react";
import { configure, render } from "@testing-library/react";
import { vi } from "vitest";
import { useShortcuts } from "@/app/commands/useShortcuts";
import { I18nProvider } from "@/shared/lib/i18n";
import {
  PreferencesProvider,
  defaultPreferences,
} from "@/shared/lib/preferences";
import { EditorScreen } from "../components/EditorScreen";
import type { CommandHandlers } from "../hooks/useEditorCommands";
import type { EditorApi, SpriteRef } from "../api/types";
import { centeredView, fitZoom } from "../view/viewMath";

export const goblin: SpriteRef = { folder: ["Characters"], name: "goblin" };

// The size of the canvas area and the screen pictures sent to the canvas, for
// the tests: jsdom has no layout and no canvas.
export const putImageData = vi.fn();

export function mockLayout(width = 500, height = 300) {
  // Many test files run at once, so a slow machine needs more than 1 second.
  configure({ asyncUtilTimeout: 5000 });
  putImageData.mockClear();
  vi.stubGlobal(
    "ImageData",
    class {
      constructor(
        public data: Uint8ClampedArray,
        public width: number,
        public height: number,
      ) {}
    },
  );
  vi.spyOn(HTMLCanvasElement.prototype, "getContext").mockReturnValue({
    putImageData,
  } as unknown as CanvasRenderingContext2D);
  vi.spyOn(HTMLElement.prototype, "clientWidth", "get").mockReturnValue(width);
  vi.spyOn(HTMLElement.prototype, "clientHeight", "get").mockReturnValue(
    height,
  );
}

// The last picture sent to the canvas: RGBA of the whole area.
export function lastPicture(): { data: Uint8ClampedArray; width: number } {
  const calls = putImageData.mock.calls;
  const image = calls[calls.length - 1][0] as {
    data: Uint8ClampedArray;
    width: number;
  };
  return image;
}

export function pictureAt(x: number, y: number): number[] {
  const { data, width } = lastPicture();
  const o = (y * width + x) * 4;
  return [data[o], data[o + 1], data[o + 2], data[o + 3]];
}

// What the app does with the editor: it owns the shortcut keys and runs the
// handlers the editor gives.
function Host({ sprite, api }: { sprite: SpriteRef | null; api: EditorApi }) {
  const [handlers, setHandlers] = useState<CommandHandlers>({});
  useShortcuts(handlers, "other");
  const register = useCallback(
    (next: CommandHandlers | null) => setHandlers(next ?? {}),
    [],
  );
  return <EditorScreen sprite={sprite} api={api} onCommands={register} />;
}

export function renderEditor(
  api: EditorApi,
  options: {
    sprite?: SpriteRef | null;
    language?: "en" | "es";
    pixelGrid?: boolean;
  } = {},
) {
  const language = options.language ?? "en";
  return render(
    <PreferencesProvider
      initial={{
        ...defaultPreferences(language),
        pixelGrid: options.pixelGrid ?? true,
      }}
    >
      <I18nProvider language={language}>
        <Host
          sprite={options.sprite === undefined ? goblin : options.sprite}
          api={api}
        />
      </I18nProvider>
    </PreferencesProvider>,
  );
}

// Where the center of a sprite pixel is on the screen when the sprite is
// fitted and centered in a 500x300 area (what `mockLayout` sets).
export function pixelPoint(
  spriteWidth: number,
  spriteHeight: number,
  x: number,
  y: number,
  area = { width: 500, height: 300 },
) {
  const view = centeredView(
    spriteWidth,
    spriteHeight,
    fitZoom(spriteWidth, spriteHeight, area.width, area.height),
    area.width,
    area.height,
  );
  return {
    clientX: view.x + x * view.zoom + Math.floor(view.zoom / 2),
    clientY: view.y + y * view.zoom + Math.floor(view.zoom / 2),
    pointerId: 1,
  };
}

// The color of the screen pixel at the center of a sprite pixel.
export function colorOfPixel(
  spriteWidth: number,
  spriteHeight: number,
  x: number,
  y: number,
): number[] {
  const point = pixelPoint(spriteWidth, spriteHeight, x, y);
  return pictureAt(point.clientX, point.clientY);
}
