import { render } from "@testing-library/react";
import { vi } from "vitest";
import { I18nProvider } from "@/shared/lib/i18n";
import {
  PreferencesProvider,
  defaultPreferences,
} from "@/shared/lib/preferences";
import { EditorScreen } from "../components/EditorScreen";
import type { EditorApi, SpriteRef } from "../api/types";

export const goblin: SpriteRef = { folder: ["Characters"], name: "goblin" };

// The size of the canvas area and the screen pictures sent to the canvas, for
// the tests: jsdom has no layout and no canvas.
export const putImageData = vi.fn();

export function mockLayout(width = 500, height = 300) {
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
        <EditorScreen
          sprite={options.sprite === undefined ? goblin : options.sprite}
          api={api}
        />
      </I18nProvider>
    </PreferencesProvider>,
  );
}
