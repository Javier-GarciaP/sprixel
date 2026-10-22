import { useEffect, useRef } from "react";

export type MenuItem = {
  id: string;
  label: string;
  hint?: string;
  disabled?: boolean;
  // Draws a separator line above this item.
  separator?: boolean;
  onSelect: () => void;
};

type Props = {
  items: MenuItem[];
  onClose: () => void;
  // A small note under the list.
  footer?: string;
};

// A drop-down list of commands. Escape or a click outside closes it.
export function Menu({ items, onClose, footer }: Props) {
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => e.key === "Escape" && onClose();
    const onClick = (e: MouseEvent) => {
      if (!ref.current?.contains(e.target as Node)) onClose();
    };
    window.addEventListener("keydown", onKey);
    window.addEventListener("mousedown", onClick);
    return () => {
      window.removeEventListener("keydown", onKey);
      window.removeEventListener("mousedown", onClick);
    };
  }, [onClose]);

  return (
    <div
      ref={ref}
      role="menu"
      className="absolute z-30 mt-1 min-w-[260px] rounded-lg border border-border-strong bg-surface p-1.5 shadow-lg"
    >
      {items.map((item) => (
        <div key={item.id}>
          {item.separator && <div className="my-1 h-px bg-border" />}
          <button
            type="button"
            role="menuitem"
            disabled={item.disabled}
            onClick={() => {
              onClose();
              item.onSelect();
            }}
            className="flex w-full items-center justify-between rounded-md px-3 py-1.5 text-left text-sm hover:bg-surface-2 disabled:opacity-40"
          >
            <span>{item.label}</span>
            {item.hint && (
              <span className="font-mono text-xs text-text-faint">
                {item.hint}
              </span>
            )}
          </button>
        </div>
      ))}
      {footer && (
        <p className="mt-1 border-t border-border px-3 pt-1.5 text-xs text-text-faint">
          {footer}
        </p>
      )}
    </div>
  );
}
