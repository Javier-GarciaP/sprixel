import { Tooltip } from "@/shared/components";
import { EditorIcon, type EditorIconName } from "./icons";

type Props = {
  icon: EditorIconName;
  label: string;
  shortcut?: string;
  // Makes it a toggle (the active state shows).
  pressed?: boolean;
  disabled?: boolean;
  // Why it is disabled, said in the tooltip.
  reason?: string;
  onClick: () => void;
};

// An icon button of the top bar: same size, hover, active state and focus ring
// as the buttons of the left toolbar, with a tooltip for its name and key.
// A disabled button stays in its place, so the bar never changes.
export function ToolbarButton({
  icon,
  label,
  shortcut,
  pressed,
  disabled,
  reason,
  onClick,
}: Props) {
  return (
    <Tooltip
      label={label}
      shortcut={shortcut}
      reason={disabled ? reason : undefined}
    >
      <button
        type="button"
        aria-label={label}
        aria-pressed={pressed}
        disabled={disabled}
        onClick={onClick}
        className={`flex h-8 w-8 items-center justify-center rounded-md outline-none focus-visible:ring-2 focus-visible:ring-amber disabled:cursor-not-allowed disabled:opacity-40 ${pressed ? "bg-amber text-ink" : "text-text-muted enabled:hover:bg-surface-2"}`}
      >
        <EditorIcon name={icon} className="h-4 w-4" />
      </button>
    </Tooltip>
  );
}
