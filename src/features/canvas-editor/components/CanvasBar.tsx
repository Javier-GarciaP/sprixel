import { useT } from "@/shared/lib/i18n";
import type { ToolId } from "../tools/tools";
import { tools } from "../tools/tools";
import { EditorIcon } from "./icons";

type Props = {
  name: string;
  width: number;
  height: number;
  tool: ToolId;
  canUndo: boolean;
  canRedo: boolean;
  onUndo: () => void;
  onRedo: () => void;
};

const iconButton =
  "flex h-8 w-8 items-center justify-center rounded-md border border-border bg-surface text-text disabled:cursor-not-allowed disabled:opacity-40";

// The bar above the canvas: the name, the size (shown, never a menu, because
// the size is fixed when the sprite is created), undo and redo, the active
// tool and the Canvas and Text switch.
export function CanvasBar({
  name,
  width,
  height,
  tool,
  canUndo,
  canRedo,
  onUndo,
  onRedo,
}: Props) {
  const t = useT();
  const active = tools.find((entry) => entry.id === tool)!;
  return (
    <div className="flex h-12 shrink-0 items-center gap-3 border-b border-border bg-bg px-4">
      <span className="font-title text-sm font-bold">{name}</span>
      <span className="font-mono text-xs text-text-muted">
        {t("editor.size", { width, height })}
      </span>
      <button
        type="button"
        className={iconButton}
        aria-label={t("command.edit.undo")}
        title={t("command.edit.undo")}
        disabled={!canUndo}
        onClick={onUndo}
      >
        <EditorIcon name="undo" className="h-4 w-4" />
      </button>
      <button
        type="button"
        className={iconButton}
        aria-label={t("command.edit.redo")}
        title={t("command.edit.redo")}
        disabled={!canRedo}
        onClick={onRedo}
      >
        <EditorIcon name="redo" className="h-4 w-4" />
      </button>
      <span className="text-xs text-text-muted" data-testid="active-tool">
        {t("editor.activeTool", { name: t(active.labelKey) })}
      </span>
      <div className="ml-auto flex rounded-md border border-border bg-surface p-0.5 text-xs">
        <button
          type="button"
          aria-pressed="true"
          className="rounded px-3 py-1 font-semibold text-text"
        >
          {t("editor.canvasMode")}
        </button>
        <button
          type="button"
          disabled
          title={t("editor.textSoon")}
          className="rounded px-3 py-1 text-text-faint disabled:cursor-not-allowed"
        >
          {t("editor.textMode")}
        </button>
      </div>
    </div>
  );
}
