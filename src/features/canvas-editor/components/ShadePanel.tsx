import { useEffect, useRef, useState } from "react";
import { useT, type TranslationKey } from "@/shared/lib/i18n";
import type {
  Applied,
  Light,
  PowerApi,
  Preview,
  ShadeArgs,
  SpriteRef,
} from "../api/types";
import { powerErrorText } from "../powerError";
import { darkToLight } from "../tools/ramp";
import type { PaletteColor, Rect } from "../types";
import { KeyPicker } from "./KeyPicker";
import { ToolPanel } from "./ToolPanel";

const lights: Light[] = [
  "top-left",
  "top",
  "top-right",
  "left",
  "right",
  "bottom-left",
  "bottom",
  "bottom-right",
];

type Props = {
  api: PowerApi;
  sprite: SpriteRef;
  frame: number;
  palette: PaletteColor[];
  // The selected rectangle, if any: the panel can limit the shading to it.
  selection: Rect | null;
  onPreview: (pixels: Preview["pixels"] | null) => void;
  onApplied: (result: Applied) => void;
  onClose: () => void;
};

// The Shade panel: the options of `auto_shade`, a live preview on the canvas
// and Apply. The preview is read from the core and writes nothing.
export function ShadePanel({
  api,
  sprite,
  frame,
  palette,
  selection,
  onPreview,
  onApplied,
  onClose,
}: Props) {
  const t = useT();
  const [light, setLight] = useState<Light>("top-left");
  const [base, setBase] = useState<string | null>(null);
  const [ramp, setRamp] = useState<string[]>([]);
  const [inSelection, setInSelection] = useState(false);
  const [counts, setCounts] = useState<number | null>(null);
  const [error, setError] = useState<string | null>(null);

  const valid = base !== null && ramp.length >= 2 && ramp.includes(base);
  const args: ShadeArgs | null = valid
    ? {
        light,
        base,
        keys: darkToLight(palette, ramp),
        region: inSelection ? selection : null,
      }
    : null;
  const argsKey = JSON.stringify(args);

  const latest = useRef(0);
  const show = useRef(onPreview);
  show.current = onPreview;
  useEffect(() => {
    const mine = ++latest.current;
    if (!args) {
      setCounts(null);
      setError(null);
      show.current(null);
      return;
    }
    api
      .shadePreview(sprite, frame, args)
      .then((preview) => {
        if (mine !== latest.current) return;
        setError(null);
        setCounts(preview.pixels.length);
        show.current(preview.pixels);
      })
      .catch((failure) => {
        if (mine !== latest.current) return;
        setCounts(null);
        setError(powerErrorText(t, failure));
        show.current(null);
      });
    // `argsKey` stands for `args`, which is a new object on every render.
  }, [api, sprite, frame, argsKey]);
  useEffect(() => () => show.current(null), []);

  // The base color is always part of the ramp.
  const chooseBase = (key: string) => {
    setBase(key);
    setRamp((now) => (now.includes(key) ? now : [...now, key]));
  };
  const toggle = (key: string) =>
    setRamp((now) =>
      now.includes(key)
        ? now.filter((k) => k !== key)
        : now.length < 8
          ? [...now, key]
          : now,
    );

  const apply = () => {
    if (!args) return;
    api
      .shadeApply(sprite, frame, args)
      .then(onApplied)
      .catch((failure) => setError(powerErrorText(t, failure)));
  };

  return (
    <ToolPanel
      title={t("shade.title")}
      onClose={onClose}
      onApply={apply}
      canApply={args !== null && (counts ?? 0) > 0}
    >
      <label className="flex flex-col gap-1 text-xs text-text-muted">
        {t("shade.light")}
        <select
          value={light}
          onChange={(event) => setLight(event.target.value as Light)}
          className="rounded-md border border-border bg-bg px-2 py-1 text-sm text-text"
        >
          {lights.map((name) => (
            <option key={name} value={name}>
              {t(`light.${name}` as TranslationKey)}
            </option>
          ))}
        </select>
      </label>
      <KeyPicker
        label={t("shade.base")}
        palette={palette}
        chosen={base ? [base] : []}
        onToggle={chooseBase}
      />
      <KeyPicker
        label={t("shade.ramp")}
        palette={palette}
        chosen={ramp}
        onToggle={toggle}
      />
      {selection && (
        <label className="flex items-center gap-2 text-sm">
          <input
            type="checkbox"
            checked={inSelection}
            onChange={(event) => setInSelection(event.target.checked)}
          />
          {t("shade.region")}
        </label>
      )}
      {error ? (
        <p role="alert" className="text-xs text-error">
          {error}
        </p>
      ) : !args ? (
        <p className="text-xs">{t("shade.needRamp")}</p>
      ) : (
        counts !== null && (
          <p className="text-xs">
            {counts === 0
              ? t("shade.none")
              : t("shade.count", { count: counts })}
          </p>
        )
      )}
    </ToolPanel>
  );
}
