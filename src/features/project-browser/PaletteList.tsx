import type { PaletteSummary } from "@/shared/lib/core";
import { useT } from "@/shared/lib/i18n";
import { BrowserIcon } from "./icons";

type Props = {
  palettes: PaletteSummary[];
  onOpen: (name: string) => void;
  onNew: () => void;
};

// The palettes of the project, below the folders.
export function PaletteList({ palettes, onOpen, onNew }: Props) {
  const t = useT();
  return (
    <section aria-label={t("browse.palettes")}>
      <div className="mb-1 flex items-center justify-between px-2">
        <h3 className="text-xs font-semibold text-text-muted">
          {t("browse.palettes")}
        </h3>
        <button
          type="button"
          aria-label={t("browse.newPalette")}
          onClick={onNew}
          className="rounded-md border border-border p-1 text-text-muted hover:bg-surface-2"
        >
          <BrowserIcon name="plus" />
        </button>
      </div>
      <ul>
        {palettes.map((palette) => (
          <li key={palette.name}>
            <button
              type="button"
              onClick={() => onOpen(palette.name)}
              className="flex w-full items-center gap-2 rounded-md px-3 py-1.5 text-left text-sm text-text-muted hover:bg-surface-2"
            >
              <BrowserIcon name="palette" />
              <span className="flex-1 truncate">{palette.name}</span>
            </button>
          </li>
        ))}
      </ul>
    </section>
  );
}
