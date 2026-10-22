type Props = {
  message: string;
  tone?: "neutral" | "error";
  actionLabel?: string;
  onAction?: () => void;
};

// A message bar at the bottom of the body. It can carry one action.
export function Toast({
  message,
  tone = "neutral",
  actionLabel,
  onAction,
}: Props) {
  return (
    <div
      role="status"
      className={`fixed bottom-12 left-1/2 flex -translate-x-1/2 items-center gap-3 rounded-md border bg-surface px-4 py-2 text-sm ${tone === "error" ? "border-error" : "border-border-strong"}`}
    >
      <span>{message}</span>
      {actionLabel && (
        <button
          type="button"
          onClick={onAction}
          className={`font-semibold ${tone === "error" ? "text-error" : "text-text underline"}`}
        >
          {actionLabel}
        </button>
      )}
    </div>
  );
}
