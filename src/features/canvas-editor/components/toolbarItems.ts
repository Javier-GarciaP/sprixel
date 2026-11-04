import type { TranslationKey } from "@/shared/lib/i18n";
import type { RegionAction } from "../api/types";
import type { EditorIconName } from "./icons";

export type SelectionItem = {
  // The id of the command (its shortcut comes from there).
  command: string;
  icon: EditorIconName;
  label: TranslationKey;
  action: RegionAction;
};

const item = (
  command: string,
  icon: EditorIconName,
  action: RegionAction,
): SelectionItem => ({
  command: `selection.${command}`,
  icon,
  label: `command.selection.${command}` as TranslationKey,
  action,
});

// The turn and flip actions, then the four mirror halves (the least used).
export const turnItems: SelectionItem[] = [
  item("flipHorizontal", "flipHorizontal", { kind: "flip_horizontal" }),
  item("flipVertical", "flipVertical", { kind: "flip_vertical" }),
  item("rotate270", "rotateLeft", { kind: "rotate", degrees: 270 }),
  item("rotate180", "rotate180", { kind: "rotate", degrees: 180 }),
  item("rotate90", "rotateRight", { kind: "rotate", degrees: 90 }),
];

export const mirrorItems: SelectionItem[] = [
  item("mirrorLeftToRight", "mirrorLeftToRight", {
    kind: "mirror",
    direction: "left_to_right",
  }),
  item("mirrorRightToLeft", "mirrorRightToLeft", {
    kind: "mirror",
    direction: "right_to_left",
  }),
  item("mirrorTopToBottom", "mirrorTopToBottom", {
    kind: "mirror",
    direction: "top_to_bottom",
  }),
  item("mirrorBottomToTop", "mirrorBottomToTop", {
    kind: "mirror",
    direction: "bottom_to_top",
  }),
];
