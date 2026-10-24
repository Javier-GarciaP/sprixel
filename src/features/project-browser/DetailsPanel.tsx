import type { SpriteSummary, Tag } from "@/shared/lib/core";
import { useLanguage, useT } from "@/shared/lib/i18n";
import type { FrameCache } from "@/shared/lib/render";
import { SpriteThumb } from "./SpriteThumb";

type Props = {
  sprite: SpriteSummary | null;
  cache: FrameCache;
  revision: number;
};

function Row({ label, value }: { label: string; value: string }) {
  return (
    <div className="flex justify-between gap-4 py-0.5 text-sm">
      <dt className="text-text-muted">{label}</dt>
      <dd className="font-mono">{value}</dd>
    </div>
  );
}

// The right side of the Project tab: everything about the selected sprite.
export function DetailsPanel({ sprite, cache, revision }: Props) {
  const t = useT();
  const language = useLanguage();

  if (!sprite) {
    return (
      <p className="text-sm text-text-muted">{t("browse.details.empty")}</p>
    );
  }

  const animation = (tag: Tag) =>
    t("browse.animationInfo", {
      from: tag.from,
      to: tag.to,
      fps: tag.fps,
      mode: t(`browse.mode.${tag.mode}`),
    });

  const editor = sprite.lastEditor;
  const editedBy = editor
    ? t("browse.editedBy", {
        name: editor.author === "user" ? t("browse.you") : editor.author,
        time: new Date(editor.atMs).toLocaleTimeString(language, {
          hour: "2-digit",
          minute: "2-digit",
        }),
      })
    : null;

  return (
    <div className="flex flex-col gap-4">
      <h2 className="truncate font-title text-xl font-bold">{sprite.name}</h2>
      <SpriteThumb
        sprite={sprite}
        cache={cache}
        revision={revision}
        box={224}
      />

      <section
        aria-label={t("browse.properties")}
        className="rounded-lg border border-border bg-surface p-3"
      >
        <h3 className="mb-1 text-xs font-semibold text-text-muted">
          {t("browse.properties")}
        </h3>
        <dl>
          <Row
            label={t("browse.size")}
            value={`${sprite.width} × ${sprite.height} px`}
          />
          <Row label={t("browse.framesLabel")} value={String(sprite.frames)} />
          <Row label={t("browse.palette")} value={sprite.palette} />
        </dl>
      </section>

      <section
        aria-label={t("browse.animations")}
        className="rounded-lg border border-border bg-surface p-3"
      >
        <h3 className="mb-1 text-xs font-semibold text-text-muted">
          {t("browse.animations")}
        </h3>
        {sprite.tags.length === 0 ? (
          <p className="text-sm text-text-faint">{t("browse.noAnimations")}</p>
        ) : (
          <ul className="flex flex-col gap-2">
            {sprite.tags.map((tag) => (
              <li key={tag.name}>
                <p className="text-sm font-bold">{tag.name}</p>
                <p className="font-mono text-xs text-text-muted">
                  {animation(tag)}
                </p>
              </li>
            ))}
          </ul>
        )}
      </section>

      {editedBy && <p className="text-xs text-text-muted">{editedBy}</p>}
    </div>
  );
}
