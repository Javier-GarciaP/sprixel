import { useEffect, useState } from "react";
import { useT } from "@/shared/lib/i18n";

type Props = {
  frame: number;
  // The own duration in milliseconds, or null when the frame has none.
  own: number | null;
  // What the frame lasts when it has none (the speed of its animation).
  fallback: number;
  onSave: (ms: number) => void;
};

// The time of one frame, under its thumbnail. Empty means "none of its own":
// the box then shows the time the animation gives it. The value is saved when
// the person leaves the box or presses Enter.
export function DurationField({ frame, own, fallback, onSave }: Props) {
  const t = useT();
  const shown = own === null ? "" : String(own);
  const [text, setText] = useState(shown);
  // The core changed it (another action, undo): show that.
  useEffect(() => setText(shown), [shown]);

  const commit = () => {
    const ms = Number(text);
    if (text === shown) return;
    // Empty gives the default back: the speed of the animation decides.
    if (text.trim() === "") return onSave(100);
    if (Number.isInteger(ms) && ms >= 10 && ms <= 10000) return onSave(ms);
    setText(shown);
  };

  return (
    <input
      type="text"
      inputMode="numeric"
      maxLength={5}
      value={text}
      placeholder={String(fallback)}
      aria-label={t("frames.duration", { n: frame })}
      title={t("frames.durationHint")}
      onChange={(event) => setText(event.target.value)}
      onBlur={commit}
      onKeyDown={(event) => event.key === "Enter" && commit()}
      className="w-11 min-w-0 rounded border border-transparent bg-transparent px-0.5 py-0.5 text-right font-mono text-[10px] text-text-muted outline-none placeholder:text-text-faint hover:border-border focus:border-amber focus:bg-bg"
    />
  );
}
