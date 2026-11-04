import { useEffect, useRef, useState } from "react";
import { useT, type TranslationKey } from "@/shared/lib/i18n";
import type { RegionAction } from "../api/types";
import { symmetryModes, type Symmetry } from "../tools/symmetry";
import type { ToolId } from "../tools/tools";
import { tools } from "../tools/tools";
import type { Rect } from "../types";
import { mirrorItems, turnItems } from "./toolbarItems";
import { ToolbarButton } from "./ToolbarButton";

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
  // Symmetry mode: what is drawn is also drawn mirrored.
  symmetry: Symmetry;
  onSymmetry: (symmetry: Symmetry) => void;
  // The panels with a preview, and which one is open.
  panel: "outline" | "shade" | "review" | null;
  onPanel: (panel: "outline" | "shade" | "review" | null) => void;
  // An AI agent is drawing: what to say and what Stop does.
  agent?: { text: string; onStop: () => void } | null;
  // The Canvas and Text switch: the same place in both views.
  mode: "canvas" | "text";
  onMode: (mode: "canvas" | "text") => void;
  // The selected rectangle, or null. The actions on it stay in the bar and
  // are disabled without one, so the bar never changes its height.
  selection: Rect | null;
  onAction: (action: RegionAction) => void;
  onCopy: () => void;
  // Paste needs something copied.
  onPaste: (() => void) | null;
  // Moving with the arrow keys: pixels that leave come back on the other side.
  wrap: boolean;
  onWrap: (wrap: boolean) => void;
  // The shortcut of a command, ready to show ("Ctrl+Z"), if it has one.
  hint?: (command: string) => string | undefined;
};

const symmetryLabels = {
  off: "toolbar.symmetryOff",
  horizontal: "toolbar.symmetryHorizontal",
  vertical: "toolbar.symmetryVertical",
  both: "toolbar.symmetryBoth",
} as const satisfies Record<Symmetry, TranslationKey>;

const symmetryIcons = {
  off: "symmetryOff",
  horizontal: "symmetryHorizontal",
  vertical: "symmetryVertical",
  both: "symmetryBoth",
} as const;

const panels = [
  {
    id: "outline",
    icon: "outline",
    label: "toolbar.outline",
    command: "edit.outline",
  },
  { id: "shade", icon: "shade", label: "toolbar.shade", command: "edit.shade" },
  {
    id: "review",
    icon: "review",
    label: "toolbar.review",
    command: "view.review",
  },
] as const;

