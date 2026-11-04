import { useEffect, useRef, useState } from "react";
import { useT } from "@/shared/lib/i18n";
import type {
  Applied,
  OutlineArgs,
  PowerApi,
  Preview,
  SpriteRef,
} from "../api/types";
import { powerErrorText } from "../powerError";
import { darkestKey } from "../tools/ramp";
import type { PaletteColor } from "../types";
import { KeyPicker } from "./KeyPicker";
import { ToolPanel } from "./ToolPanel";

type Props = {
  api: PowerApi;
  sprite: SpriteRef;
  frame: number;
  palette: PaletteColor[];
  // The pixels the canvas shows as a preview; null takes the preview away.
  onPreview: (pixels: Preview["pixels"] | null) => void;
  onApplied: (result: Applied) => void;
  onClose: () => void;
};

// The Outline panel: the options of `auto_outline`, a live preview on the
// canvas and Apply. The preview is read from the core and writes nothing.
export function OutlinePanel({
  api,
  sprite,
  frame,
  palette,
  onPreview,
  onApplied,
  onClose,
}: Props) {
  const t = useT();
  const [key, setKey] = useState(() => darkestKey(palette));
  const [connectivity, setConnectivity] = useState<4 | 8>(8);
  const [outsideOnly, setOutsideOnly] = useState(false);
  const [inner, setInner] = useState(false);
  const [count, setCount] = useState<number | null>(null);
  const [error, setError] = useState<string | null>(null);
  const args: OutlineArgs | null = key
    ? { key, connectivity, outsideOnly, inner }
    : null;

  // Every change of an option asks for a new preview. An answer that comes
  // after a newer question is ignored.
  const latest = useRef(0);
  const show = useRef(onPreview);
  show.current = onPreview;
  useEffect(() => {
    if (!args) return;
    const mine = ++latest.current;
    api
      .outlinePreview(sprite, frame, args)
      .then((preview) => {
        if (mine !== latest.current) return;
        setError(null);
        setCount(preview.pixels.length);
        show.current(preview.pixels);
      })
      .catch((failure) => {
        if (mine !== latest.current) return;
        setCount(null);
        setError(powerErrorText(t, failure));
        show.current(null);
      });
    // `args` is rebuilt every render; its parts are the real inputs.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [api, sprite, frame, key, connectivity, outsideOnly, inner]);
  // Closing the panel takes the preview away.
  useEffect(() => () => show.current(null), []);

  const apply = () => {
    if (!args) return;
    api
      .outlineApply(sprite, frame, args)
      .then(onApplied)
      .catch((failure) => setError(powerErrorText(t, failure)));
  };

  return (
    <ToolPanel
      title={t("outline.title")}
      onClose={onClose}
      onApply={apply}
      canApply={args !== null && (count ?? 0) > 0}
    >
      <KeyPicker
        label={t("outline.key")}
        palette={palette}
        chosen={key ? [key] : []}
        onToggle={setKey}
      />
      <div
        role="group"
        aria-label={t("outline.neighbors")}
        className="flex rounded-md border border-border bg-bg p-0.5 text-xs"
      >
        {([4, 8] as const).map((value) => (
          <button
            key={value}
            type="button"
            aria-pressed={connectivity === value}
            onClick={() => setConnectivity(value)}
            className={`flex-1 rounded px-2 py-1 ${connectivity === value ? "bg-surface-2 font-semibold text-text" : "text-text-muted"}`}
          >
            {t(value === 4 ? "outline.n4" : "outline.n8")}
          </button>
        ))}
      </div>
      <label className="flex items-center gap-2 text-sm">
        <input
          type="checkbox"
          checked={outsideOnly}
          onChange={(event) => setOutsideOnly(event.target.checked)}
        />
        {t("outline.outsideOnly")}
      </label>
      <label className="flex items-center gap-2 text-sm">
        <input
          type="checkbox"
          checked={inner}
          onChange={(event) => setInner(event.target.checked)}
        />
        {t("outline.inner")}
      </label>
      {error ? (
        <p role="alert" className="text-xs text-error">
          {error}
        </p>
      ) : (
        count !== null && (
          <p className="text-xs">
            {count === 0 ? t("outline.none") : t("outline.count", { count })}
          </p>
        )
      )}
    </ToolPanel>
  );
}
