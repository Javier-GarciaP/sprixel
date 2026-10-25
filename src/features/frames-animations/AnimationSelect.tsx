import { useT } from "@/shared/lib/i18n";
import type { Tag } from "./types";

type Props = {
  tags: Tag[];
  // The name of the chosen animation, or null for "All".
  value: string | null;
  onChange: (animation: string | null) => void;
};

// The list above the frames: "All" and the animations of the sprite.
export function AnimationSelect({ tags, value, onChange }: Props) {
  const t = useT();
  return (
    <select
      aria-label={t("anim.select")}
      value={tags.some((tag) => tag.name === value) ? (value ?? "") : ""}
      onChange={(event) => onChange(event.target.value || null)}
      className="w-24 rounded-md border border-border bg-surface px-1.5 py-1 text-xs text-text"
    >
      <option value="">{t("anim.all")}</option>
      {tags.map((tag) => (
        <option key={tag.name} value={tag.name}>
          {tag.name}
        </option>
      ))}
    </select>
  );
}
