import type { TranslationKey } from "@/shared/lib/i18n";

export type ToolId =
  | "pencil"
  | "eraser"
  | "bucket"
  | "eyedropper"
  | "line"
  | "rectangle"
  | "ellipse"
  | "select"
  | "move";

// The tools in the order of the tools column. The key is shown next to the
// name; the shortcut itself is in the command registry of the app.
export const tools: { id: ToolId; key: string; labelKey: TranslationKey }[] = [
  { id: "pencil", key: "B", labelKey: "command.tool.pencil" },
  { id: "eraser", key: "E", labelKey: "command.tool.eraser" },
  { id: "bucket", key: "G", labelKey: "command.tool.bucket" },
  { id: "eyedropper", key: "I", labelKey: "command.tool.eyedropper" },
  { id: "line", key: "L", labelKey: "command.tool.line" },
  { id: "rectangle", key: "R", labelKey: "command.tool.rectangle" },
  { id: "ellipse", key: "C", labelKey: "command.tool.ellipse" },
  { id: "select", key: "M", labelKey: "command.tool.select" },
  { id: "move", key: "V", labelKey: "command.tool.move" },
];
