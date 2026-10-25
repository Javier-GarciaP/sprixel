import { useState } from "react";
import { Menu, type MenuItem } from "@/shared/components";
import { BrowserIcon } from "./icons";

type Props = {
  label: string;
  items: MenuItem[];
  className?: string;
};

// The "..." button of a card or a folder, with its menu below it.
export function ItemMenu({ label, items, className = "" }: Props) {
  const [open, setOpen] = useState(false);
  return (
    <div className={`relative ${className}`}>
      <button
        type="button"
        aria-label={label}
        aria-haspopup="menu"
        aria-expanded={open}
        onClick={() => setOpen(!open)}
        className="rounded-md border border-border bg-surface p-1 text-text-muted hover:bg-surface-2"
      >
        <BrowserIcon name="more" />
      </button>
      {open && <Menu items={items} onClose={() => setOpen(false)} />}
    </div>
  );
}
