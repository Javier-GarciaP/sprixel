import type { MenuItem } from "@/shared/components";
import { Button, Menu } from "@/shared/components";
import { useState } from "react";
import type { SpriteSummary } from "@/shared/lib/core";
import { useT } from "@/shared/lib/i18n";
import { BrowserIcon } from "./icons";

type Props = {
  sprites: SpriteSummary[];
  moveItems: MenuItem[];
  onDelete: () => void;
};

// The right side when several sprites are selected: their names, their frames
// added up and what can be done with all of them.
export function SelectionDetails({ sprites, moveItems, onDelete }: Props) {
  const t = useT();
  const [moving, setMoving] = useState(false);
  const frames = sprites.reduce((total, sprite) => total + sprite.frames, 0);
  const sizes = new Set(sprites.map((s) => `${s.width} × ${s.height} px`));
  return (
    <div className="flex flex-col gap-4">
      <h2 className="font-title text-xl font-bold">
        {t("browse.selectedSprites", { count: sprites.length })}
      </h2>
      <section
        aria-label={t("browse.selection")}
        className="rounded-lg border border-border bg-surface p-3"
      >
        <h3 className="mb-1 text-xs font-semibold text-text-muted">
          {t("browse.selection")}
        </h3>
        <dl className="text-sm">
          <div className="flex justify-between gap-4 py-0.5">
            <dt className="text-text-muted">{t("browse.sprites")}</dt>
            <dd className="text-right font-mono">
              {sprites.map((s) => s.name).join(", ")}
            </dd>
          </div>
          <div className="flex justify-between gap-4 py-0.5">
            <dt className="text-text-muted">{t("browse.framesLabel")}</dt>
            <dd className="font-mono">{frames}</dd>
          </div>
          <div className="flex justify-between gap-4 py-0.5">
            <dt className="text-text-muted">{t("browse.size")}</dt>
            <dd className="font-mono">
              {sizes.size === 1 ? [...sizes][0] : t("browse.mixedSizes")}
            </dd>
          </div>
        </dl>
      </section>
      <div className="relative">
        <Button
          className="w-full justify-center"
          aria-haspopup="menu"
          onClick={() => setMoving(!moving)}
        >
          <BrowserIcon name="folder" />
          {t("command.sprite.move")}
        </Button>
        {moving && <Menu items={moveItems} onClose={() => setMoving(false)} />}
      </div>
      <Button variant="danger" className="justify-center" onClick={onDelete}>
        <BrowserIcon name="trash" />
        {t("command.sprite.deleteMany", { count: sprites.length })}
      </Button>
    </div>
  );
}
