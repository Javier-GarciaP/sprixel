import { useEffect, useMemo, useRef, useState } from "react";
import { Button, Icon } from "@/shared/components";
import {
  asCoreFailure,
  type ProjectTree,
  type SpriteTarget,
} from "@/shared/lib/core";
import { useT } from "@/shared/lib/i18n";
import { coreExportApi } from "./api";
import { OptionsStep } from "./OptionsStep";
import { ScopeStep } from "./ScopeStep";
import {
  countProject,
  defaultKind,
  existing,
  findFolder,
  folderChoices,
  toScope,
} from "./scope";
import { SheetPreview } from "./SheetPreview";
import { filesText, framesText, spritesText } from "./text";
import type {
  ExportApi,
  Format,
  Options,
  Preview,
  ScopeKind,
  Summary,
  Written,
} from "./types";

const DEFAULT_OUTPUT = "assets/sprites";
const MAX_LISTED = 6;

type Props = {
  api?: ExportApi;
  // The folder open in the Project tab and what is selected there.
  folder: string[];
  selection: SpriteTarget[];
  onGoToProject: () => void;
};

// The Export tab: what to export, the format and the options on the left, the
// preview and the files on the right.
export function ExportScreen({ api = coreExportApi, ...props }: Props) {
  const [tree, setTree] = useState<ProjectTree | null>(null);

  // The tree is read once and again after every change of the project.
  useEffect(() => {
    let stopped = false;
    let unlisten: (() => void) | undefined;
    const load = () =>
      api
        .loadTree()
        .then((next) => {
          if (!stopped) setTree(next);
        })
        .catch(() => {});
    load();
    api.onChanged(load).then((stop) => {
      if (stopped) stop();
      else unlisten = stop;
    });
    return () => {
      stopped = true;
      unlisten?.();
    };
  }, [api]);

  if (!tree) return null;
  return <Export api={api} tree={tree} {...props} />;
}

