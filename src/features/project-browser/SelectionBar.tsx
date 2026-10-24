import { useState } from "react";
import { Button, Menu, type MenuItem } from "@/shared/components";
import { useT } from "@/shared/lib/i18n";
import { BrowserIcon } from "./icons";

type Props = {
  count: number;
  // The folders the selected sprites can move to.
  moveItems: MenuItem[];
  onDelete: () => void;
  onCancel: () => void;
};

// Shown above the cards when several sprites are selected: how many, and what
// can be done with all of them.
export function SelectionBar({ count, moveItems, onDelete, onCancel }: Props) {
  const t = useT();
  const [moving, setMoving] = useState(false);
  return (
    <div
      role="toolbar"
      aria-label={t("browse.selectionBar")}
      className="flex items-center gap-2"
    >
      <span className="mr-2 text-sm font-semibold text-amber">
        {t("browse.selectedCount", { count })}
      </span>
      <div className="relative">
        <Button aria-haspopup="menu" onClick={() => setMoving(!moving)}>
          <BrowserIcon name="folder" />
          {t("browse.move")}
        </Button>
        {moving && <Menu items={moveItems} onClose={() => setMoving(false)} />}
      </div>
      <Button variant="danger" onClick={onDelete}>
        <BrowserIcon name="trash" />
        {t("browse.delete")}
      </Button>
      <Button onClick={onCancel}>{t("browse.cancelSelection")}</Button>
    </div>
  );
}
