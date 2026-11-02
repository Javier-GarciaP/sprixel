import { useEffect, useState } from "react";
import { useT } from "@/shared/lib/i18n";
import { asGridFailure, coreTextApi } from "./api";
import { GridPreview } from "./GridPreview";
import type {
  GridCheck,
  GridError,
  GridPreview as Preview,
  SpriteRef,
  TextApi,
} from "./types";

// Every line has the same height, so the line numbers line up with the text.
const LINE_HEIGHT = 21;
// How long the view waits after the last key before it checks the text.
const CHECK_DELAY_MS = 200;

// What a person needs next to the line: the message without its "line 15
// (row 7 of frame 1):" start, which the line already says.
const shortMessage = (error: GridError) =>
  error.message.replace(/^line \d+( \([^)]*\))?: /, "");

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
  // The last answer of the core for a text, and the last picture that was
  // valid (it stays while the text has errors).
  const [result, setResult] = useState<{
    text: string;
    check: GridCheck;
  } | null>(null);
  const [lastValid, setLastValid] = useState<Preview | null>(null);
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

  // Another sprite or frame has another picture.
  useEffect(() => {
    setLastValid(null);
    setResult(null);
  }, [path, frame]);

  // Checks the text. The text that comes from the core is checked at once;
  // what a person types waits for a pause, so a key does not call the core.
  useEffect(() => {
    if (text === null) return;
    let stopped = false;
    const timer = setTimeout(
      () => {
        api
          .check(sprite, text)
          .then((check) => {
            if (stopped) return;
            setError(null);
            setResult({ text, check });
            if (check.preview) setLastValid(check.preview);
          })
          .catch((failure) => {
            if (stopped) return;
            const { message, code } = asGridFailure(failure);
            setError(t("text.checkError", { message: message ?? code }));
          });
      },
      text === core ? 0 : CHECK_DELAY_MS,
    );
    return () => {
      stopped = true;
      clearTimeout(timer);
    };
  }, [api, path, text, core, t]);

  // The answer is about the text on the screen only when they match.
  const current = result && result.text === text ? result.check : null;
  const errors = current?.errors ?? [];
  const errorAt = new Map<number, GridError>();
  for (const e of errors) {
    if (e.line !== null && !errorAt.has(e.line)) errorAt.set(e.line, e);
  }
  const lines = (text ?? "").split("\n");
  const widest = Math.max(1, ...lines.map((line) => line.length));

  let status: { tone: "ok" | "error" | "wait"; message: string } | null = null;
  if (text !== null) {
    if (errors.length > 0) {
      const [first, ...rest] = errors;
      status = {
        tone: "error",
        message:
          rest.length === 0
            ? first.message
            : t("text.moreErrors", {
                message: first.message,
                count: rest.length,
              }),
      };
    } else if (current?.summary) {
      status = {
        tone: "ok",
        message: t("text.valid", { ...current.summary }),
      };
    } else if (!error) {
      status = { tone: "wait", message: t("text.checking") };
    }
  }
  const invalid = errors.length > 0;

  return (
    <div className="flex min-h-0 min-w-0 flex-1 gap-4 p-4">
      <section
        aria-label={t("text.editor")}
        className={`flex min-h-0 min-w-0 flex-1 flex-col rounded-md border bg-surface ${invalid ? "border-error" : "border-border"}`}
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
              <div className="relative flex-1">
                {/* Behind the text: the lines with an error and what is wrong. */}
                <div
                  aria-hidden
                  className="pointer-events-none absolute inset-0"
                >
                  {lines.map((line, i) => {
                    const problem = errorAt.get(i + 1);
                    return (
                      <div
                        key={i}
                        data-error-line={problem ? i + 1 : undefined}
                        className={`whitespace-pre ${problem ? "bg-error/15" : ""}`}
                        style={{
                          height: LINE_HEIGHT,
                          lineHeight: `${LINE_HEIGHT}px`,
                        }}
                      >
                        {problem && (
                          <>
                            <span className="invisible">{line}</span>
                            <span className="ml-3 font-sans text-xs font-semibold text-error">
                              {shortMessage(problem)}
                            </span>
                          </>
                        )}
                      </div>
                    );
                  })}
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
                  className="relative block resize-none overflow-hidden bg-transparent text-text outline-none"
                  style={{
                    height: lines.length * LINE_HEIGHT,
                    lineHeight: `${LINE_HEIGHT}px`,
                    width: `max(100%, ${widest + 2}ch)`,
                  }}
                />
              </div>
            </div>
          </div>
        )}
        {status && (
          <p
            role="status"
            data-tone={status.tone}
            className={`flex shrink-0 items-center gap-2 border-t border-border px-3 py-2 text-xs ${status.tone === "ok" ? "text-success" : status.tone === "error" ? "text-error" : "text-text-muted"}`}
          >
            <span
              aria-hidden
              className={`h-2 w-2 shrink-0 rounded-full ${status.tone === "ok" ? "bg-success" : status.tone === "error" ? "bg-error" : "bg-text-faint"}`}
            />
            {status.message}
          </p>
        )}
      </section>
      <aside className="flex w-[360px] shrink-0 flex-col items-center gap-3 rounded-md border border-border bg-surface p-4">
        <p className="font-title text-sm font-bold">
          {invalid ? t("text.lastValid") : t("text.liveView")}
        </p>
        <GridPreview preview={lastValid} dimmed={invalid} />
      </aside>
    </div>
  );
}
