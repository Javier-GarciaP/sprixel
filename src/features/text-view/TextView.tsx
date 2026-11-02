import { useEffect, useState } from "react";
import { useT } from "@/shared/lib/i18n";
import { asGridFailure, coreTextApi } from "./api";
import type { SpriteRef, TextApi } from "./types";

// Every line has the same height, so the line numbers line up with the text.
const LINE_HEIGHT = 21;

type Props = {
  // The calls to the core. The tests give a fake one.
  api?: TextApi;
  sprite: SpriteRef;
  frame: number;
  frameCount: number;
  // Changes when the sprite changes, so the text is read again.
  revision: number;
  onFrame: (frame: number) => void;
};

// The sprite as a text grid: the palette, the legend and the rows of one
// frame, in the same format the AI tools read and write. A person can edit it.
export function TextView({
  api = coreTextApi,
  sprite,
  frame,
  frameCount,
  revision,
  onFrame,
}: Props) {
  const t = useT();
  // The text of the frame as the core has it.
  const [loaded, setLoaded] = useState<{ frame: number; text: string } | null>(
    null,
  );
  // What the person typed, for each frame that was edited.
  const [drafts, setDrafts] = useState<Record<number, string>>({});
  const [error, setError] = useState<string | null>(null);
  const path = JSON.stringify([...sprite.folder, sprite.name]);

  useEffect(() => {
    let stopped = false;
    api
      .loadText(sprite, frame)
      .then((text) => {
        if (stopped) return;
        setError(null);
        setLoaded({ frame, text });
      })
      .catch((failure) => {
        if (stopped) return;
        const { message, code } = asGridFailure(failure);
        setError(t("text.loadError", { message: message ?? code }));
      });
    return () => {
      stopped = true;
    };
  }, [api, path, frame, revision, t]);

  const core = loaded?.frame === frame ? loaded.text : null;
  const text = drafts[frame] ?? core;
  const lines = (text ?? "").split("\n");
  const widest = Math.max(1, ...lines.map((line) => line.length));

  return (
    <div className="flex min-h-0 min-w-0 flex-1 flex-col p-4">
      <section
        aria-label={t("text.editor")}
        className="flex min-h-0 flex-1 flex-col rounded-md border border-border bg-surface"
      >
        <header className="flex h-10 shrink-0 items-center gap-3 border-b border-border px-3">
          <span className="font-title text-sm font-bold">
            {t("text.fileName", { name: sprite.name })}
          </span>
          <div
            role="group"
            aria-label={t("text.frames")}
            className="ml-auto flex items-center gap-1"
          >
            {Array.from({ length: frameCount }, (_, i) => i + 1).map((n) => (
              <button
                key={n}
                type="button"
                aria-pressed={n === frame}
                onClick={() => onFrame(n)}
                className={`rounded px-2 py-0.5 font-mono text-xs ${n === frame ? "bg-surface-2 font-semibold text-text" : "text-text-muted"}`}
              >
                {n}
              </button>
            ))}
          </div>
          <span className="text-xs text-text-muted">
            {t("text.frame", { n: frame })}
          </span>
        </header>
        {error && (
          <p role="alert" className="p-3 text-sm text-error">
            {error}
          </p>
        )}
        {text !== null && (
          <div className="min-h-0 flex-1 overflow-auto">
            <div className="flex font-mono text-[13px]">
              <div
                aria-hidden
                data-testid="line-numbers"
                className="shrink-0 select-none px-3 text-right text-text-faint"
              >
                {lines.map((_, i) => (
                  <div key={i} style={{ height: LINE_HEIGHT }}>
                    {i + 1}
                  </div>
                ))}
              </div>
              <textarea
                aria-label={t("text.editor")}
                value={text}
                wrap="off"
                spellCheck={false}
                autoCapitalize="off"
                autoCorrect="off"
                onChange={(event) =>
                  setDrafts({ ...drafts, [frame]: event.target.value })
                }
                className="block resize-none overflow-hidden bg-transparent text-text outline-none"
                style={{
                  height: lines.length * LINE_HEIGHT,
                  lineHeight: `${LINE_HEIGHT}px`,
                  width: `max(100%, ${widest + 2}ch)`,
                }}
              />
            </div>
          </div>
        )}
      </section>
    </div>
  );
}
