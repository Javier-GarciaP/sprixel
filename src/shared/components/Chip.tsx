import type { ReactNode } from "react";

type Props = {
  tone?: "neutral" | "success" | "warning" | "error";
  children: ReactNode;
};

const dot = {
  neutral: "bg-text-faint",
  success: "bg-success",
  warning: "bg-amber",
  error: "bg-error",
};

// Waiting and error labels also take the color in the border and the text.
const outline = {
  neutral: "border-border text-text-muted",
  success: "border-border text-text-muted",
  warning: "border-amber text-amber",
  error: "border-error text-error",
};

// A small rounded label with a colored dot, like "Claude Code · OpenCode".
export function Chip({ tone = "neutral", children }: Props) {
  return (
    <span
      className={`inline-flex items-center gap-2 rounded-full border bg-surface px-3 py-1 text-xs ${outline[tone]}`}
    >
      <span className={`h-2 w-2 rounded-full ${dot[tone]}`} aria-hidden />
      {children}
    </span>
  );
}
