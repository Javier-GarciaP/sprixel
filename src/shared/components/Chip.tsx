import type { ReactNode } from "react";

type Props = { tone?: "neutral" | "success" | "error"; children: ReactNode };

const dot = {
  neutral: "bg-text-faint",
  success: "bg-success",
  error: "bg-error",
};

// A small rounded label with a colored dot, like "Claude Code · OpenCode".
export function Chip({ tone = "neutral", children }: Props) {
  return (
    <span className="inline-flex items-center gap-2 rounded-full border border-border bg-surface px-3 py-1 text-xs text-text-muted">
      <span className={`h-2 w-2 rounded-full ${dot[tone]}`} aria-hidden />
      {children}
    </span>
  );
}
