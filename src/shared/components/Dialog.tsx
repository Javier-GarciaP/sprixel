import { useEffect, type ReactNode } from "react";

type Props = {
  title: string;
  closeLabel: string;
  onClose: () => void;
  children: ReactNode;
};

// A modal over the current screen. Escape closes it.
export function Dialog({ title, closeLabel, onClose, children }: Props) {
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => e.key === "Escape" && onClose();
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [onClose]);

  return (
    <div className="fixed inset-0 z-40 flex items-center justify-center bg-black/60">
      <div
        role="dialog"
        aria-modal="true"
        aria-label={title}
        className="w-[560px] max-w-[90vw] rounded-lg border-2 border-border-strong bg-surface p-7"
      >
        <div className="mb-4 flex items-center justify-between">
          <h2 className="font-title text-xl font-bold">{title}</h2>
          <button
            type="button"
            aria-label={closeLabel}
            onClick={onClose}
            className="text-text-muted"
          >
            ✕
          </button>
        </div>
        {children}
      </div>
    </div>
  );
}
