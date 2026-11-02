import { useRef, useState } from "react";
import { Button } from "@/shared/components";
import { useT } from "@/shared/lib/i18n";
import type { Options } from "./types";

const scales: Options["scale"][] = [1, 2, 4, 8];

type Props = {
  options: Options;
  onOptions: (options: Options) => void;
  output: string;
  onOutput: (output: string) => void;
  // True when the format has no sheet to space (PNG files).
  noSpacing: boolean;
  // The output folder box was asked to be edited (by "Change folder").
  outputRef?: React.RefObject<HTMLInputElement | null>;
};

// Step 3: scale, space between frames, the output folder and, in "More
// options", the naming pattern.
export function OptionsStep({
  options,
  onOptions,
  output,
  onOutput,
  noSpacing,
  outputRef,
}: Props) {
  const t = useT();
  const [editing, setEditing] = useState(false);
  const [more, setMore] = useState(false);
  const fallback = useRef<HTMLInputElement>(null);
  const input = outputRef ?? fallback;

  const spacing = (text: string) => {
    const value = Math.round(Number(text));
    onOptions({
      ...options,
      spacing: Number.isFinite(value) ? Math.min(16, Math.max(0, value)) : 0,
    });
  };

  return (
    <div className="flex flex-col gap-3">
      <div className="flex items-center justify-between text-sm">
        <span>{t("export.scale")}</span>
        <div className="flex gap-1">
          {scales.map((scale) => (
            <button
              key={scale}
              type="button"
              aria-pressed={options.scale === scale}
              onClick={() => onOptions({ ...options, scale })}
              className={`rounded border px-2 py-0.5 font-mono text-xs ${options.scale === scale ? "border-amber bg-amber/20" : "border-border bg-surface-2 text-text-muted"}`}
            >
              {scale}x
            </button>
          ))}
        </div>
      </div>
      <label className="flex items-center justify-between text-sm">
        {t("export.spacing")}
        <input
          type="number"
          min={0}
          max={16}
          value={options.spacing}
          disabled={noSpacing}
          onChange={(event) => spacing(event.target.value)}
          className="w-20 rounded-md border border-border bg-surface-2 px-2 py-1 text-sm"
        />
      </label>
      <div className="flex flex-col gap-1 text-sm">
        <label htmlFor="export-output">{t("export.output")}</label>
        <div className="flex gap-2">
          <input
            id="export-output"
            ref={input}
            value={output}
            readOnly={!editing}
            onChange={(event) => onOutput(event.target.value)}
            className="min-w-0 flex-1 rounded-md border border-border bg-surface-2 px-2 py-1.5 text-sm"
          />
          <Button
            onClick={() => {
              setEditing(true);
              input.current?.focus();
            }}
          >
            {t("export.change")}
          </Button>
        </div>
      </div>
      <button
        type="button"
        aria-expanded={more}
        onClick={() => setMore(!more)}
        className="text-left text-xs text-text-muted"
      >
        {more ? "▾" : "▸"} {t("export.more")}
      </button>
      {more && (
        <div className="flex flex-col gap-1 text-sm">
          <label htmlFor="export-naming">{t("export.naming")}</label>
          <input
            id="export-naming"
            value={options.naming ?? "{sprite}_{animation}_{frame}"}
            onChange={(event) =>
              onOptions({ ...options, naming: event.target.value })
            }
            className="rounded-md border border-border bg-surface-2 px-2 py-1.5 font-mono text-xs"
          />
          <span className="text-xs text-text-muted">
            {t("export.namingHint")}
          </span>
        </div>
      )}
    </div>
  );
}
