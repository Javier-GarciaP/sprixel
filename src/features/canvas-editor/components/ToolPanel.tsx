import { useEffect, type ReactNode } from "react";
import { Button, Icon } from "@/shared/components";
import { useT } from "@/shared/lib/i18n";

type Props = {
  title: string;
  onClose: () => void;
  // The Apply button; it stays off while `canApply` is false.
  // Without Apply the panel only shows information (the Review panel).
  onApply?: () => void;
  canApply?: boolean;
  children: ReactNode;
};

// A panel over the corner of the canvas for the tools that show a preview.
// It is not a modal: there is no dark backdrop, so the preview on the canvas
// stays visible. Escape closes it without writing anything.
export function ToolPanel({
  title,
  onClose,
  onApply,
  canApply = false,
  children,
}: Props) {
  const t = useT();
  useEffect(() => {
    const onKey = (event: KeyboardEvent) => event.key === "Escape" && onClose();
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [onClose]);
  return (
    <div
      role="dialog"
      aria-modal="false"
      aria-label={title}
      className="absolute right-3 top-3 z-20 flex max-h-[calc(100%-1.5rem)] w-72 flex-col gap-3 overflow-y-auto rounded-lg border-2 border-border-strong bg-surface p-4 shadow-lg"
    >
      <div className="flex items-center justify-between">
        <h2 className="font-title text-base font-bold">{title}</h2>
        <button
          type="button"
          aria-label={t("panel.close")}
          onClick={onClose}
          className="text-text-muted"
        >
          <Icon name="close" />
        </button>
      </div>
      {onApply && (
        <p className="text-xs text-text-muted">{t("panel.previewing")}</p>
      )}
      {children}
      <div className="flex justify-end gap-2">
        <Button onClick={onClose}>
          {t(onApply ? "panel.cancel" : "panel.done")}
        </Button>
        {onApply && (
          <Button variant="primary" disabled={!canApply} onClick={onApply}>
            {t("panel.apply")}
          </Button>
        )}
      </div>
    </div>
  );
}
