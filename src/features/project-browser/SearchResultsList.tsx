import type { FolderNode, SpriteSummary } from "@/shared/lib/core";
import { useT } from "@/shared/lib/i18n";
import { BrowserIcon } from "./icons";
import { pathKey, spriteKey } from "./paths";
import type { SearchResults } from "./search";

type Props = {
  results: SearchResults;
  selectedSprites: Set<string>;
  onSelectFolder: (folder: FolderNode) => void;
  onSelectSprite: (sprite: SpriteSummary) => void;
};

const where = (path: string[]) => path.slice(0, -1).join(" / ");

// The left side while the person searches: the folders and sprites that match,
// one list, each with the folder it is in.
export function SearchResultsList({
  results,
  selectedSprites,
  onSelectFolder,
  onSelectSprite,
}: Props) {
  const t = useT();
  if (results.folders.length + results.sprites.length === 0) {
    return (
      <p className="px-3 py-2 text-xs text-text-faint">
        {t("browse.noResults")}
      </p>
    );
  }
  const row =
    "flex w-full items-center gap-2 rounded-md px-3 py-1.5 text-left text-sm hover:bg-surface-2";
  return (
    <ul aria-label={t("browse.results")}>
      {results.folders.map((folder) => (
        <li key={`f:${pathKey(folder.path)}`}>
          <button
            type="button"
            onClick={() => onSelectFolder(folder)}
            className={`${row} text-text-muted`}
          >
            <BrowserIcon name="folder" />
            <span className="flex-1 truncate">{folder.name}</span>
            <span className="truncate text-xs text-text-faint">
              {where(folder.path)}
            </span>
          </button>
        </li>
      ))}
      {results.sprites.map((sprite) => {
        const selected = selectedSprites.has(spriteKey(sprite));
        return (
          <li key={`s:${spriteKey(sprite)}`}>
            <button
              type="button"
              aria-pressed={selected}
              onClick={() => onSelectSprite(sprite)}
              className={`${row} ${selected ? "bg-surface-2 font-semibold text-amber" : "text-text-muted"}`}
            >
              <BrowserIcon name="sprite" />
              <span className="flex-1 truncate">{sprite.name}</span>
              <span className="truncate text-xs text-text-faint">
                {sprite.folder.join(" / ")}
              </span>
            </button>
          </li>
        );
      })}
    </ul>
  );
}
