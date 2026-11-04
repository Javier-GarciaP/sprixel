import { useT, type TranslationKey } from "@/shared/lib/i18n";
import type { RegionAction } from "../api/types";
import type { Rect } from "../types";
import { Icon } from "@/shared/components";
import { EditorIcon, type EditorIconName } from "./icons";

const barButton =
  "flex h-7 w-7 items-center justify-center rounded-md border border-border bg-surface text-text";

type Props = {
  selection: Rect;
  // Moving with the arrow keys: pixels that leave come back on the other side.
  wrap: boolean;
  onWrap: (wrap: boolean) => void;
  onAction: (action: RegionAction) => void;
  onCopy: () => void;
  // Paste needs something copied.
  onPaste: (() => void) | null;
};

const buttons: {
  icon: EditorIconName;
  label: TranslationKey;
  action: RegionAction;
}[] = [
  {
    icon: "flipHorizontal",
    label: "command.selection.flipHorizontal",
    action: { kind: "flip_horizontal" },
  },
  {
    icon: "flipVertical",
    label: "command.selection.flipVertical",
    action: { kind: "flip_vertical" },
  },
  {
    icon: "rotateLeft",
    label: "command.selection.rotate270",
    action: { kind: "rotate", degrees: 270 },
  },
  {
    icon: "rotate180",
    label: "command.selection.rotate180",
    action: { kind: "rotate", degrees: 180 },
  },
  {
    icon: "rotateRight",
    label: "command.selection.rotate90",
    action: { kind: "rotate", degrees: 90 },
  },
  {
    icon: "mirrorLeftToRight",
    label: "command.selection.mirrorLeftToRight",
    action: { kind: "mirror", direction: "left_to_right" },
  },
  {
    icon: "mirrorRightToLeft",
    label: "command.selection.mirrorRightToLeft",
    action: { kind: "mirror", direction: "right_to_left" },
  },
  {
    icon: "mirrorTopToBottom",
    label: "command.selection.mirrorTopToBottom",
    action: { kind: "mirror", direction: "top_to_bottom" },
  },
  {
    icon: "mirrorBottomToTop",
    label: "command.selection.mirrorBottomToTop",
    action: { kind: "mirror", direction: "bottom_to_top" },
  },
];

// The bar under the canvas bar while a rectangle is selected: flip, turn and
// mirror what is inside it. Each button is one action of the core, so one
// click is one step of Undo.
export function SelectionBar({
  selection,
  wrap,
  onWrap,
  onAction,
  onCopy,
  onPaste,
}: Props) {
  const t = useT();
  return (
    <div
      role="toolbar"
      aria-label={t("selection.bar")}
      className="flex h-10 shrink-0 items-center gap-1 border-b border-border bg-bg px-4"
    >
      <span className="mr-2 font-mono text-xs text-text-muted">
        {t("selection.size", {
          width: selection.width,
          height: selection.height,
        })}
      </span>
      {buttons.map(({ icon, label, action }) => (
        <button
          key={label}
          type="button"
          aria-label={t(label)}
          title={t(label)}
          onClick={() => onAction(action)}
          className={barButton}
        >
          <EditorIcon name={icon} className="h-4 w-4" />
        </button>
      ))}
      <span className="mx-1 h-5 border-l border-border" aria-hidden />
      <button
        type="button"
        aria-label={t("command.edit.copy")}
        title={t("command.edit.copy")}
        onClick={onCopy}
        className={barButton}
      >
        <Icon name="copy" className="h-4 w-4" />
      </button>
      <button
        type="button"
        aria-label={t("command.edit.paste")}
        title={t("command.edit.paste")}
        disabled={!onPaste}
        onClick={() => onPaste?.()}
        className={`${barButton} disabled:cursor-not-allowed disabled:opacity-40`}
      >
        <EditorIcon name="paste" className="h-4 w-4" />
      </button>
      <label className="ml-3 flex items-center gap-1 text-xs text-text-muted">
        <input
          type="checkbox"
          checked={wrap}
          onChange={(event) => onWrap(event.target.checked)}
        />
        {t("selection.wrapOn")}
      </label>
    </div>
  );
}