const divider = <span className="h-5 w-px shrink-0 bg-border" aria-hidden />;

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
  symmetry,
  onSymmetry,
  panel,
  onPanel,
  agent,
  mode,
  onMode,
  selection,
  onAction,
  onCopy,
  onPaste,
  wrap,
  onWrap,
  hint,
}: Props) {
  const t = useT();
  const [more, setMore] = useState(false);
  const moreRef = useRef<HTMLDivElement>(null);
  // The More popover closes with Escape or a click outside it.
  useEffect(() => {
    if (!more) return;
    const close = (event: Event) => {
      if (
        event instanceof KeyboardEvent
          ? event.key === "Escape"
          : !moreRef.current?.contains(event.target as Node)
      )
        setMore(false);
    };
    window.addEventListener("keydown", close);
    window.addEventListener("mousedown", close);
    return () => {
      window.removeEventListener("keydown", close);
      window.removeEventListener("mousedown", close);
    };
  }, [more]);
  const needSelection = selection ? undefined : t("toolbar.selectFirst");
  const finishing = panels.map(({ id, icon, label, command }) => (
    <ToolbarButton
      key={id}
      icon={icon}
      label={t(label)}
      shortcut={hint?.(command)}
      pressed={panel === id}
      onClick={() => onPanel(panel === id ? null : id)}
    />
  ));
  const selectionTools = (
    <>
      {turnItems.map(({ command, icon, label, action }) => (
        <ToolbarButton
          key={command}
          icon={icon}
          label={t(label)}
          shortcut={hint?.(command)}
          disabled={!selection}
          reason={needSelection}
          onClick={() => onAction(action)}
        />
      ))}
      {divider}
      {mirrorItems.map(({ command, icon, label, action }) => (
        <ToolbarButton
          key={command}
          icon={icon}
          label={t(label)}
          shortcut={hint?.(command)}
          disabled={!selection}
          reason={needSelection}
          onClick={() => onAction(action)}
        />
      ))}
      {divider}
      <ToolbarButton
        icon="copy"
        label={t("command.edit.copy")}
        shortcut={hint?.("edit.copy")}
        onClick={onCopy}
      />
      <ToolbarButton
        icon="paste"
        label={t("command.edit.paste")}
        shortcut={hint?.("edit.paste")}
        disabled={!onPaste}
        reason={t("toolbar.copyFirst")}
        onClick={() => onPaste?.()}
      />
      <ToolbarButton
        icon="wrap"
        label={t("toolbar.wrap")}
        shortcut={hint?.("selection.wrap")}
        pressed={wrap}
        onClick={() => onWrap(!wrap)}
      />
    </>
  );
  const active = tools.find((entry) => entry.id === tool)!;
  return (
    <div className="@container h-12 shrink-0 border-b border-border bg-bg">
      <div
        role="toolbar"
        aria-label={t("toolbar.label")}
        className="flex h-full items-center gap-2 px-4"
      >
        <span className="max-w-32 min-w-0 truncate font-title text-sm font-bold">
          {name}
        </span>
        <span className="shrink-0 font-mono text-xs text-text-muted">
          {t("editor.size", { width, height })}
        </span>
        {divider}
        <ToolbarButton
          icon="undo"
          label={t("command.edit.undo")}
          shortcut={hint?.("edit.undo")}
          disabled={!canUndo}
          reason={t("toolbar.nothingToUndo")}
          onClick={onUndo}
        />
        <ToolbarButton
          icon="redo"
          label={t("command.edit.redo")}
          shortcut={hint?.("edit.redo")}
          disabled={!canRedo}
          reason={t("toolbar.nothingToRedo")}
          onClick={onRedo}
        />
        {divider}
        <div
          role="group"
          aria-label={t("editor.symmetry")}
          className="flex shrink-0 rounded-md border border-border bg-surface p-0.5"
        >
          {symmetryModes.map((value) => (
            <ToolbarButton
              key={value}
              icon={symmetryIcons[value]}
              label={t(symmetryLabels[value])}
              pressed={symmetry === value}
              onClick={() => onSymmetry(value)}
            />
          ))}
        </div>
        <div
          role="group"
          aria-label={t("editor.finish")}
          className="hidden shrink-0 items-center gap-1 @min-[700px]:flex"
        >
          {divider}
          {finishing}
        </div>
        <div
          role="group"
          aria-label={t("selection.bar")}
          className="hidden shrink-0 items-center gap-1 @min-[1100px]:flex"
        >
          {divider}
          {selectionTools}
        </div>
        <div ref={moreRef} className="relative shrink-0 @min-[1100px]:hidden">
          <ToolbarButton
            icon="more"
            label={t("toolbar.more")}
            pressed={more}
            onClick={() => setMore(!more)}
          />
          {more && (
            <div
              role="group"
              aria-label={t("toolbar.more")}
              className="absolute top-full left-0 z-20 mt-1.5 flex max-w-80 flex-wrap items-center gap-1 rounded-lg border border-border-strong bg-surface p-1.5 shadow-lg"
            >
              <div className="flex items-center gap-1 @min-[700px]:hidden">
                {finishing}
                {divider}
              </div>
              {selectionTools}
            </div>
          )}
        </div>
        <span
          className="hidden text-xs text-text-muted @min-[900px]:inline"
          data-testid="active-tool"
        >
          {t("editor.activeTool", { name: t(active.labelKey) })}
        </span>
        {(tool === "rectangle" || tool === "ellipse") && (
          <div
            role="group"
            aria-label={t("editor.shapeStyle")}
            className="flex shrink-0 rounded-md border border-border bg-surface p-0.5 text-xs"
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
            className="flex shrink-0 items-center gap-2 rounded-full border border-amber bg-surface px-3 py-1 text-xs"
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
        <div
          role="group"
          aria-label={t("editor.viewMode")}
          className="ml-auto flex shrink-0 rounded-md border border-border bg-surface p-0.5 text-xs"
        >
          {(["canvas", "text"] as const).map((value) => (
            <button
              key={value}
              type="button"
              aria-pressed={mode === value}
              onClick={() => onMode(value)}
              className={`rounded px-3 py-1 ${mode === value ? "font-semibold text-text" : "text-text-muted"}`}
            >
              {t(value === "canvas" ? "editor.canvasMode" : "editor.textMode")}
            </button>
          ))}
        </div>
      </div>
    </div>
  );
}
