import { useT } from "@/shared/lib/i18n";
import { tools, type ToolId } from "../tools/tools";
import { EditorIcon } from "./icons";

type Props = {
  tool: ToolId;
  onTool: (tool: ToolId) => void;
  main: string;
  secondary: string;
  onSwap: () => void;
};

// The tools on the left, then the main and the secondary color and a button
// to swap them. A tool shows its name and key when the pointer rests on it.
export function ToolsColumn({ tool, onTool, main, secondary, onSwap }: Props) {
  const t = useT();
  return (
    <aside
      aria-label={t("editor.tools")}
      className="flex w-16 shrink-0 flex-col items-center gap-2 border-r border-border bg-bg py-3"
    >
      {tools.map(({ id, key, labelKey }) => {
        const label = t(labelKey);
        return (
          <button
            key={id}
            type="button"
            title={`${label} (${key})`}
            aria-label={label}
            aria-pressed={tool === id}
            onClick={() => onTool(id)}
            className={`flex h-10 w-10 items-center justify-center rounded-md ${tool === id ? "bg-amber text-ink" : "text-text-muted hover:bg-surface-2"}`}
          >
            <EditorIcon name={id} />
          </button>
        );
      })}
      <span className="my-1 h-px w-8 bg-border" aria-hidden />
      <div className="relative h-11 w-11">
        <span
          data-testid="secondary-color"
          aria-label={t("colors.secondary")}
          className="absolute right-0 bottom-0 h-7 w-7 rounded border-2 border-border-strong"
          style={{ background: secondary }}
        />
        <span
          data-testid="main-color"
          aria-label={t("colors.main")}
          className="absolute top-0 left-0 h-7 w-7 rounded border-2 border-text"
          style={{ background: main }}
        />
      </div>
      <button
        type="button"
        title={t("colors.swap")}
        aria-label={t("colors.swap")}
        onClick={onSwap}
        className="flex h-8 w-8 items-center justify-center rounded text-text-muted hover:bg-surface-2"
      >
        <EditorIcon name="swap" className="h-4 w-4" />
      </button>
    </aside>
  );
}
