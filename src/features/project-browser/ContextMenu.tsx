import { Menu, type MenuItem } from "@/shared/components";

type Props = {
  // Where the mouse was, in window coordinates.
  x: number;
  y: number;
  items: MenuItem[];
  onClose: () => void;
};

// The menu of a right click, opened next to the pointer.
export function ContextMenu({ x, y, items, onClose }: Props) {
  return (
    <div className="fixed z-30" style={{ left: x, top: y }}>
      <Menu items={items} onClose={onClose} />
    </div>
  );
}
