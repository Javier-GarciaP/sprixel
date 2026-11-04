import { useEffect, useRef, useState } from "react";
import {
  backgroundStyle,
  type PreviewBackground,
} from "@/features/frames-animations";
import { useT } from "@/shared/lib/i18n";
import type { PowerApi, SpriteRef, TileCheck } from "../api/types";
import type { ColorTable } from "../render/colors";
import { TILES, tilePicture } from "../render/tiles";

type Props = {
  api: PowerApi;
  sprite: SpriteRef;
  frame: number;
  pixels: Uint8Array;
  width: number;
  height: number;
  colors: ColorTable;
  background: PreviewBackground;
  // Changes after every stroke, so the tile and the check are made again.
  version: number;
};

// The frame repeated 3 x 3 times, to see the seams of a tile, with a warning
// when the first and the last column (or row) do not match. The check is the
// one of the `check_tile` tool of the agents.
export function TilePreview({
  api,
  sprite,
  frame,
  pixels,
  width,
  height,
  colors,
  background,
  version,
}: Props) {
  const t = useT();
  const canvas = useRef<HTMLCanvasElement>(null);
  const [check, setCheck] = useState<TileCheck | null>(null);

  useEffect(() => {
    const element = canvas.current;
    const context = element?.getContext("2d");
    if (!element || !context) return;
    element.width = width * TILES;
    element.height = height * TILES;
    context.putImageData(
      new ImageData(
        tilePicture(pixels, width, height, colors),
        width * TILES,
        height * TILES,
      ),
      0,
      0,
    );
  }, [pixels, width, height, colors, version, frame]);

  useEffect(() => {
    let stale = false;
    api
      .tileCheck(sprite, frame)
      .then((result) => !stale && setCheck(result))
      .catch(() => !stale && setCheck(null));
    return () => {
      stale = true;
    };
  }, [api, sprite, frame, version]);

  return (
    <div className="flex flex-col gap-2">
      <div
        className="flex items-center justify-center overflow-hidden rounded"
        style={{ ...backgroundStyle(background), height: 200 }}
      >
        <canvas
          ref={canvas}
          role="img"
          aria-label={t("preview.tileLabel", { frame })}
          className="max-h-full max-w-full object-contain [image-rendering:pixelated]"
        />
      </div>
      {check && (
        <p
          role="note"
          className={`rounded-md border px-2 py-1 text-xs ${check.seamless ? "border-border" : "border-amber"}`}
        >
          {check.seamless
            ? t("preview.tileSeamless")
            : t("preview.tileSeams", { count: check.mismatches.length })}
        </p>
      )}
    </div>
  );
}
