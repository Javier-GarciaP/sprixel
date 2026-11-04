import { useId, useState, type ReactNode } from "react";

type Props = {
  // The name of the control.
  label: string;
  // The shortcut, already formatted ("Ctrl+Z"), shown as a key badge.
  shortcut?: string;
  // Why the control is disabled, shown under the name.
  reason?: string;
  children: ReactNode;
};

// A small help bubble under its child. It shows when the pointer is over the
// child or when the keyboard focus is inside it. The wrapper listens, so a
// disabled button (which gets no mouse events) still explains itself.
export function Tooltip({ label, shortcut, reason, children }: Props) {
  const [hover, setHover] = useState(false);
  const [focus, setFocus] = useState(false);
  const id = useId();
  return (
    <span
      className="relative inline-flex"
      aria-describedby={hover || focus ? id : undefined}
      onMouseEnter={() => setHover(true)}
      onMouseLeave={() => setHover(false)}
      onFocus={() => setFocus(true)}
      onBlur={() => setFocus(false)}
    >
      {children}
      {(hover || focus) && (
        <span
          id={id}
          role="tooltip"
          className="pointer-events-none absolute top-full left-1/2 z-30 mt-1.5 flex w-max max-w-56 -translate-x-1/2 flex-col gap-0.5 rounded-md border border-border-strong bg-surface px-2 py-1 text-xs text-text shadow-lg"
        >
          <span className="flex items-center gap-2">
            {label}
            {shortcut && (
              <kbd className="rounded border border-border bg-surface-2 px-1 font-mono text-[11px] text-text-muted">
                {shortcut}
              </kbd>
            )}
          </span>
          {reason && <span className="text-text-faint">{reason}</span>}
        </span>
      )}
    </span>
  );
}
