import { useRef, useState } from "react";
import { Menu } from "@/shared/components";
import { useT } from "@/shared/lib/i18n";
import { AnimationSelect } from "./AnimationSelect";
import { AnimationHeader } from "./AnimationHeader";
import { chosenTag } from "./animation";
import { DurationField } from "./DurationField";
import { frameDurationMs, DEFAULT_FRAME_MS } from "./duration";
import { FrameThumb } from "./FrameThumb";
import { NewAnimationDialog } from "./NewAnimationDialog";
import { RenameDialog } from "./RenameDialog";
import { groupsOf } from "./groups";
import type { FramesApi, SpriteRef, Tag } from "./types";
import { useSpriteChanges } from "./useSpriteChanges";
import { copyName } from "./validate";

type Props = {
  api: FramesApi;
  sprite: SpriteRef;
  frameCount: number;
  frame: number;
  onFrame: (frame: number) => void;
  // After a change: the editor reads the sprite again and, with a number,
  // goes to that frame.
  onChanged: (select?: number) => void;
  // The frame an AI agent is writing to, if any.
  busyFrame?: number | null;
  // The animations of the sprite and the chosen one (null is "All"). The
  // chosen one is what the preview plays.
  tags: Tag[];
  animation: string | null;
  onAnimation: (animation: string | null) => void;
  // The play button of a group: choose that animation and play it. Without it
  // the button only chooses the animation.
  onPlay?: (animation: string) => void;
  // The own duration of each frame (the default means none of its own).
  durations?: number[];
};

// Above this many frames the groups start closed.
export const COLLAPSE_ABOVE = 40;

const button =
  "flex h-7 flex-1 items-center justify-center rounded border border-border bg-surface text-sm text-text hover:bg-surface-2";