function Export({
  api,
  tree,
  folder: openFolder,
  selection: selected,
  onGoToProject,
}: Props & { api: ExportApi; tree: ProjectTree }) {
  const t = useT();
  const folders = useMemo(() => folderChoices(tree), [tree]);
  const selection = useMemo(() => existing(tree, selected), [tree, selected]);
  const [chosenKind, setChosenKind] = useState<ScopeKind | null>(null);
  const [chosenFolder, setChosenFolder] = useState<string[] | null>(null);
  const [format, setFormat] = useState<Format>("sheet");
  const [options, setOptions] = useState<Options>({
    scale: 1,
    spacing: 1,
    naming: null,
    jsonFormat: "sprixel",
  });
  const [output, setOutput] = useState(DEFAULT_OUTPUT);
  const [summary, setSummary] = useState<Summary | null>(null);
  const [preview, setPreview] = useState<Preview | null>(null);
  const [busy, setBusy] = useState(false);
  // How the last export ended. It goes away when the request changes.
  const [result, setResult] = useState<
    | { kind: "ok"; written: Written }
    | { kind: "error"; code: string; message: string }
    | null
  >(null);
  const [editingOutput, setEditingOutput] = useState(false);
  const [focusTick, setFocusTick] = useState(0);
  const outputBox = useRef<HTMLInputElement>(null);

  // The folder: the one chosen, else the open one, else the first.
  const folderPath =
    [chosenFolder, openFolder.length > 0 ? openFolder : null]
      .map((path) => (path && findFolder(tree, path) ? path : null))
      .find((path) => path !== null) ??
    folders[0]?.path ??
    [];
  const folderNode = folders.length > 0 ? findFolder(tree, folderPath) : null;
  const folderCount = folders.find(
    (f) => f.path.join("/") === folderPath.join("/"),
  )?.count;

  let kind =
    chosenKind ?? defaultKind(selection.length > 0, openFolder.length > 0);
  if (kind === "sprites" && selection.length === 0) kind = "project";
  if (kind === "folder" && !folderNode) kind = "project";

  const scope = toScope(kind, folderPath, selection);
  const request = JSON.stringify([scope, format, options]);

  // What the export would do and how it looks. A late answer for an old
  // request is dropped.
  useEffect(() => {
    let stale = false;
    const [asked, askedFormat, askedOptions] = JSON.parse(request);
    api
      .summary(asked, askedFormat, askedOptions)
      .then((next) => !stale && setSummary(next))
      .catch(() => !stale && setSummary(null));
    api
      .preview(asked, askedOptions)
      .then((next) => !stale && setPreview(next))
      .catch(() => !stale && setPreview(null));
    return () => {
      stale = true;
    };
  }, [api, request, tree.revision]);

  // A result describes one request: when the scope, the format, the options
  // or the folder change, it does not match the screen anymore.
  useEffect(() => setResult(null), [request, output]);

  // "Change" and "Change folder": the box can be typed in and has the focus.
  useEffect(() => {
    if (focusTick > 0) outputBox.current?.focus();
  }, [focusTick]);
  const editOutput = () => {
    setEditingOutput(true);
    setFocusTick((n) => n + 1);
  };

  const chooseKind = (next: ScopeKind) => setChosenKind(next);

  const run = async () => {
    setBusy(true);
    try {
      const written = await api.run(scope, format, options, output);
      setResult({ kind: "ok", written });
    } catch (error) {
      const { code, message } = asCoreFailure(error);
      setResult({ kind: "error", code, message });
    } finally {
      setBusy(false);
    }
  };

  const total = countProject(tree);
  const choices = [
    {
      kind: "project" as const,
      title: t("export.scope.project"),
      detail: spritesText(t, total),
      disabled: false,
    },
    {
      kind: "folder" as const,
      title: t("export.scope.folder"),
      detail: folderNode
        ? `${folderNode.name} · ${spritesText(t, folderCount ?? 0)}`
        : t("export.scope.noFolders"),
      disabled: !folderNode,
    },
    {
      kind: "sprites" as const,
      title: t("export.scope.selection"),
      detail: t("export.scope.chosen", { count: selection.length }),
      disabled: selection.length === 0,
    },
  ];

  const frames = summary?.frames ?? 0;
  const files = summary?.files ?? [];
  const listed = files.slice(0, MAX_LISTED);

  return (
    <div className="grid h-full grid-cols-[minmax(0,28rem)_1fr]">
      <aside className="flex flex-col gap-5 overflow-auto border-r border-border p-6">
        <fieldset
          disabled={total === 0}
          className="m-0 flex min-w-0 flex-col gap-5 border-0 p-0 disabled:opacity-50"
        >
          <Step number={1} title={t("export.what")}>
            <ScopeStep
              choices={choices}
              value={kind}
              onChange={chooseKind}
              folders={folders}
              folder={folderPath}
              onFolder={setChosenFolder}
            />
          </Step>
          <Step number={2} title={t("export.format")}>
            <div className="grid grid-cols-3 gap-2">
              {(["sheet", "png"] as const).map((id) => (
                <button
                  key={id}
                  type="button"
                  aria-pressed={format === id}
                  onClick={() => setFormat(id)}
                  className={`flex items-center justify-between gap-2 rounded-md border px-3 py-3 text-left text-sm font-semibold ${format === id ? "border-amber bg-surface" : "border-border bg-surface"}`}
                >
                  {t(
                    id === "sheet"
                      ? "export.format.sheet"
                      : "export.format.png",
                  )}
                  {format === id && (
                    <Icon name="check" className="h-4 w-4 text-amber" />
                  )}
                </button>
              ))}
              <button
                type="button"
                disabled
                title={t("export.format.gifSoon")}
                className="rounded-md border border-border bg-surface px-3 py-3 text-left text-sm font-semibold disabled:opacity-40"
              >
                {t("export.format.gif")}
              </button>
            </div>
            <p className="mt-2 text-xs text-text-muted">
              {t(
                format === "sheet"
                  ? "export.format.sheetHint"
                  : "export.format.pngHint",
              )}
            </p>
          </Step>
          <Step number={3} title={t("export.options")}>
            <OptionsStep
              options={options}
              onOptions={setOptions}
              output={output}
              onOutput={setOutput}
              noSpacing={format === "png"}
              editing={editingOutput}
              onEdit={editOutput}
              invalid={result?.kind === "error"}
              outputRef={outputBox}
            />
          </Step>
        </fieldset>
        <p className="text-xs text-text-muted">
          {t("export.summary", {
            frames: framesText(t, frames),
            files: filesText(t, files.length),
          })}
        </p>
        <Button
          variant="primary"
          className="justify-center py-3"
          disabled={busy || !summary || summary.sprites === 0}
          onClick={run}
        >
          {!busy && <Icon name="download" />}
          {busy
            ? t("export.exporting")
            : result?.kind === "ok"
              ? t("export.again")
              : result?.kind === "error"
                ? t("export.retry")
                : t("export.export")}
        </Button>
      </aside>
      <section className="flex flex-col gap-4 overflow-auto bg-surface-2 p-6">
        <div className="flex items-baseline justify-between">
          <h2 className="font-title text-lg font-semibold">
            {result?.kind === "ok" ? t("export.ready") : t("export.preview")}
          </h2>
          {summary && summary.sprites > 0 && format === "sheet" && (
            <span className="font-mono text-xs text-text-muted">
              {t("export.previewSize", {
                w: summary.sheetWidth,
                h: summary.sheetHeight,
                frames: framesText(t, frames),
              })}
            </span>
          )}
        </div>
        {result?.kind === "ok" && (
          <Banner
            tone="success"
            title={
              result.written.files.length === 1
                ? t("export.saved.one")
                : t("export.saved.other", {
                    count: result.written.files.length,
                  })
            }
            detail={result.written.output}
            action={t("export.open")}
            onAction={() => api.reveal(result.written.output).catch(() => {})}
          />
        )}
        {result?.kind === "error" && (
          <Banner
            tone="error"
            title={t("export.failed")}
            detail={errorText(t, result.code, result.message, output)}
            action={t("export.changeFolder")}
            onAction={editOutput}
          />
        )}
        {(summary?.sprites === 0 || total === 0) && (
          <EmptyPreview onGoToProject={onGoToProject} />
        )}
        {preview && summary && summary.sprites > 0 && (
          <div className="overflow-auto rounded-md border border-border bg-bg p-4">
            <SheetPreview preview={preview} />
          </div>
        )}
        {listed.length > 0 && (
          <section
            aria-label={t("export.filesTitle")}
            className="rounded-md border border-border bg-surface p-3"
          >
            <h3 className="mb-2 text-xs font-semibold">
              {t("export.filesTitle")}
            </h3>
            <ul className="flex flex-col gap-1">
              {listed.map((name) => (
                <li key={name} className="flex gap-3 text-xs">
                  <span className="font-mono">{name}</span>
                  <span className="text-text-muted">
                    {name.endsWith(".json")
                      ? t("export.filesJson")
                      : format === "sheet"
                        ? t("export.filesSheet", {
                            frames: framesText(t, frames),
                          })
                        : t("export.filesFrame")}
                  </span>
                </li>
              ))}
              {files.length > listed.length && (
                <li className="text-xs text-text-muted">
                  {t("export.filesMore", {
                    count: files.length - listed.length,
                  })}
                </li>
              )}
            </ul>
          </section>
        )}
      </section>
    </div>
  );
}

