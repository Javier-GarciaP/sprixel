import { Button } from "@/shared/components";
import { useLanguage, useT, type TranslationKey } from "@/shared/lib/i18n";
import { ActivityThumb } from "./ActivityThumb";
import type { ActivityRow, AgentsApi, AgentWrite } from "./types";

// What an action is called, by the tool that made it. A tool that is not
// here (or has no sprite when it needs one) shows the text of the core.
const toolText: Record<string, TranslationKey> = {
  create_folder: "agents.tool.create_folder",
  create_sprite: "agents.tool.create_sprite",
  add_frame: "agents.tool.add_frame",
  apply_grid: "agents.tool.apply_grid",
  set_pixels: "agents.tool.set_pixels",
  draw_shape: "agents.tool.draw_shape",
  auto_outline: "agents.tool.auto_outline",
  auto_shade: "agents.tool.auto_shade",
  delete_frame: "agents.tool.delete_frame",
  create_tag: "agents.tool.create_tag",
  set_palette_color: "agents.tool.set_palette_color",
  delete_sprite: "agents.tool.delete_sprite",
  export: "agents.tool.export",
  undo_last_ai_action: "agents.tool.undo_last_ai_action",
};

// The tools whose text names the sprite.
const needsSprite = new Set([
  "create_sprite",
  "add_frame",
  "apply_grid",
  "set_pixels",
  "draw_shape",
  "auto_outline",
  "auto_shade",
  "delete_frame",
  "create_tag",
  "delete_sprite",
]);

type Props = {
  api: AgentsApi;
  rows: ActivityRow[];
  // The agent that writes now: it is the first row, with Stop.
  writing: AgentWrite | null;
};

// The sprite as a path: "Characters/goblin".
const pathOf = (write: AgentWrite) =>
  [...write.sprite.folder, write.sprite.name].join("/");

// What the agents did, newest first. The action in progress is highlighted
// and has Stop. Only the latest finished action has Undo.
export function ActivityList({ api, rows, writing }: Props) {
  const t = useT();
  const language = useLanguage();
  const time = (ms: number) =>
    new Intl.DateTimeFormat(language, {
      hour: "2-digit",
      minute: "2-digit",
    }).format(ms);
  // The core marks one row with Undo; this keeps it to the first one.
  const undoId = rows.find((row) => row.can_undo)?.id;

  const describe = (row: ActivityRow) => {
    if (row.result === "error") {
      return t("agents.activity.failed", {
        tool: row.tool,
        code: row.error ?? "error",
      });
    }
    const key = toolText[row.tool];
    if (!key || (needsSprite.has(row.tool) && !row.sprite)) return row.summary;
    return t(key, { sprite: row.sprite ?? "" });
  };

  if (!writing && rows.length === 0) {
    return (
      <div className="flex min-h-64 flex-col items-center justify-center rounded-lg border border-border bg-surface px-6 py-12 text-center">
        <p className="font-title text-xl font-bold">
          {t("agents.activity.emptyTitle")}
        </p>
        <p className="mt-2 text-sm text-text-muted">
          {t("agents.activity.emptyText")}
        </p>
      </div>
    );
  }

  return (
    <ul className="flex flex-col gap-3">
      {writing && (
        <li
          aria-label={t("agents.activity.drawing", {
            frame: writing.frame,
            sprite: pathOf(writing),
          })}
          data-testid="in-progress"
          className="flex items-center gap-3 rounded-lg border border-amber bg-surface p-3"
        >
          <ActivityThumb
            api={api}
            sprite={pathOf(writing)}
            label={t("agents.activity.thumb", { sprite: pathOf(writing) })}
          />
          <div className="min-w-0 flex-1">
            <p className="truncate font-semibold">
              {t("agents.activity.drawing", {
                frame: writing.frame,
                sprite: pathOf(writing),
              })}
            </p>
            <p className="truncate text-xs text-amber">{writing.agent}</p>
          </div>
          <Button
            variant="secondary"
            className="border-amber text-amber"
            onClick={() => void api.stop(writing.agent)}
          >
            {t("agents.activity.stop")}
          </Button>
        </li>
      )}
      {rows.map((row) => (
        <li
          key={row.id}
          className={`flex items-center gap-3 rounded-lg border bg-surface p-3 ${row.result === "error" ? "border-error" : "border-border"}`}
        >
          <ActivityThumb
            api={api}
            sprite={row.sprite}
            label={t("agents.activity.thumb", { sprite: row.sprite ?? "" })}
          />
          <div className="min-w-0 flex-1">
            <p className="truncate font-semibold">{describe(row)}</p>
            <p className="truncate text-xs text-text-muted">
              {row.client} · {time(row.at_ms)}
            </p>
          </div>
          {row.id === undoId && (
            <Button onClick={() => void api.undo()}>
              {t("agents.activity.undo")}
            </Button>
          )}
        </li>
      ))}
    </ul>
  );
}
