import { useEffect, useId, useState } from "react";
import { Button, Dialog, Icon } from "@/shared/components";
import { useT, type TranslationKey } from "@/shared/lib/i18n";
import { asPaletteFailure } from "./api";
import {
  MAX_COLORS,
  changedKeys,
  draftOf,
  isHex,
  keyError,
  nameError,
  newColor,
  toEdits,
  toHex,
  toReplacements,
  toRgb,
  type DraftColor,
  type Removed,
} from "./draft";
import { RampTool } from "./RampTool";
import { UnusedTool } from "./UnusedTool";
import type { PaletteApi } from "./types";

type Props = {
  api: PaletteApi;
  // The palette to edit.
  name: string;
  // True while an agent writes to the sprite that is open. Changing a key (or
  // removing a used color) rewrites sprites, so it has to wait.
  agentBusy?: boolean;
  onClose: () => void;
  // Called after Done applied the changes, so the screen reads them again.
  onApplied: () => void;
};

// The messages of the errors of the core.
const failures: Record<string, TranslationKey> = {
  reserved_key: "palette.errorReserved",
  bad_key: "palette.errorBadKey",
  duplicate_key: "palette.errorDuplicateKey",
  duplicate_name: "palette.errorDuplicateName",
  bad_name: "palette.errorEmptyName",
  bad_color: "palette.errorBadHex",
  palette_full: "palette.errorFull",
  replacement_needed: "palette.errorReplacement",
  agent_writing: "palette.agentBusy",
  bad_replacement: "palette.errorReplacement",
};

// A text field that keeps what the user types until it is valid. The color
// keeps its last valid value, so an invalid text is never applied.
function DraftField({
  label,
  value,
  error,
  onValid,
  maxLength,
  mono,
}: {
  label: string;
  value: string;
  // The message for this text, or null when it is valid.
  error: (text: string) => string | null;
  onValid: (text: string) => void;
  maxLength?: number;
  mono?: boolean;
}) {
  const id = useId();
  const [text, setText] = useState(value);
  const message = error(text);
  // The color changed elsewhere (a slider, another color): start again.
  useEffect(() => setText(value), [value]);
  return (
    <div className="relative flex flex-col gap-1 text-xs text-text-muted">
      <label htmlFor={id}>{label}</label>
      <input
        id={id}
        value={text}
        maxLength={maxLength}
        spellCheck={false}
        aria-invalid={message !== null}
        onChange={(event) => {
          setText(event.target.value);
          if (error(event.target.value) === null) onValid(event.target.value);
        }}
        className={`rounded-md border bg-bg px-3 py-2 text-sm text-text ${mono ? "font-mono" : ""} ${message ? "border-error" : "border-border"}`}
      />
      {message && (
        <span
          role="alert"
          className="absolute right-0 top-full z-10 mt-1 w-max max-w-[14rem] rounded-md border border-error bg-surface px-2 py-1 text-error shadow-lg"
        >
          {message}
        </span>
      )}
    </div>
  );
}

