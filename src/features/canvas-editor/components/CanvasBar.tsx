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
  // Rectangle and Ellipse can be outlined or filled.
  filled: boolean;
  onFilled: (filled: boolean) => void;
  // An AI agent is drawing: what to say and what Stop does.
  agent?: { text: string; onStop: () => void } | null;
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
  filled,
  onFilled,
  agent,
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
      {(tool === "rectangle" || tool === "ellipse") && (
        <div
          role="group"
          aria-label={t("editor.shapeStyle")}
          className="flex rounded-md border border-border bg-surface p-0.5 text-xs"
        >
          {[false, true].map((value) => (
            <button
              key={String(value)}
              type="button"
              aria-pressed={filled === value}
              onClick={() => onFilled(value)}
              className={`rounded px-2 py-1 ${filled === value ? "bg-surface-2 font-semibold text-text" : "text-text-muted"}`}
            >
              {t(value ? "editor.filled" : "editor.outline")}
            </button>
          ))}
        </div>
      )}
      {agent && (
        <div
          role="status"
          className="flex items-center gap-2 rounded-full border border-amber bg-surface px-3 py-1 text-xs"
        >
          <span className="h-2 w-2 rounded-full bg-amber" aria-hidden />
          <span className="text-amber">{agent.text}</span>
          <button
            type="button"
            onClick={agent.onStop}
            className="border-l border-amber pl-2 font-semibold text-text"
          >
            {t("marker.stop")}
          </button>
        </div>
      )}
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
