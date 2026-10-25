import { useState } from "react";
import {
  AnimationDialog,
  coreFramesApi,
  type FramesApi,
  type Tag as Animation,
} from "@/features/frames-animations";
import { Button, Menu, type MenuItem } from "@/shared/components";
import type { SpriteSummary, Tag } from "@/shared/lib/core";
import { useLanguage, useT } from "@/shared/lib/i18n";
import type { FrameCache } from "@/shared/lib/render";
import { SpriteThumb } from "./SpriteThumb";

type Props = {
  sprite: SpriteSummary | null;
  cache: FrameCache;
  revision: number;
  onOpen: (sprite: SpriteSummary) => void;
  onRename: (sprite: SpriteSummary) => void;
  onDuplicate: (sprite: SpriteSummary) => void;
  onDelete: (sprite: SpriteSummary) => void;
  // The folders the sprite can move to.
  moveItems: MenuItem[];
  // The calls to make, edit and delete animations. The tests give a fake one.
  framesApi?: FramesApi;
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
export function DetailsPanel({
  sprite,
  cache,
  revision,
  onOpen,
  onRename,
  onDuplicate,
  onDelete,
  moveItems,
  framesApi = coreFramesApi,
}: Props) {
  const t = useT();
  const [moving, setMoving] = useState(false);
  // The dialog of an animation: a new one, or the one that is edited.
  const [editing, setEditing] = useState<{ tag?: Animation } | null>(null);
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

  const target = { folder: sprite.folder, name: sprite.name };
  // The project refreshes by itself with the change event, and Undo (Ctrl+Z)
  // brings an animation back.
  const remove = (name: string) =>
    framesApi.removeTag(target, name).catch(() => {});

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
      <h2
        className="truncate font-title text-xl font-bold"
        onDoubleClick={() => onRename(sprite)}
      >
        {sprite.name}
      </h2>
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
        <div className="mb-1 flex items-center justify-between">
          <h3 className="text-xs font-semibold text-text-muted">
            {t("browse.animations")}
          </h3>
          <button
            type="button"
            aria-label={t("anim.newTitle")}
            title={t("anim.newTitle")}
            onClick={() => setEditing({})}
            className="flex h-5 w-5 items-center justify-center rounded border border-border text-sm"
          >
            +
          </button>
        </div>
        {sprite.tags.length === 0 ? (
          <p className="text-sm text-text-faint">{t("browse.noAnimations")}</p>
        ) : (
          <ul className="flex flex-col gap-2">
            {sprite.tags.map((tag) => (
              <li
                key={tag.name}
                className="flex items-start justify-between gap-2"
              >
                <div>
                  <p className="text-sm font-bold">{tag.name}</p>
                  <p className="font-mono text-xs text-text-muted">
                    {animation(tag)}
                  </p>
                </div>
                <div className="flex gap-1 text-xs">
                  <button
                    type="button"
                    aria-label={t("anim.editNamed", { name: tag.name })}
                    onClick={() => setEditing({ tag: tag as Animation })}
                    className="rounded border border-border px-1.5 py-0.5"
                  >
                    {t("anim.edit")}
                  </button>
                  <button
                    type="button"
                    aria-label={t("anim.deleteNamed", { name: tag.name })}
                    onClick={() => void remove(tag.name)}
                    className="rounded border border-error px-1.5 py-0.5 text-error"
                  >
                    {t("anim.delete")}
                  </button>
                </div>
              </li>
            ))}
          </ul>
        )}
      </section>

      {editedBy && <p className="text-xs text-text-muted">{editedBy}</p>}

      <Button
        variant="primary"
        className="justify-center"
        onClick={() => onOpen(sprite)}
      >
        {t("command.sprite.open")}
      </Button>
      <div className="flex gap-2">
        <Button
          className="flex-1 justify-center"
          onClick={() => onDuplicate(sprite)}
        >
          {t("command.sprite.duplicate")}
        </Button>
        <Button
          variant="danger"
          className="flex-1 justify-center"
          onClick={() => onDelete(sprite)}
        >
          {t("command.sprite.delete")}
        </Button>
      </div>
      <div className="flex gap-2">
        <Button
          className="flex-1 justify-center"
          onClick={() => onRename(sprite)}
        >
          {t("command.sprite.rename")}
        </Button>
        <div className="relative flex-1">
          <Button
            className="w-full justify-center"
            aria-haspopup="menu"
            onClick={() => setMoving(!moving)}
          >
            {t("command.sprite.move")}
          </Button>
          {moving && (
            <Menu items={moveItems} onClose={() => setMoving(false)} />
          )}
        </div>
      </div>
      {editing && (
        <AnimationDialog
          frameCount={sprite.frames}
          tag={editing.tag}
          others={sprite.tags
            .map((tag) => tag.name)
            .filter((name) => name !== editing.tag?.name)}
          onSave={(tag) =>
            editing.tag
              ? framesApi.setTag(target, editing.tag.name, tag)
              : framesApi.addTag(target, tag)
          }
          onClose={() => setEditing(null)}
        />
      )}
    </div>
  );
}
