import { useEffect, useState } from "react";
import { useT } from "@/shared/lib/i18n";
import type { PowerApi, Review, SpriteRef } from "../api/types";
import { powerErrorText } from "../powerError";
import type { PaletteColor } from "../types";
import { KeyPicker } from "./KeyPicker";
import { ToolPanel } from "./ToolPanel";

// The most items of one list that are shown.
const SHOWN = 50;

type Props = {
  api: PowerApi;
  sprite: SpriteRef;
  palette: PaletteColor[];
  // Changes when the sprite changes, so the review is made again.
  revision: number;
  // Point at a pixel of a frame (the editor goes to the frame and marks it).
  onPoint: (frame: number, x: number, y: number) => void;
  onClose: () => void;
};

// The Review panel: what `analyze_sprite` finds, for a person. Clicking a
// pixel in a list marks it on the canvas.
export function ReviewPanel({
  api,
  sprite,
  palette,
  revision,
  onPoint,
  onClose,
}: Props) {
  const t = useT();
  const [outlineKey, setOutlineKey] = useState<string | null>(null);
  const [review, setReview] = useState<Review | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let stale = false;
    api
      .review(sprite, outlineKey)
      .then((result) => {
        if (stale) return;
        setError(null);
        setReview(result);
      })
      .catch((failure) => {
        if (!stale) setError(powerErrorText(t, failure));
      });
    return () => {
      stale = true;
    };
  }, [api, sprite, outlineKey, revision]);

  const pixels = (
    title: string,
    items: { frame: number; at: [number, number] }[],
  ) =>
    items.length > 0 && (
      <section aria-label={title} className="flex flex-col gap-1">
        <h3 className="text-xs font-semibold">
          {title} ({items.length})
        </h3>
        <ul className="flex flex-wrap gap-1">
          {items.slice(0, SHOWN).map(({ frame, at: [x, y] }) => (
            <li key={`${frame}-${x}-${y}`}>
              <button
                type="button"
                onClick={() => onPoint(frame, x, y)}
                className="rounded border border-border bg-bg px-1.5 py-0.5 font-mono text-xs"
              >
                {t("review.at", { frame, x, y })}
              </button>
            </li>
          ))}
        </ul>
        {items.length > SHOWN && (
          <p className="text-xs text-text-muted">
            {t("review.more", { count: items.length - SHOWN })}
          </p>
        )}
      </section>
    );

  const loose =
    review?.frames.flatMap((f) =>
      f.loose.map((at) => ({ frame: f.frame, at })),
    ) ?? [];
  const gaps =
    review?.frames.flatMap((f) =>
      f.gaps.map((at) => ({ frame: f.frame, at })),
    ) ?? [];
  const clean =
    review !== null &&
    loose.length === 0 &&
    gaps.length === 0 &&
    review.unused.length === 0;

  return (
    <ToolPanel title={t("review.title")} onClose={onClose}>
      <KeyPicker
        label={t("review.outlineKey")}
        palette={palette}
        chosen={outlineKey ? [outlineKey] : []}
        onToggle={(key) => setOutlineKey((now) => (now === key ? null : key))}
      />
      {error && (
        <p role="alert" className="text-xs text-error">
          {error}
        </p>
      )}
      {clean && <p className="text-sm">{t("review.clean")}</p>}
      {review && review.unused.length > 0 && (
        <section
          aria-label={t("review.unused")}
          className="flex flex-col gap-1"
        >
          <h3 className="text-xs font-semibold">
            {t("review.unused")} ({review.unused.length})
          </h3>
          <div className="flex flex-wrap gap-1">
            {review.unused.map((key) => (
              <span
                key={key}
                title={key}
                style={{
                  backgroundColor: palette.find((c) => c.key === key)?.hex,
                }}
                className="inline-flex h-6 w-6 items-center justify-center rounded border border-border font-mono text-[10px] mix-blend-normal"
              >
                {key}
              </span>
            ))}
          </div>
        </section>
      )}
      {pixels(t("review.loose"), loose)}
      {pixels(t("review.gaps"), gaps)}
      {review && review.changes.length > 0 && (
        <section
          aria-label={t("review.changes")}
          className="flex flex-col gap-1"
        >
          <h3 className="text-xs font-semibold">{t("review.changes")}</h3>
          <ul className="flex flex-col gap-0.5 text-xs">
            {review.changes.map((change) => (
              <li key={change.from}>
                {change.changedPixels === 0
                  ? t("review.noChange", { from: change.from, to: change.to })
                  : `${t("review.change", { from: change.from, to: change.to, count: change.changedPixels })} (${t(change.groundKept ? "review.groundKept" : "review.groundMoved")})`}
              </li>
            ))}
          </ul>
        </section>
      )}
    </ToolPanel>
  );
}
