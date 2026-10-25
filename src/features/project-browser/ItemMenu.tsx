import { useState, type CSSProperties } from "react";
import { Menu, type MenuItem } from "@/shared/components";
import { BrowserIcon } from "./icons";

type Props = {
  label: string;
  items: MenuItem[];
  className?: string;
};

// The "..." button of a card or a folder, with its menu below it.
export function ItemMenu({ label, items, className = "" }: Props) {
  // Where the menu is drawn. The tree column scrolls and would cut a menu
  // placed inside it, so it is placed on the screen, under the button.
  const [position, setPosition] = useState<CSSProperties | null>(null);
  const toggle = (button: HTMLButtonElement) => {
    if (position) return setPosition(null);
    const rect = button.getBoundingClientRect();
    setPosition({
      top: rect.bottom + 4,
      left: Math.max(8, Math.min(rect.left, window.innerWidth - 276)),
    });
  };
  return (
    <div className={className}>
      <button
        type="button"
        aria-label={label}
        aria-haspopup="menu"
        aria-expanded={position !== null}
        onClick={(e) => toggle(e.currentTarget)}
        className="rounded-md border border-border bg-surface p-1 text-text-muted hover:bg-surface-2"
      >
        <BrowserIcon name="more" />
      </button>
      {position && (
        <Menu
          items={items}
          position={position}
          onClose={() => setPosition(null)}
        />
      )}
    </div>
  );
}
