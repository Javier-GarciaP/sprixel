import { useState } from "react";
import { Button, Field } from "@/shared/components";
import { useT } from "@/shared/lib/i18n";
import type { Anchor, AnchorKey } from "../api/types";
import { keyOf, sameKey } from "../tools/anchors";

// What the panel edits: an anchor that exists, or a new one at a place.
export type AnchorTarget =
  { kind: "edit"; key: AnchorKey } | { kind: "new"; x: number; y: number };

type Props = {
  // The anchors of this frame.
  anchors: Anchor[];
  frame: number;
  width: number;
  height: number;
  target: AnchorTarget | null;
  onTarget: (target: AnchorTarget | null) => void;
  // `replaces` is the anchor that is renamed or moved, if any.
  onSave: (anchor: Anchor, replaces: AnchorKey | null) => void;
  onDelete: (key: AnchorKey) => void;
  // A failure of the core, shown in the panel.
  error: string | null;
};

// The panel of the Anchor tool: the anchors of the frame, and a form to
// create, rename, move (also with the number boxes, for the keyboard) and
// delete one. Dragging on the canvas moves an anchor too.
export function AnchorPanel(props: Props) {
  const t = useT();
  const { anchors, target, onTarget } = props;
  const current =
    target?.kind === "edit"
      ? anchors.find((a) => sameKey(keyOf(a), target.key))
      : undefined;
  return (
    <div
      role="dialog"
      aria-modal="false"
      aria-label={t("anchor.title")}
      className="absolute right-3 top-3 z-20 flex max-h-[calc(100%-1.5rem)] w-64 flex-col gap-3 overflow-y-auto rounded-lg border-2 border-border-strong bg-surface p-4 shadow-lg"
    >
      <h2 className="font-title text-base font-bold">{t("anchor.title")}</h2>
      <p className="text-xs text-text-muted">{t("anchor.hint")}</p>
      {anchors.length === 0 ? (
        <p className="text-xs">{t("anchor.empty")}</p>
      ) : (
        <ul className="flex flex-wrap gap-1">
          {anchors.map((anchor) => (
            <li key={`${anchor.name}-${anchor.frame}`}>
              <button
                type="button"
                aria-pressed={current === anchor}
                onClick={() => onTarget({ kind: "edit", key: keyOf(anchor) })}
                className={`rounded border px-1.5 py-0.5 text-xs ${current === anchor ? "border-amber bg-surface-2" : "border-border bg-bg"}`}
              >
                {anchor.name}
              </button>
            </li>
          ))}
        </ul>
      )}
      <Button
        onClick={() => onTarget({ kind: "new", x: 0, y: 0 })}
        className="self-start"
      >
        {t("anchor.new")}
      </Button>
      {(current || target?.kind === "new") && (
        <AnchorForm
          // A different anchor starts a fresh form.
          key={
            target?.kind === "new"
              ? `new-${target.x}-${target.y}`
              : `${current!.name}-${current!.x}-${current!.y}`
          }
          {...props}
          current={current}
        />
      )}
    </div>
  );
}

function AnchorForm({
  current,
  target,
  frame,
  width,
  height,
  onSave,
  onDelete,
  onTarget,
  error,
}: Props & { current: Anchor | undefined }) {
  const t = useT();
  const start =
    current ?? (target?.kind === "new" ? { x: target.x, y: target.y } : null);
  const [name, setName] = useState(current?.name ?? "");
  const [x, setX] = useState(String(start?.x ?? 0));
  const [y, setY] = useState(String(start?.y ?? 0));
  const [onlyFrame, setOnlyFrame] = useState(false);

  const cx = Number(x);
  const cy = Number(y);
  const validPlace =
    Number.isInteger(cx) &&
    Number.isInteger(cy) &&
    cx >= 0 &&
    cy >= 0 &&
    cx < width &&
    cy < height;
  const trimmed = name.trim();
  const validName = trimmed.length >= 1 && trimmed.length <= 64;

  const save = () => {
    if (!validName || !validPlace) return;
    const anchor: Anchor = {
      name: trimmed,
      x: cx,
      y: cy,
      frame: current ? current.frame : onlyFrame ? frame : null,
    };
    onSave(anchor, current ? keyOf(current) : null);
  };

  return (
    <div className="flex flex-col gap-2 border-t border-border pt-3">
      <Field
        label={t("anchor.name")}
        value={name}
        onChange={(event) => setName(event.target.value)}
        maxLength={64}
        error={name !== "" && !validName ? t("anchor.badName") : undefined}
      />
      <div className="flex gap-2">
        <Field
          label={t("anchor.x")}
          inputMode="numeric"
          value={x}
          onChange={(event) => setX(event.target.value)}
        />
        <Field
          label={t("anchor.y")}
          inputMode="numeric"
          value={y}
          onChange={(event) => setY(event.target.value)}
        />
      </div>
      {current ? (
        <p className="text-xs text-text-muted">
          {current.frame === null
            ? t("anchor.scopeSprite")
            : t("anchor.scopeFrame", { n: current.frame })}
        </p>
      ) : (
        <label className="flex items-center gap-2 text-sm">
          <input
            type="checkbox"
            checked={onlyFrame}
            onChange={(event) => setOnlyFrame(event.target.checked)}
          />
          {t("anchor.onlyFrame")}
        </label>
      )}
      {error && (
        <p role="alert" className="text-xs text-error">
          {error}
        </p>
      )}
      <div className="flex gap-2">
        <Button
          variant="primary"
          disabled={!validName || !validPlace}
          onClick={save}
        >
          {t("anchor.save")}
        </Button>
        {current && (
          <Button
            variant="danger"
            onClick={() => {
              onDelete(keyOf(current));
              onTarget(null);
            }}
          >
            {t("anchor.delete")}
          </Button>
        )}
      </div>
    </div>
  );
}
