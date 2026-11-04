import { useEffect, useRef, useState, type KeyboardEvent } from "react";
import { Menu } from "@/shared/components";
import { useT } from "@/shared/lib/i18n";
import { AnimationHeader } from "./AnimationHeader";
import { chosenTag } from "./animation";
import { DurationField } from "./DurationField";
import { frameDurationMs, DEFAULT_FRAME_MS } from "./duration";
import { FrameThumb } from "./FrameThumb";
import { IconButton } from "./IconButton";
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

// The size of a thumbnail: two of them fit side by side in the column.
const THUMB = 56;

// The icons of the two buttons at the bottom.
const PLUS = "M12 5v14M5 12h14";
const COPY = "M9 9h11v11H9zM5 15V4h11";

const spriteKeyOf = (sprite: SpriteRef) =>
  JSON.stringify([...sprite.folder, sprite.name]);

// The frames of the sprite in groups: Base, one for each animation and the
// frames of no animation. The groups are the selector of the animation: the
// chosen one is open and shows its frames in a grid, the others are one row.
// The buttons at the bottom add a frame right after the current one, blank or
// as a copy of it.
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
  // Base and "No animation" are not animations, so choosing them sets no
  // animation ("All"). This remembers which of the two the person chose; it
  // is forgotten when the current frame changes.
  const [plain, setPlain] = useState<string | null>(null);
  useEffect(() => setPlain(null), [frame, spriteKeyOf(sprite)]);
  // Only one group is open: the chosen animation, else the one the person
  // chose, else the group of the current frame.
  const chosen = chosenTag(tags, animation);
  const openId =
    groups.find((g) => g.tag && g.tag === chosen)?.id ??
    groups.find((g) => g.id === plain)?.id ??
    groups.find((g) => g.frames.includes(frame))?.id ??
    groups[0]?.id;

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
      className="relative flex flex-col items-center gap-0.5"
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
        className={`relative rounded-md border p-1 outline-none focus-visible:ring-2 focus-visible:ring-amber ${n === frame ? "border-amber bg-surface-2" : "border-border bg-surface"}`}
      >
        <FrameThumb
          api={api}
          sprite={sprite}
          frame={n}
          revision={revision}
          box={THUMB}
        />
        {n === busyFrame && (
          <span
            aria-hidden
            className="absolute top-1.5 right-1.5 h-2 w-2 rounded-full bg-amber"
          />
        )}
      </button>
      <div className="flex w-full items-center justify-between gap-1 px-0.5">
        <span aria-hidden className="font-mono text-[10px] text-text-faint">
          {n}
        </span>
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
        title={t("frames.menu", { n })}
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

  // Up and Down (and Home, End) move between the headers of the groups.
  const moveBetweenHeaders = (event: KeyboardEvent<HTMLDivElement>) => {
    const keys = ["ArrowUp", "ArrowDown", "Home", "End"];
    const from = (event.target as HTMLElement).closest("[data-group-header]");
    if (!from || !keys.includes(event.key)) return;
    const all = [
      ...event.currentTarget.querySelectorAll<HTMLElement>(
        "[data-group-header]",
      ),
    ];
    const at = all.indexOf(from as HTMLElement);
    const next =
      event.key === "Home"
        ? 0
        : event.key === "End"
          ? all.length - 1
          : Math.max(
              0,
              Math.min(all.length - 1, at + (event.key === "ArrowUp" ? -1 : 1)),
            );
    event.preventDefault();
    all[next]?.focus();
  };

  return (
    <aside
      aria-label={t("editor.frames")}
      className="flex w-40 shrink-0 flex-col border-r border-border bg-bg"
    >
      <h2 className="px-3 pt-3 pb-2 text-xs font-semibold text-text-muted">
        {t("editor.frames")}
      </h2>
      <div
        onKeyDown={moveBetweenHeaders}
        className="flex min-h-0 flex-1 flex-col gap-1 overflow-y-auto px-2 pb-2"
      >
        {overlaps && (
          <p role="status" className="text-[10px] text-text-faint">
            {t("frames.overlap")}
          </p>
        )}
        {groups.map((group) => {
          const selected = group.id === openId;
          return (
            <section
              key={group.id}
              aria-label={
                group.tag?.name ??
                t(`frames.group${group.kind === "base" ? "Base" : "None"}`)
              }
              className="flex flex-col"
            >
              <AnimationHeader
                group={group}
                selected={selected}
                onSelect={() => {
                  if (group.tag) onAnimation(group.tag.name);
                  else {
                    setPlain(group.id);
                    onAnimation(null);
                  }
                }}
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
              {selected && (
                <div className="grid grid-cols-2 justify-items-center gap-2 pt-1 pb-2">
                  {group.frames.map(renderFrame)}
                </div>
              )}
            </section>
          );
        })}
      </div>
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
      <div className="flex shrink-0 flex-col gap-2 border-t border-border p-2">
        <button
          type="button"
          className="h-8 w-full rounded-md border border-border bg-surface text-xs font-semibold text-text outline-none hover:bg-surface-2 focus-visible:ring-2 focus-visible:ring-amber"
          onClick={() => setDialog({ kind: "new" })}
        >
          {t("anim.newOpen")}
        </button>
        <div className="flex gap-1">
          <IconButton
            label={t("frames.add")}
            path={PLUS}
            onClick={() => add(null)}
          />
          <IconButton
            label={t("frames.addCopy")}
            path={COPY}
            onClick={() => add(frame)}
          />
        </div>
      </div>
    </aside>
  );
}
