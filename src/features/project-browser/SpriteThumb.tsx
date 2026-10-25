import { useEffect, useRef, useState } from "react";
import type { FrameData, SpriteSummary } from "@/shared/lib/core";
import { drawFrame, type FrameCache } from "@/shared/lib/render";

// A neutral gray checkerboard, the same in both themes, so transparent
// pixels and black outlines stay visible.
export const checkerboard = {
  backgroundColor: "#3a3a42",
  backgroundImage:
    "conic-gradient(#2e2e35 25%, transparent 25% 50%, #2e2e35 50% 75%, transparent 75%)",
  backgroundSize: "16px 16px",
} as const;

type Props = {
  sprite: Pick<SpriteSummary, "folder" | "name">;
  cache: FrameCache;
  // Changes after every change of the project, so a changed sprite is read again.
  revision: number;
  // The preview fits in a square of this many pixels, with whole-number scaling.
  box: number;
};

// The first frame of a sprite. It is asked from the core only when it
// scrolls into view, so a project with many sprites opens fast.
export function SpriteThumb({ sprite, cache, revision, box }: Props) {
  const canvas = useRef<HTMLCanvasElement>(null);
  const [frame, setFrame] = useState<FrameData | null>(null);
  const [visible, setVisible] = useState(
    typeof IntersectionObserver === "undefined",
  );
  const folderKey = sprite.folder.join("/");

  useEffect(() => {
    const element = canvas.current;
    if (visible || !element) return;
    const watcher = new IntersectionObserver((entries) => {
      if (entries.some((entry) => entry.isIntersecting)) setVisible(true);
    });
    watcher.observe(element);
    return () => watcher.disconnect();
  }, [visible]);

  useEffect(() => {
    if (!visible) return;
    let stopped = false;
    cache
      .get(folderKey ? folderKey.split("/") : [], sprite.name)
      .then((loaded) => {
        if (!stopped) setFrame(loaded);
      })
      .catch(() => {});
    return () => {
      stopped = true;
    };
  }, [visible, cache, folderKey, sprite.name, revision]);

  useEffect(() => {
    if (frame && canvas.current) drawFrame(canvas.current, frame, box);
  }, [frame, box]);

  return (
    <div
      className="flex items-center justify-center rounded-md"
      style={{ ...checkerboard, width: "100%", height: box }}
    >
      <canvas
        ref={canvas}
        aria-hidden
        className="[image-rendering:pixelated]"
      />
    </div>
  );
}