function Step({
  number,
  title,
  children,
}: {
  number: number;
  title: string;
  children: React.ReactNode;
}) {
  return (
    <div className="flex flex-col gap-3">
      <h2 className="flex items-center gap-2 font-title text-lg font-semibold">
        <span className="rounded bg-amber/30 px-2 text-sm">{number}</span>
        {title}
      </h2>
      {children}
    </div>
  );
}

function errorText(
  t: ReturnType<typeof useT>,
  code: string,
  message: string,
  folder: string,
) {
  switch (code) {
    case "no_permission":
      return t("export.error.no_permission", { folder });
    case "no_space":
      return t("export.error.no_space", { folder });
    case "outside_project":
      return t("export.error.outside_project");
    case "invalid_path":
      return t("export.error.invalid_path", { message });
    default:
      return message;
  }
}

// The green or red message above the preview (screens 26 and 27).
function Banner({
  tone,
  title,
  detail,
  action,
  onAction,
}: {
  tone: "success" | "error";
  title: string;
  detail: string;
  action: string;
  onAction: () => void;
}) {
  return (
    <div
      role={tone === "error" ? "alert" : "status"}
      className={`flex items-center justify-between gap-4 rounded-md border px-4 py-3 ${tone === "error" ? "border-error bg-error/10" : "border-success bg-success/10"}`}
    >
      <div className="flex items-center gap-3">
        {tone === "success" && (
          <span className="flex h-9 w-9 shrink-0 items-center justify-center rounded-full bg-success text-bg">
            <Icon name="check" className="h-5 w-5" />
          </span>
        )}
        <div>
          <p className="text-sm font-semibold">{title}</p>
          <p className="font-mono text-xs text-text-muted">{detail}</p>
        </div>
      </div>
      <Button
        variant={tone === "error" ? "secondary" : "primary"}
        onClick={onAction}
      >
        {tone === "success" && <Icon name="folder" />}
        {action}
      </Button>
    </div>
  );
}

// Screen 25: nothing to show, and the way to fix it.
function EmptyPreview({ onGoToProject }: { onGoToProject: () => void }) {
  const t = useT();
  return (
    <div className="flex flex-1 flex-col items-center justify-center gap-2 rounded-md border border-border bg-bg p-10 text-center">
      <h3 className="font-title text-xl font-semibold">
        {t("export.empty.title")}
      </h3>
      <p className="text-sm text-text-muted">{t("export.empty.text")}</p>
      <Button variant="primary" onClick={onGoToProject}>
        {t("export.empty.go")}
      </Button>
    </div>
  );
}