// The dialog to edit a palette. It works on a copy: nothing reaches the
// project until Done, and then it all goes in as one undoable action.
export function PaletteEditorDialog({
  api,
  name,
  agentBusy,
  onClose,
  onApplied,
}: Props) {
  const t = useT();
  const [draft, setDraft] = useState<DraftColor[] | null>(null);
  const [selected, setSelected] = useState(0);
  const [removed, setRemoved] = useState<Removed[]>([]);
  // The color being removed that sprites use, while the user picks what its
  // pixels become.
  const [replacing, setReplacing] = useState<DraftColor | null>(null);
  const [failure, setFailure] = useState<string | null>(null);
  const [loadFailed, setLoadFailed] = useState(false);
  const [busy, setBusy] = useState(false);

  // The draft as it was loaded: the tools that write at once (the ramp) only
  // work when nothing was edited, or the edits would be lost.
  const [loaded, setLoaded] = useState<DraftColor[]>([]);
  const load = () =>
    api
      .loadPalette(name)
      .then((info) => {
        const next = draftOf(info.colors);
        setDraft(next);
        setLoaded(next);
        setRemoved([]);
      })
      .catch(() => setLoadFailed(true));
  useEffect(() => {
    void load();
  }, [api, name]);

  const title = t("palette.title", { name });
  if (loadFailed) {
    return (
      <Dialog title={title} closeLabel={t("dialog.close")} onClose={onClose}>
        <p role="alert" className="text-sm text-error">
          {t("palette.loadError")}
        </p>
      </Dialog>
    );
  }
  if (!draft) return null;

  const current = draft.find((c) => c.id === selected) ?? draft[0];
  const change = (id: number, patch: Partial<DraftColor>) =>
    setDraft(draft.map((c) => (c.id === id ? { ...c, ...patch } : c)));

  const add = () => {
    const color = newColor(draft);
    if (!color) return;
    setDraft([...draft, color]);
    setSelected(color.id);
  };

  const drop = (color: DraftColor, replacement: string | null) => {
    const rest = draft.filter((c) => c.id !== color.id);
    setDraft(rest);
    if (color.from !== null) {
      setRemoved([...removed, { key: color.from, with: replacement }]);
    }
    setSelected(rest[0]?.id ?? 0);
    setReplacing(null);
  };

  const remove = (color: DraftColor) => {
    if (agentBusy && color.from !== null && color.used) {
      setFailure(t("palette.agentBusy"));
      return;
    }
    // A color that sprites use needs a replacement for its pixels.
    if (color.from !== null && color.used) setReplacing(color);
    else drop(color, null);
  };

  const done = async () => {
    setBusy(true);
    setFailure(null);
    try {
      await api.applyEdit(name, toEdits(draft), toReplacements(removed));
      onApplied();
      onClose();
    } catch (error) {
      const { code, message } = asPaletteFailure(error);
      setFailure(failures[code] ? t(failures[code]) : (message ?? code));
      setBusy(false);
    }
  };

  const keys = changedKeys(draft);
  const dirty =
    JSON.stringify(draft) !== JSON.stringify(loaded) || removed.length > 0;
  const rgb = current ? toRgb(current.hex) : [0, 0, 0];
  // The colors that stay (and already existed) can take the pixels of a
  // removed color.
  const candidates = draft.filter(
    (c) => c.from !== null && c.id !== replacing?.id,
  );

  return (
    <Dialog title={title} closeLabel={t("dialog.close")} onClose={onClose} wide>
      <div className="flex gap-5">
        <div className="flex w-72 shrink-0 flex-col gap-2">
          <ul
            aria-label={t("palette.colors")}
            className="flex max-h-[min(26rem,50vh)] flex-col gap-1 overflow-y-auto"
          >
            {draft.map((color) => (
              <li key={color.id}>
                <button
                  type="button"
                  aria-pressed={color.id === current?.id}
                  aria-label={`${color.name} ${color.key}`}
                  onClick={() => setSelected(color.id)}
                  className={`flex w-full items-center gap-2 rounded-md border px-2 py-1.5 text-left text-sm ${color.id === current?.id ? "border-amber" : "border-transparent hover:bg-surface-2"}`}
                >
                  <span
                    className="h-5 w-5 shrink-0 rounded border border-border-strong"
                    style={{ background: color.hex }}
                  />
                  <span className="w-6 rounded border border-border text-center font-mono text-xs">
                    {color.key}
                  </span>
                  <span className="flex-1 truncate">{color.name}</span>
                  <span className="font-mono text-xs text-text-muted">
                    {color.hex}
                  </span>
                </button>
              </li>
            ))}
          </ul>
          <Button onClick={add} disabled={draft.length >= MAX_COLORS}>
            <Icon name="plus" className="h-4 w-4" />
            {t("palette.addColor")}
          </Button>
        </div>
        <div className="flex min-w-0 flex-1 flex-col gap-3">
          {current && (
            <>
              <p className="text-xs font-semibold text-text-muted">
                {t("palette.selected")}
              </p>
              <div
                data-testid="selected-color"
                className="h-16 rounded-md border border-border-strong"
                style={{ background: current.hex }}
              />
              <div className="flex gap-3">
                <div className="flex-1">
                  <DraftField
                    key={`name-${current.id}`}
                    label={t("palette.name")}
                    value={current.name}
                    error={(text) => {
                      const code = nameError(draft, current.id, text);
                      return code === "empty"
                        ? t("palette.errorEmptyName")
                        : code === "duplicate"
                          ? t("palette.errorDuplicateName")
                          : null;
                    }}
                    onValid={(text) => change(current.id, { name: text })}
                  />
                </div>
                <div className="w-28">
                  <DraftField
                    key={`key-${current.id}`}
                    label={t("palette.key")}
                    value={current.key}
                    maxLength={1}
                    mono
                    error={(text) => {
                      const code = keyError(draft, current.id, text);
                      return code === "reserved"
                        ? t("palette.errorReserved")
                        : code === "duplicate"
                          ? t("palette.errorDuplicateKey")
                          : code
                            ? t("palette.errorBadKey")
                            : null;
                    }}
                    onValid={(text) => {
                      // A key that changes rewrites the sprites: it waits.
                      if (
                        agentBusy &&
                        current.from !== null &&
                        text !== current.from
                      ) {
                        setFailure(t("palette.agentBusy"));
                        return;
                      }
                      setFailure(null);
                      change(current.id, { key: text });
                    }}
                  />
                </div>
              </div>
              <DraftField
                key={`hex-${current.id}`}
                label={t("palette.hex")}
                value={current.hex}
                mono
                error={(text) =>
                  isHex(text) ? null : t("palette.errorBadHex")
                }
                onValid={(text) =>
                  change(current.id, { hex: text.toUpperCase() })
                }
              />
              {(["R", "G", "B"] as const).map((channel, i) => (
                <label
                  key={channel}
                  className="flex items-center gap-3 text-xs text-text-muted"
                >
                  <span className="w-3">{channel}</span>
                  <input
                    type="range"
                    min={0}
                    max={255}
                    aria-label={t(
                      channel === "R"
                        ? "palette.red"
                        : channel === "G"
                          ? "palette.green"
                          : "palette.blue",
                    )}
                    value={rgb[i]}
                    onChange={(event) => {
                      const next = [...rgb] as [number, number, number];
                      next[i] = Number(event.target.value);
                      change(current.id, { hex: toHex(next) });
                    }}
                    className="flex-1 accent-amber"
                  />
                  <span className="w-8 text-right font-mono">{rgb[i]}</span>
                </label>
              ))}
              {replacing?.id === current.id ? (
                <ReplacementPicker
                  color={replacing}
                  candidates={candidates}
                  onPick={(key) => drop(replacing, key)}
                  onCancel={() => setReplacing(null)}
                />
              ) : (
                <Button variant="danger" onClick={() => remove(current)}>
                  {t("palette.removeColor")}
                </Button>
              )}
            </>
          )}
          {keys.length > 0 && (
            <div
              role="alert"
              className="rounded-md border border-amber px-3 py-2 text-xs text-text"
            >
              {keys.map((key) => (
                <p key={key}>{t("palette.keyWarning", { key })}</p>
              ))}
            </div>
          )}
          <RampTool
            api={api}
            name={name}
            startHex={current?.hex ?? "#808080"}
            disabled={dirty}
            onAdded={() => {
              void load();
              onApplied();
            }}
          />
          <UnusedTool
            api={api}
            name={name}
            unused={loaded.filter((c) => c.from !== null && !c.used)}
            disabled={dirty}
            onRemoved={() => {
              void load();
              onApplied();
            }}
          />
          <p className="text-xs text-text-muted">{t("palette.appliesToAll")}</p>
        </div>
      </div>
      {failure && (
        <p role="alert" className="mt-3 text-sm text-error">
          {failure}
        </p>
      )}
      <div className="mt-4 flex justify-end gap-2">
        <Button onClick={onClose}>{t("dialog.cancel")}</Button>
        <Button variant="primary" onClick={done} disabled={busy}>
          {t("palette.done")}
        </Button>
      </div>
    </Dialog>
  );
}

// Asks what the pixels of a removed color become.
function ReplacementPicker({
  color,
  candidates,
  onPick,
  onCancel,
}: {
  color: DraftColor;
  candidates: DraftColor[];
  onPick: (key: string) => void;
  onCancel: () => void;
}) {
  const t = useT();
  const [choice, setChoice] = useState(".");
  return (
    <div className="flex flex-col gap-2 rounded-md border border-border p-3">
      <p className="text-xs">
        {t("palette.replaceTitle", { name: color.name })}
      </p>
      <select
        aria-label={t("palette.replaceWith")}
        value={choice}
        onChange={(event) => setChoice(event.target.value)}
        className="rounded-md border border-border bg-bg px-2 py-1.5 text-sm"
      >
        <option value=".">{t("editor.transparent")}</option>
        {candidates.map((c) => (
          <option key={c.id} value={c.from as string}>
            {c.name} ({c.from})
          </option>
        ))}
      </select>
      <div className="flex gap-2">
        <Button variant="danger" onClick={() => onPick(choice)}>
          {t("palette.replaceConfirm")}
        </Button>
        <Button onClick={onCancel}>{t("dialog.cancel")}</Button>
      </div>
    </div>
  );
}
