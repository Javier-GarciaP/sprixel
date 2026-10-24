import { useState } from "react";
import type { MenuItem } from "@/shared/components";
import type {
  FolderNode,
  ProjectTree,
  SpriteSummary,
  SpriteTarget,
} from "@/shared/lib/core";
import { useT } from "@/shared/lib/i18n";
import { BrowserIcon } from "./icons";
import { ItemMenu } from "./ItemMenu";
import { DRAG_TYPE, itemCount, pathKey, spriteKey } from "./paths";

type Props = {
  tree: ProjectTree;
  // The folder shown in the middle area.
  selectedFolder: string[];
  selectedSprites: Set<string>;
  expanded: Set<string>;
  onSelectFolder: (path: string[]) => void;
  onToggle: (path: string[]) => void;
  onSelectSprite: (sprite: SpriteSummary) => void;
  // Double-clicking a name renames it.
  onRenameFolder: (path: string[]) => void;
  onRenameSprite: (sprite: SpriteSummary) => void;
  folderMenu: (path: string[]) => MenuItem[];
  // Sprite cards dropped on a folder move there.
  onDropSprites: (sprites: SpriteTarget[], to: string[]) => void;
};

const indent = (depth: number) => ({ paddingLeft: `${8 + depth * 16}px` });

// The folders of the project with the number of items next to each one, and
// the sprites inside the folders that are open, with their number of frames.
export function FolderTree(props: Props) {
  const t = useT();
  const { tree } = props;
  return (
    <div role="tree" aria-label={t("browse.folders")} className="text-sm">
      <FolderRow {...props} folder={tree.root} label={tree.name} depth={0} />
      {tree.root.folders.length === 0 && (
        <p className="px-3 py-2 text-xs text-text-faint">
          {t("browse.noFolders")}
        </p>
      )}
    </div>
  );
}

function FolderRow({
  folder,
  label,
  depth,
  ...props
}: Props & { folder: FolderNode; label: string; depth: number }) {
  const t = useT();
  const key = pathKey(folder.path);
  // The root is always open.
  const open = depth === 0 || props.expanded.has(key);
  const selected = pathKey(props.selectedFolder) === key;
  const name = label || folder.name;
  const [over, setOver] = useState(false);

  return (
    <>
      <div
        role="treeitem"
        aria-selected={selected}
        aria-expanded={open}
        className={`group flex items-center gap-1 rounded-md pr-2 ${over ? "outline outline-2 outline-amber" : ""} ${selected ? "bg-surface-2 font-semibold text-text" : "text-text-muted hover:bg-surface-2"}`}
        style={indent(depth)}
        onDragOver={(event) => {
          if (!event.dataTransfer.types.includes(DRAG_TYPE)) return;
          event.preventDefault();
          setOver(true);
        }}
        onDragLeave={() => setOver(false)}
        onDrop={(event) => {
          setOver(false);
          const data = event.dataTransfer.getData(DRAG_TYPE);
          if (!data) return;
          event.preventDefault();
          props.onDropSprites(JSON.parse(data) as SpriteTarget[], folder.path);
        }}
      >
        {depth > 0 ? (
          <button
            type="button"
            aria-label={t(open ? "browse.collapse" : "browse.expand", { name })}
            onClick={() => props.onToggle(folder.path)}
            className="p-0.5"
          >
            <BrowserIcon
              name="chevron"
              className={`h-3 w-3 ${open ? "rotate-90" : ""}`}
            />
          </button>
        ) : (
          <span className="w-4" aria-hidden />
        )}
        <button
          type="button"
          onClick={() => props.onSelectFolder(folder.path)}
          onDoubleClick={() => depth > 0 && props.onRenameFolder(folder.path)}
          className={`flex flex-1 items-center gap-2 py-1.5 text-left ${selected ? "text-amber" : ""}`}
        >
          <BrowserIcon name="folder" />
          <span className="flex-1 truncate">{name}</span>
          <span className="font-mono text-xs text-text-faint">
            {itemCount(folder)}
          </span>
        </button>
        {depth > 0 && (
          <ItemMenu
            label={t("browse.moreActions", { name })}
            items={props.folderMenu(folder.path)}
          />
        )}
      </div>
      {open && (
        <div role="group">
          {folder.folders.map((child) => (
            <FolderRow
              key={pathKey(child.path)}
              {...props}
              folder={child}
              label=""
              depth={depth + 1}
            />
          ))}
          {folder.sprites.map((sprite) => (
            <SpriteRow
              key={spriteKey(sprite)}
              sprite={sprite}
              depth={depth + 1}
              selected={props.selectedSprites.has(spriteKey(sprite))}
              onSelect={props.onSelectSprite}
              onRename={props.onRenameSprite}
            />
          ))}
        </div>
      )}
    </>
  );
}

function SpriteRow({
  sprite,
  depth,
  selected,
  onSelect,
  onRename,
}: {
  sprite: SpriteSummary;
  depth: number;
  selected: boolean;
  onSelect: (sprite: SpriteSummary) => void;
  onRename: (sprite: SpriteSummary) => void;
}) {
  return (
    <div
      role="treeitem"
      aria-selected={selected}
      className={`flex items-center rounded-md pr-2 ${selected ? "bg-surface-2 font-semibold text-text" : "text-text-muted hover:bg-surface-2"}`}
      style={indent(depth)}
    >
      <span className="w-4" aria-hidden />
      <button
        type="button"
        onClick={() => onSelect(sprite)}
        onDoubleClick={() => onRename(sprite)}
        className={`flex flex-1 items-center gap-2 py-1.5 text-left ${selected ? "text-amber" : ""}`}
      >
        <BrowserIcon name="sprite" />
        <span className="flex-1 truncate">{sprite.name}</span>
        <span className="font-mono text-xs text-text-faint">
          {sprite.frames}
        </span>
      </button>
    </div>
  );
}
