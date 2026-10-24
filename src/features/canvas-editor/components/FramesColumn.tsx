import { useT } from "@/shared/lib/i18n";

type Props = {
  frameCount: number;
  frame: number;
  onFrame: (frame: number) => void;
};

// The frames column. Adding and deleting frames and the animations belong to
// the frames and animations spec; for now it lists the frame numbers so the
// current frame can be changed.
export function FramesColumn({ frameCount, frame, onFrame }: Props) {
  const t = useT();
  return (
    <aside
      aria-label={t("editor.frames")}
      className="flex w-28 shrink-0 flex-col items-center gap-2 overflow-y-auto border-r border-border bg-bg py-3"
    >
      <h2 className="text-xs font-semibold text-text-muted">
        {t("editor.frames")}
      </h2>
      {Array.from({ length: frameCount }, (_, i) => i + 1).map((n) => (
        <button
          key={n}
          type="button"
          aria-label={t("editor.frame", { n })}
          aria-current={n === frame ? "true" : undefined}
          onClick={() => onFrame(n)}
          className={`h-16 w-20 rounded-md border font-mono text-xs ${n === frame ? "border-amber bg-surface-2 text-text" : "border-border bg-surface text-text-muted"}`}
        >
          {n}
        </button>
      ))}
    </aside>
  );
}
