import { useT, type TranslationKey } from "@/shared/lib/i18n";
import type { RepeatMode } from "./types";

const modes: { id: RepeatMode; labelKey: TranslationKey }[] = [
  { id: "loop", labelKey: "anim.modeLoop" },
  { id: "once", labelKey: "anim.modeOnce" },
  { id: "pingpong", labelKey: "anim.modePingpong" },
];

// The three choices for what happens at the end of an animation.
export function ModePicker({
  value,
  onChange,
}: {
  value: RepeatMode;
  onChange: (mode: RepeatMode) => void;
}) {
  const t = useT();
  return (
    <fieldset className="flex flex-col gap-1">
      <legend className="mb-1 text-xs text-text-muted">{t("anim.mode")}</legend>
      <div className="flex gap-2">
        {modes.map(({ id, labelKey }) => (
          <button
            key={id}
            type="button"
            role="radio"
            aria-checked={value === id}
            onClick={() => onChange(id)}
            className={`rounded-md border px-3 py-1.5 text-sm ${value === id ? "border-amber bg-surface-2 font-semibold" : "border-border"}`}
          >
            {t(labelKey)}
          </button>
        ))}
      </div>
    </fieldset>
  );
}