// The frames of the sprite, one thumbnail each. "+" adds a frame right after
// the current one, blank or as a copy of it.
export function FramesColumn({
  api,
  sprite,
  frameCount,
  frame,
  onFrame,
  onChanged,
  busyFrame,
  tags,
  animation,
  onAnimation,
  onPlay,
  durations = [],
}: Props) {
  const t = useT();
  const revision = useSpriteChanges(api, sprite);
  const { groups, overlaps } = groupsOf(tags, frameCount);
  // Groups the person closed (true) or opened (false). Without a choice, all
  // groups are open, except in a long sprite (more than COLLAPSE_ABOVE
  // frames), where all start closed so the strip stays short.
  const [closed, setClosed] = useState<Record<string, boolean>>({});

  const [menu, setMenu] = useState<{
    frame: number;
    x: number;
    y: number;
  } | null>(null);
  const dragged = useRef<number | null>(null);
  // The menu of an animation header, and the dialog that is open.
  const [groupMenu, setGroupMenu] = useState<{
    name: string;
    x: number;
    y: number;
  } | null>(null);
  const [dialog, setDialog] = useState<
    { kind: "new" } | { kind: "rename"; name: string } | null
  >(null);
  const spriteKey = JSON.stringify([...sprite.folder, sprite.name]);
  const names = tags.map((tag) => tag.name);
  const menuTag = tags.find((tag) => tag.name === groupMenu?.name);
  const menuSize = menuTag ? menuTag.to - menuTag.from + 1 : 0;

  // A change that fails (the core knows better) just reads the sprite again.
  const run = (change: Promise<unknown>, select?: number) =>
    change.then(() => onChanged(select)).catch(() => onChanged());

  const remove = (n: number) =>
    // After the delete the current frame keeps its picture when it can.
    run(
      api.deleteFrame(sprite, n),
      n < frame ? frame - 1 : Math.min(frame, frameCount - 1),
    );

  const add = (copyOf: number | null) =>
    api
      .insertFrame(sprite, frame + 1, copyOf)
      .then((created) => onChanged(created))
      .catch(() => onChanged());

  const renderFrame = (n: number) => (
    <div
      key={n}
      className="relative"
      draggable
      onDragStart={(event) => {
        dragged.current = n;
        event.dataTransfer?.setData("text/plain", String(n));
      }}
      onDragOver={(event) => event.preventDefault()}
      onDrop={(event) => {
        event.preventDefault();
        const from = dragged.current;
        dragged.current = null;
        if (from !== null && from !== n) {
          void run(api.moveFrame(sprite, from, n), n);
        }
      }}
      onContextMenu={(event) => {
        event.preventDefault();
        setMenu({ frame: n, x: event.clientX, y: event.clientY });
      }}
    >
      <button
        type="button"
        aria-label={t("editor.frame", { n })}
        aria-current={n === frame ? "true" : undefined}
        aria-busy={n === busyFrame ? true : undefined}
        title={n === busyFrame ? t("marker.frameBusy") : undefined}
        onClick={() => onFrame(n)}
        className={`relative rounded-md border p-1 ${n === frame ? "border-amber bg-surface-2" : "border-border bg-surface"}`}
      >
        <FrameThumb
          api={api}
          sprite={sprite}
          frame={n}
          revision={revision}
          box={64}
        />
        <span className="absolute bottom-1.5 left-2 font-mono text-[10px] text-white">
          {n}
        </span>
        {n === busyFrame && (
          <span
            aria-hidden
            className="absolute top-1.5 right-2 h-2 w-2 rounded-full bg-amber"
          />
        )}
      </button>
      <div className="mt-0.5 flex justify-center">
        <DurationField
          frame={n}
          own={
            (durations[n - 1] ?? DEFAULT_FRAME_MS) === DEFAULT_FRAME_MS
              ? null
              : durations[n - 1]
          }
          fallback={frameDurationMs(
            DEFAULT_FRAME_MS,
            (
              chosenTag(tags, animation) ??
              tags.find((tag) => n >= tag.from && n <= tag.to)
            )?.fps ?? 10,
          )}
          onSave={(ms) => void run(api.setDuration(sprite, n, ms))}
        />
      </div>
      <button
        type="button"
        aria-label={t("frames.menu", { n })}
        onClick={(event) => {
          const box = event.currentTarget.getBoundingClientRect();
          setMenu({ frame: n, x: box.left, y: box.bottom });
        }}
        className="absolute top-1.5 right-1.5 flex h-4 w-4 items-center justify-center rounded bg-black/50 text-[10px] leading-none text-white"
      >
        ⋯
      </button>
    </div>
  );

  return (
    <aside
      aria-label={t("editor.frames")}
      className="flex w-36 shrink-0 flex-col items-center gap-2 overflow-y-auto border-r border-border bg-bg py-3"
    >
      <h2 className="text-xs font-semibold text-text-muted">
        {t("editor.frames")}
      </h2>
      <AnimationSelect tags={tags} value={animation} onChange={onAnimation} />
      {overlaps && (
        <p role="status" className="w-24 text-[10px] text-text-faint">
          {t("frames.overlap")}
        </p>
      )}
      {groups.map((group) => {
        const key = `${spriteKey}|${group.id}`;
        const open = !(closed[key] ?? frameCount > COLLAPSE_ABOVE);
        return (
          <section
            key={group.id}
            aria-label={
              group.tag?.name ??
              t(`frames.group${group.kind === "base" ? "Base" : "None"}`)
            }
            className="flex w-full flex-col items-center gap-2 px-1"
          >
            <AnimationHeader
              group={group}
              open={open}
              chosen={group.tag?.name === animation}
              onToggle={() => setClosed({ ...closed, [key]: open })}
              onPlay={() =>
                group.tag && (onPlay ?? onAnimation)(group.tag.name)
              }
              onMenu={(box) =>
                group.tag &&
                setGroupMenu({
                  name: group.tag.name,
                  x: box.left,
                  y: box.bottom,
                })
              }
            />
            {open && group.frames.map(renderFrame)}
          </section>
        );
      })}
      {menu && (
        <div className="fixed z-30" style={{ left: menu.x, top: menu.y }}>
          <Menu
            onClose={() => setMenu(null)}
            items={[
              {
                id: "duplicate",
                label: t("frames.duplicate"),
                onSelect: () =>
                  void run(
                    api.insertFrame(sprite, menu.frame + 1, menu.frame),
                    menu.frame + 1,
                  ),
              },
              {
                id: "delete",
                label: t("frames.delete"),
                // The last frame stays: a sprite always has one.
                disabled: frameCount <= 1,
                onSelect: () => void remove(menu.frame),
              },
            ]}
          />
        </div>
      )}
      {groupMenu && (
        <div
          className="fixed z-30"
          style={{ left: groupMenu.x, top: groupMenu.y }}
        >
          <Menu
            onClose={() => setGroupMenu(null)}
            items={[
              {
                id: "rename",
                label: t("anim.rename"),
                onSelect: () =>
                  setDialog({ kind: "rename", name: groupMenu.name }),
              },
              {
                id: "duplicate",
                label: t("anim.duplicate"),
                onSelect: () =>
                  void run(
                    api.animationDuplicate(
                      sprite,
                      groupMenu.name,
                      copyName(groupMenu.name, names),
                    ),
                  ),
              },
              {
                id: "delete-keep",
                label: t("anim.deleteKeep"),
                separator: true,
                onSelect: () =>
                  void run(api.animationDelete(sprite, groupMenu.name, false)),
              },
              {
                id: "delete-frames",
                label: t("anim.deleteFrames"),
                // The last frame stays: a sprite always has one.
                disabled: menuSize >= frameCount,
                onSelect: () =>
                  void run(api.animationDelete(sprite, groupMenu.name, true)),
              },
            ]}
          />
        </div>
      )}
      {dialog?.kind === "new" && (
        <NewAnimationDialog
          others={names}
          onClose={() => setDialog(null)}
          onSave={(animation) =>
            api
              .animationCreate(sprite, animation)
              // The first new frame is the one after the last frame.
              .then(() => onChanged(frameCount + 1))
          }
        />
      )}
      {dialog?.kind === "rename" && (
        <RenameDialog
          name={dialog.name}
          others={names.filter((n) => n !== dialog.name)}
          onClose={() => setDialog(null)}
          onSave={(newName) =>
            api
              .animationRename(sprite, dialog.name, newName)
              .then(() => onChanged())
          }
        />
      )}
      <button
        type="button"
        className="h-7 w-28 rounded border border-border bg-surface text-xs text-text hover:bg-surface-2"
        onClick={() => setDialog({ kind: "new" })}
      >
        {t("anim.newOpen")}
      </button>
      <div className="flex w-20 gap-1">
        <button
          type="button"
          className={button}
          aria-label={t("frames.add")}
          title={t("frames.add")}
          onClick={() => add(null)}
        >
          +
        </button>
        <button
          type="button"
          className={button}
          aria-label={t("frames.addCopy")}
          title={t("frames.addCopy")}
          onClick={() => add(frame)}
        >
          ⧉
        </button>
      </div>
    </aside>
  );
}
