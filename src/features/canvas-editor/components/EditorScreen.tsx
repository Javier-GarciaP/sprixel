import { useEffect, useMemo, useRef, useState, type RefObject } from "react";
import ghost from "@/assets/sprixel-ghost.png";
import { Button, Icon, Toast } from "@/shared/components";
import {
  FramesColumn,
  PreviewPanel,
  chosenTag,
  nextBackground,
  type PreviewBackground,
  coreFramesApi,
  rangeOf,
  useFramesInfo,
  type Clock,
  type FramesApi,
} from "@/features/frames-animations";
import {
  PaletteEditorDialog,
  PalettePanel,
  corePaletteApi,
  type PaletteApi,
} from "@/features/palette-editor";
import { TextView, coreTextApi, type TextApi } from "@/features/text-view";
import { useT } from "@/shared/lib/i18n";
import { usePreferences } from "@/shared/lib/preferences";
import { coreEditorApi } from "../api/editor";
import { corePowerApi } from "../api/power";
import type {
  Clip,
  EditorApi,
  Anchor,
  AnchorKey,
  PowerApi,
  Preview,
  RegionAction,
  SpriteRef,
} from "../api/types";
import { powerErrorText } from "../powerError";
import { SelectionBar } from "./SelectionBar";
import { OutlinePanel } from "./OutlinePanel";
import { ShadePanel } from "./ShadePanel";
import { ReviewPanel } from "./ReviewPanel";
import { PreviewTools } from "./PreviewTools";
import { AnchorLabels } from "./AnchorLabels";
import { AnchorPanel, type AnchorTarget } from "./AnchorPanel";
import { anchorAt, effectiveAnchors, keyOf, sameKey } from "../tools/anchors";
import {
  useEditorCommands,
  type CommandHandlers,
} from "../hooks/useEditorCommands";
import { sameSprite, useAgentActivity } from "../hooks/useAgentActivity";
import { screenClock, useAgentPlayback } from "../hooks/useAgentPlayback";
import { useFollowAgent } from "../hooks/useFollowAgent";
import { useReducedMotion } from "../hooks/useReducedMotion";
import { useSpriteData, isBlank } from "../hooks/useSpriteData";
import { tools } from "../tools/tools";
import { ColorBook } from "../controller/colorBook";
import { DrawingController } from "../controller/DrawingController";
import type { Scene } from "../render/renderer";
import type { ToolId } from "../tools/tools";
import { axisLines, nextSymmetry, type Symmetry } from "../tools/symmetry";
import type { Rect } from "../types";
import { useCanvasView } from "../view/useCanvasView";
import { AgentCursor } from "./AgentCursor";
import { CanvasArea } from "./CanvasArea";
import type { PixelCanvasHandle } from "./PixelCanvas";
import { CanvasBar } from "./CanvasBar";
import { CanvasStatusBar } from "./CanvasStatusBar";
import { DEFAULT_OPACITY, OnionPanel } from "./OnionPanel";
import { RightPanel } from "./RightPanel";
import { ToolsColumn } from "./ToolsColumn";
import type { PixelChange, SpriteInfo } from "../api/types";

type Props = {
  // The sprite to edit, or null when none is open.
  sprite: SpriteRef | null;
  // The calls to the core. The tests give a fake one.
  api?: EditorApi;
  // The calls of the power tools. The tests give a fake one.
  powerApi?: PowerApi;
  // Gives the app what the editor commands do (the app owns the keys).
  // The calls for frames and animations. The tests give a fake one.
  framesApi?: FramesApi;
  // The calls of the palette editor. The tests give a fake one.
  paletteApi?: PaletteApi;
  // The calls of the Text view. The tests give a fake one.
  textApi?: TextApi;
  // The clock of the preview. The tests give a manual one.
  clock?: Clock;
  onCommands?: (handlers: CommandHandlers | null) => void;
  // With no sprite open, the button that leads to the Project tab.
  onGoToProject?: () => void;
  // An agent edits another sprite: the app may show it (see `useFollowAgent`).
  onFollow?: (sprite: SpriteRef, frame: number) => void;
  // The frame to start at when this sprite opens (an agent works there).
  startFrame?: number;
};

export function EditorScreen({
  sprite,
  api = coreEditorApi,
  powerApi = corePowerApi,
  framesApi = coreFramesApi,
  paletteApi = corePaletteApi,
  textApi = coreTextApi,
  clock,
  onCommands,
  onGoToProject,
  onFollow,
  startFrame,
}: Props) {
  const t = useT();
  // The copied piece. It lives here, above the open sprite, so it stays when
  // another sprite is opened and can be pasted there.
  const [clip, setClip] = useState<Clip | null>(null);
  // True while the person has a stroke in progress (set by the open sprite).
  const strokeActive = useRef(false);
  useFollowAgent(api, sprite, strokeActive, onFollow);
  if (!sprite) {
    return (
      <div className="flex h-full flex-col items-center justify-center gap-3 text-center">
        <img
          src={ghost}
          alt=""
          className="h-20 w-20 opacity-40 grayscale [image-rendering:pixelated]"
        />
        <p className="font-title text-xl font-bold">{t("editor.noSprite")}</p>
        <p className="text-sm text-text-muted">{t("editor.noSpriteHint")}</p>
        <Button variant="primary" onClick={onGoToProject}>
          <Icon name="folder" className="h-4 w-4" />
          {t("editor.goToProject")}
        </Button>
      </div>
    );
  }
  // Another sprite starts from the first frame, with nothing left over.
  return (
    <OpenSprite
      key={JSON.stringify(sprite)}
      sprite={sprite}
      api={api}
      powerApi={powerApi}
      framesApi={framesApi}
      paletteApi={paletteApi}
      textApi={textApi}
      clock={clock}
      onCommands={onCommands}
      strokeActive={strokeActive}
      startFrame={startFrame}
      clip={clip}
      onClip={setClip}
    />
  );
}

function OpenSprite({
  sprite,
  api,
  powerApi,
  framesApi,
  paletteApi,
  textApi,
  clock,
  onCommands,
  strokeActive,
  startFrame,
  clip,
  onClip,
}: {
  clip: Clip | null;
  onClip: (clip: Clip) => void;
  startFrame?: number;
  sprite: SpriteRef;
  api: EditorApi;
  powerApi: PowerApi;
  framesApi: FramesApi;
  paletteApi: PaletteApi;
  textApi: TextApi;
  clock?: Clock;
  onCommands?: (handlers: CommandHandlers | null) => void;
  strokeActive: RefObject<boolean>;
}) {
  const t = useT();
  const { preferences } = usePreferences();
  const [frame, setFrame] = useState(startFrame ?? 1);
  const [tool, setTool] = useState<ToolId>("pencil");
  const [main, setMain] = useState("#000000");
  const [secondary, setSecondary] = useState("#FFFFFF");
  const [filled, setFilled] = useState(false);
  const [symmetry, setSymmetry] = useState<Symmetry>("off");
  const [onion, setOnion] = useState({ on: true, opacity: DEFAULT_OPACITY });
  const { data, previous, reload } = useSpriteData(
    api,
    sprite,
    frame,
    onion.on,
  );

  // A frame that no longer exists (after an undo, for example) goes to the
  // last one.
  const frameCount = data.status === "ready" ? data.info.frameCount : null;
  useEffect(() => {
    if (frameCount !== null && frame > frameCount) setFrame(frameCount);
  }, [frameCount, frame]);

  if (data.status === "loading") return null;
  if (data.status === "error") {
    return (
      <p role="alert" className="p-6 text-sm text-error">
        {t("editor.loadError", { message: data.message })}
      </p>
    );
  }
  return (
    <Ready
      api={api}
      powerApi={powerApi}
      framesApi={framesApi}
      paletteApi={paletteApi}
      textApi={textApi}
      clock={clock}
      onFramesChanged={(select) => {
        if (select) setFrame(select);
        reload();
      }}
      onCommands={onCommands}
      strokeActive={strokeActive}
      clip={clip}
      onClip={onClip}
      reload={reload}
      sprite={sprite}
      info={data.info}
      pixels={data.pixels}
      previous={previous}
      frame={frame}
      onFrame={setFrame}
      tool={tool}
      onTool={setTool}
      main={main}
      secondary={secondary}
      onMain={setMain}
      onSecondary={setSecondary}
      onSwap={() => {
        setMain(secondary);
        setSecondary(main);
      }}
      filled={filled}
      onFilled={setFilled}
      symmetry={symmetry}
      onSymmetry={setSymmetry}
      onion={onion}
      onOnion={setOnion}
      pixelGrid={preferences.pixelGrid}
    />
  );
}

type ReadyProps = {
  clip: Clip | null;
  onClip: (clip: Clip) => void;
  api: EditorApi;
  powerApi: PowerApi;
  clock?: Clock;
  framesApi: FramesApi;
  paletteApi: PaletteApi;
  textApi: TextApi;
  onFramesChanged: (select?: number) => void;
  onCommands?: (handlers: CommandHandlers | null) => void;
  strokeActive: RefObject<boolean>;
  // Reads the sprite from the core again.
  reload: () => void;
  sprite: SpriteRef;
  info: SpriteInfo;
  pixels: Uint8Array;
  previous: Uint8Array | null;
  frame: number;
  onFrame: (frame: number) => void;
  tool: ToolId;
  onTool: (tool: ToolId) => void;
  main: string;
  secondary: string;
  onMain: (hex: string) => void;
  onSecondary: (hex: string) => void;
  onSwap: () => void;
  filled: boolean;
  onFilled: (filled: boolean) => void;
  symmetry: Symmetry;
  onSymmetry: (symmetry: Symmetry) => void;
  onion: { on: boolean; opacity: number };
  onOnion: (onion: { on: boolean; opacity: number }) => void;
  pixelGrid: boolean;
};

function Ready(props: ReadyProps) {
  const t = useT();
  const { info, pixels, sprite } = props;
  // The animation that filters the frames (null is "All").
  const framesInfo = useFramesInfo(props.framesApi, sprite);
  const tags = framesInfo?.tags ?? [];
  const [animation, setAnimation] = useState<string | null>(null);
  const chooseAnimation = (name: string | null) => {
    setAnimation(name);
    const range = rangeOf(tags, name);
    // The current frame goes into the animation when it is outside of it.
    if (range && (props.frame < range.from || props.frame > range.to)) {
      props.onFrame(range.from);
    }
  };
  const canvas = useCanvasView(info.width, info.height);
  const [position, setPosition] = useState<{ x: number; y: number } | null>(
    null,
  );

  // The drawing controller works on `pixels` for as long as this copy of the
  // frame lasts. A new copy from the core makes a new controller.
  const canvasRef = useRef<PixelCanvasHandle>(null);
  const send = useRef<(changes: PixelChange[]) => void>(() => {});
  // The selected rectangle. The controller draws and edits it; the bar and
  // the actions read this copy. A new controller (a new copy of the pixels
  // from the core) starts with the same selection.
  const [selection, setSelection] = useState<Rect | null>(null);
  const selectionRef = useRef<Rect | null>(null);
  const [wrap, setWrap] = useState(false);
  // What shows behind the transparent pixels of the preview.
  const [background, setBackground] = useState<PreviewBackground>("checker");
  // The panel with a preview that is open, and the pixels it shows.
  const [panel, setPanel] = useState<"outline" | "shade" | "review" | null>(
    null,
  );
  // The anchors of the sprite, as the core has them, and the Anchor tool: the
  // one being edited, and the one being dragged (with where it is now).
  const [anchors, setAnchors] = useState<Anchor[]>([]);
  const [anchorTarget, setAnchorTarget] = useState<AnchorTarget | null>(null);
  const [anchorError, setAnchorError] = useState<string | null>(null);
  const dragged = useRef<Anchor | null>(null);
  const [dragPos, setDragPos] = useState<{ x: number; y: number } | null>(null);
  useEffect(() => {
    let stale = false;
    props.powerApi
      .anchors(sprite)
      .then((list) => !stale && setAnchors(list))
      .catch(() => {});
    return () => {
      stale = true;
    };
  }, [props.powerApi, sprite, info.revision]);
  const frameAnchors = useMemo(
    () => effectiveAnchors(anchors, props.frame),
    [anchors, props.frame],
  );
  const shownAnchors = frameAnchors.map((anchor) =>
    dragged.current && dragPos && sameKey(keyOf(anchor), keyOf(dragged.current))
      ? { ...anchor, ...dragPos }
      : anchor,
  );
  const saveAnchor = (anchor: Anchor, replaces: AnchorKey | null) =>
    props.powerApi
      .saveAnchor(sprite, anchor, replaces)
      .then(() => {
        setAnchorError(null);
        setAnchorTarget({ kind: "edit", key: keyOf(anchor) });
        props.reload();
      })
      .catch((error: { code?: string; message?: string }) =>
        setAnchorError(
          t("anchor.error", { message: error?.message ?? error?.code ?? "" }),
        ),
      );
  const deleteAnchor = (key: AnchorKey) =>
    props.powerApi
      .deleteAnchor(sprite, key)
      .then(() => {
        setAnchorError(null);
        props.reload();
      })
      .catch((error: { code?: string; message?: string }) =>
        setAnchorError(
          t("anchor.error", { message: error?.message ?? error?.code ?? "" }),
        ),
      );
  // The Anchor tool uses the pointer on the canvas: a click on an anchor
  // selects it and a drag moves it; a click elsewhere starts a new anchor.
  const anchorDown = (x: number, y: number) => {
    const hit = anchorAt(frameAnchors, x, y);
    if (hit) {
      dragged.current = hit;
      setAnchorTarget({ kind: "edit", key: keyOf(hit) });
    } else if (x >= 0 && y >= 0 && x < info.width && y < info.height) {
      setAnchorTarget({ kind: "new", x, y });
    }
  };
  const anchorMove = (x: number, y: number) => {
    if (!dragged.current) return;
    if (x >= 0 && y >= 0 && x < info.width && y < info.height) {
      setDragPos({ x, y });
    }
  };
  const anchorUp = () => {
    const anchor = dragged.current;
    const to = dragPos;
    dragged.current = null;
    setDragPos(null);
    if (anchor && to && (to.x !== anchor.x || to.y !== anchor.y)) {
      void saveAnchor({ ...anchor, ...to }, keyOf(anchor));
    }
  };
  // The pixel the Review panel points at.
  const [pointed, setPointed] = useState<{
    frame: number;
    x: number;
    y: number;
  } | null>(null);
  const [preview, setPreview] = useState<Preview["pixels"] | null>(null);
  const controller = useMemo(() => {
    const made = new DrawingController(
      info.width,
      info.height,
      pixels,
      new ColorBook(info.palette),
      {
        onDirty: (area) => canvasRef.current?.invalidate(area),
        onCommit: (changes) => send.current(changes),
        onPick: (hex, button) =>
          button === 2 ? props.onSecondary(hex) : props.onMain(hex),
        onSelect: (area) => {
          selectionRef.current = area;
          setSelection(area);
        },
      },
    );
    made.selection = selectionRef.current;
    return made;
    // A new copy of the pixels is a new controller.
  }, [pixels]);
  // The palette of the sprite as the core has it. A stroke can add colors.
  const [palette, setPalette] = useState(info.palette);
  const [history, setHistory] = useState({
    canUndo: info.canUndo,
    canRedo: info.canRedo,
  });
  const [notice, setNotice] = useState<string | null>(null);
  // A message that is not an error (for example "Copied").
  const [message, setMessage] = useState<string | null>(null);
  const [editingPalette, setEditingPalette] = useState(false);
  // The editor shows the sprite as a canvas or as text. The sprite and the
  // frame are the same in both.
  const [mode, setMode] = useState<"canvas" | "text">("canvas");
  useEffect(() => {
    setPalette(info.palette);
    setHistory({ canUndo: info.canUndo, canRedo: info.canRedo });
  }, [info]);
  useEffect(() => {
    if (!notice) return;
    const timer = setTimeout(() => setNotice(null), 5000);
    return () => clearTimeout(timer);
  }, [notice]);
  useEffect(() => {
    if (!message) return;
    const timer = setTimeout(() => setMessage(null), 3000);
    return () => clearTimeout(timer);
  }, [message]);
  // The agent that is writing, and the strokes held while it works on the
  // frame the user is drawing on. They are sent when the agent ends.
  const { activity, endCount } = useAgentActivity(props.api);
  const writing =
    activity && sameSprite(activity.sprite, sprite) ? activity : null;
  const busy = writing !== null && writing.frame === props.frame;
  const busyRef = useRef(false);
  busyRef.current = busy;
  // The agent's pixels appear on the canvas one after another.
  const reducedMotion = useReducedMotion();
  const { cursor } = useAgentPlayback({
    api: props.api,
    sprite,
    frame: props.frame,
    pixels,
    width: info.width,
    height: info.height,
    canvasRef,
    clock: props.clock ?? screenClock,
    reducedMotion,
    strokeActive: props.strokeActive,
    onFrame: props.onFrame,
    endCount,
  });
  const held = useRef<{ frame: number; changes: PixelChange[] }[]>([]);
  const failed = (error: { code?: string; message?: string }) => {
    // The core is the truth: read the frame again so the screen shows what
    // the core has, without the stroke it refused.
    if (error?.code === "palette_full") setNotice(t("colors.paletteFull"));
    else setNotice(t("editor.strokeError", { message: error?.message ?? "" }));
    props.reload();
  };
  send.current = (changes) => {
    if (busyRef.current) {
      held.current.push({ frame: props.frame, changes });
      return;
    }
    props.api
      .applyChanges(sprite, props.frame, changes)
      .then((result) => {
        controller.settle(result.palette);
        setPalette(result.palette);
        setHistory(result);
      })
      .catch((error: { code?: string; message?: string }) => {
        // The core knows the frame is busy before the screen does: keep the
        // stroke like the others.
        if (error?.code === "frame_busy") {
          held.current.push({ frame: props.frame, changes });
        } else failed(error);
      });
  };
  // When an agent ends: send the held strokes in order, then read the sprite
  // again to show what the agent drew.
  useEffect(() => {
    if (endCount === 0) return;
    const items = held.current.splice(0);
    (async () => {
      for (const item of items) {
        try {
          await props.api.applyChanges(sprite, item.frame, item.changes);
        } catch (error) {
          setNotice(
            t("editor.strokeError", {
              message: (error as { message?: string })?.message ?? "",
            }),
          );
        }
      }
      props.reload();
    })();
    // Only a new end of work runs this.
  }, [endCount]);
  useEffect(() => {
    controller.tool = props.tool;
    controller.main = props.main;
    controller.secondary = props.secondary;
    controller.filled = props.filled;
    controller.symmetry = props.symmetry;
  }, [
    controller,
    props.tool,
    props.main,
    props.secondary,
    props.filled,
    props.symmetry,
  ]);

  // Undo and redo are done by the core. The screen then reads the sprite
  // again, because the change can be anywhere (pixels and palette).
  const step = (call: () => Promise<{ canUndo: boolean; canRedo: boolean }>) =>
    call()
      .then((state) => {
        setHistory(state);
        props.reload();
      })
      .catch(() => {
        // Nothing to undo or redo.
      });
  const undo = () => step(() => props.api.undo());
  const redo = () => step(() => props.api.redo());

  // An action of the power tools on the selection (the whole frame when
  // nothing is selected). The core does it as one step of Undo; then the
  // screen reads the sprite again.
  const runRegion = (action: RegionAction) => {
    if (busyRef.current)
      return setNotice(powerErrorText(t, { code: "frame_busy" }));
    props.powerApi
      .region(sprite, props.frame, selectionRef.current, action)
      .then((result) => {
        setHistory(result);
        props.reload();
      })
      .catch((error) => setNotice(powerErrorText(t, error)));
  };
  // Copy the selection (the whole frame if nothing is selected), and paste it
  // at the top left of the selection (or of the frame). What is pasted
  // becomes the selection, so it can be moved or flipped. The `.` of the copy
  // do not erase what is under them.
  const copy = () =>
    props.powerApi
      .copy(sprite, props.frame, selectionRef.current)
      .then((clip) => {
        props.onClip(clip);
        setMessage(
          t("selection.copied", { width: clip.width, height: clip.height }),
        );
      })
      .catch((error) => setNotice(powerErrorText(t, error)));
  const paste = () => {
    const clip = props.clip;
    if (!clip) return setNotice(t("power.nothingToPaste"));
    if (busyRef.current) {
      return setNotice(powerErrorText(t, { code: "frame_busy" }));
    }
    const at = {
      x: selectionRef.current?.x ?? 0,
      y: selectionRef.current?.y ?? 0,
    };
    props.powerApi
      .paste(sprite, props.frame, clip, at, true)
      .then((result) => {
        const area = { ...at, width: clip.width, height: clip.height };
        selectionRef.current = area;
        setSelection(area);
        setHistory(result);
        props.reload();
      })
      .catch((error) => setNotice(powerErrorText(t, error)));
  };
  const nudge = (dx: number, dy: number) =>
    runRegion({ kind: "move", dx, dy, wrap });

  useEditorCommands(props.onCommands, {
    "edit.undo": undo,
    "edit.redo": redo,
    ...Object.fromEntries(
      tools.map((tool) => [`tool.${tool.id}`, () => props.onTool(tool.id)]),
    ),
    "frame.previous": () => props.onFrame(Math.max(1, props.frame - 1)),
    "frame.next": () =>
      props.onFrame(Math.min(info.frameCount, props.frame + 1)),
    "colors.swap": props.onSwap,
    "selection.flipHorizontal": () => runRegion({ kind: "flip_horizontal" }),
    "selection.flipVertical": () => runRegion({ kind: "flip_vertical" }),
    "selection.rotate90": () => runRegion({ kind: "rotate", degrees: 90 }),
    "selection.rotate180": () => runRegion({ kind: "rotate", degrees: 180 }),
    "selection.rotate270": () => runRegion({ kind: "rotate", degrees: 270 }),
    "selection.mirrorLeftToRight": () =>
      runRegion({ kind: "mirror", direction: "left_to_right" }),
    "selection.mirrorRightToLeft": () =>
      runRegion({ kind: "mirror", direction: "right_to_left" }),
    "selection.mirrorTopToBottom": () =>
      runRegion({ kind: "mirror", direction: "top_to_bottom" }),
    "selection.mirrorBottomToTop": () =>
      runRegion({ kind: "mirror", direction: "bottom_to_top" }),
    "selection.moveLeft": () => nudge(-1, 0),
    "selection.moveRight": () => nudge(1, 0),
    "selection.moveUp": () => nudge(0, -1),
    "selection.moveDown": () => nudge(0, 1),
    "selection.wrap": () => setWrap((on) => !on),
    "view.background": () => setBackground(nextBackground),
    "draw.symmetry": () => props.onSymmetry(nextSymmetry(props.symmetry)),
    "edit.outline": () =>
      setPanel((now) => (now === "outline" ? null : "outline")),
    "edit.shade": () => setPanel((now) => (now === "shade" ? null : "shade")),
    "view.review": () =>
      setPanel((now) => (now === "review" ? null : "review")),
    "edit.copy": copy,
    "edit.paste": paste,
    "view.onionSkin": () =>
      props.onOnion({ ...props.onion, on: !props.onion.on }),
    "view.zoomIn": () => canvas.zoomIn(),
    "view.zoomOut": () => canvas.zoomOut(),
    "view.fit": canvas.fit,
  });

  const agentText = writing
    ? t("marker.drawing", { agent: writing.agent, frame: writing.frame })
    : "";

  const scene = useMemo<Scene>(
    () => ({
      width: info.width,
      height: info.height,
      pixels,
      colors: controller.colors.table,
      onion:
        props.onion.on && props.previous
          ? { pixels: props.previous, opacity: props.onion.opacity / 100 }
          : null,
      grid: props.pixelGrid,
      anchors: shownAnchors.map((a) => ({
        x: a.x,
        y: a.y,
        selected:
          anchorTarget?.kind === "edit" && sameKey(anchorTarget.key, keyOf(a)),
      })),
      axis: axisLines(info.width, info.height, props.symmetry),
      highlight:
        pointed && pointed.frame === props.frame
          ? { x: pointed.x, y: pointed.y }
          : null,
      overlay: preview?.map(([x, y, key]) => ({
        x,
        y,
        code: key.charCodeAt(0),
      })),
      // The controller owns the selection and repaints its area.
      get selection() {
        return controller.selection;
      },
    }),
    [
      info.width,
      info.height,
      pixels,
      controller,
      props.pixelGrid,
      props.symmetry,
      preview,
      pointed,
      anchors,
      anchorTarget,
      dragPos,
      props.frame,
      props.previous,
      props.onion.on,
      props.onion.opacity,
    ],
  );

  return (
    <div className="flex h-full min-h-0">
      <ToolsColumn
        tool={props.tool}
        onTool={props.onTool}
        main={props.main}
        secondary={props.secondary}
        onMain={props.onMain}
        onSecondary={props.onSecondary}
        onSwap={props.onSwap}
      />
      <FramesColumn
        api={props.framesApi}
        sprite={sprite}
        frameCount={info.frameCount}
        frame={props.frame}
        onFrame={props.onFrame}
        onChanged={props.onFramesChanged}
        busyFrame={writing?.frame}
        durations={framesInfo?.durations}
        tags={tags}
        animation={animation}
        onAnimation={chooseAnimation}
      />
      <section className="relative flex min-w-0 flex-1 flex-col">
        <CanvasBar
          name={sprite.name}
          width={info.width}
          height={info.height}
          tool={props.tool}
          canUndo={history.canUndo}
          canRedo={history.canRedo}
          onUndo={undo}
          onRedo={redo}
          filled={props.filled}
          onFilled={props.onFilled}
          symmetry={props.symmetry}
          onSymmetry={props.onSymmetry}
          panel={panel}
          onPanel={setPanel}
          agent={
            writing && {
              text: agentText,
              onStop: () => void props.api.stopAgent(),
            }
          }
          mode={mode}
          onMode={setMode}
        />
        {mode === "canvas" && selection && (
          <SelectionBar
            selection={selection}
            wrap={wrap}
            onWrap={setWrap}
            onAction={runRegion}
            onCopy={copy}
            onPaste={props.clip ? paste : null}
          />
        )}
        {mode === "text" && (
          <TextView
            api={props.textApi}
            sprite={sprite}
            frame={props.frame}
            frameCount={info.frameCount}
            revision={info.revision}
            onFrame={props.onFrame}
            onApplied={props.reload}
          />
        )}
        <div
          data-testid={writing ? "agent-border" : undefined}
          className={`relative min-h-0 flex-1 ${mode === "text" ? "hidden" : "flex"} ${writing ? "outline-2 -outline-offset-2 outline-amber" : ""}`}
        >
          <CanvasArea
            scene={scene}
            view={canvas.view}
            onSize={(width, height) => canvas.setArea({ width, height })}
            onZoom={(direction, at) =>
              direction === 1 ? canvas.zoomIn(at) : canvas.zoomOut(at)
            }
            onMove={canvas.moveTo}
            onHover={setPosition}
            canvasRef={canvasRef}
            onStrokeStart={(pixel, button, shift) => {
              props.strokeActive.current = true;
              if (props.tool === "anchor") return anchorDown(pixel.x, pixel.y);
              controller.pointerDown(pixel.x, pixel.y, button, { shift });
            }}
            onStrokeMove={(pixel, shift) =>
              props.tool === "anchor"
                ? anchorMove(pixel.x, pixel.y)
                : controller.pointerMove(pixel.x, pixel.y, { shift })
            }
            onStrokeEnd={() => {
              props.strokeActive.current = false;
              if (props.tool === "anchor") return anchorUp();
              controller.pointerUp();
            }}
          />
          {cursor && <AgentCursor view={canvas.view} {...cursor} />}
          <AnchorLabels view={canvas.view} anchors={shownAnchors} />
        </div>
        {mode === "canvas" && isBlank(pixels) && (
          <p className="pointer-events-none absolute bottom-12 left-1/2 -translate-x-1/2 rounded-md border border-amber bg-surface px-4 py-2 text-xs text-text-muted">
            {t("editor.hint")}
          </p>
        )}
        {mode === "canvas" && (
          <CanvasStatusBar
            position={position}
            zoom={canvas.view.zoom}
            onZoomIn={() => canvas.zoomIn()}
            onZoomOut={() => canvas.zoomOut()}
            onFit={canvas.fit}
          />
        )}
        {panel === "outline" && (
          <OutlinePanel
            api={props.powerApi}
            sprite={sprite}
            frame={props.frame}
            palette={palette}
            onPreview={setPreview}
            onApplied={(result) => {
              setHistory(result);
              setPanel(null);
              props.reload();
            }}
            onClose={() => setPanel(null)}
          />
        )}
        {panel === "shade" && (
          <ShadePanel
            api={props.powerApi}
            sprite={sprite}
            frame={props.frame}
            palette={palette}
            selection={selection}
            onPreview={setPreview}
            onApplied={(result) => {
              setHistory(result);
              setPanel(null);
              props.reload();
            }}
            onClose={() => setPanel(null)}
          />
        )}
        {panel === "review" && (
          <ReviewPanel
            api={props.powerApi}
            sprite={sprite}
            palette={palette}
            revision={info.revision}
            onPoint={(frame, x, y) => {
              props.onFrame(frame);
              setPointed({ frame, x, y });
            }}
            onClose={() => {
              setPanel(null);
              setPointed(null);
            }}
          />
        )}
        {props.tool === "anchor" && panel === null && (
          <AnchorPanel
            anchors={frameAnchors}
            frame={props.frame}
            width={info.width}
            height={info.height}
            target={anchorTarget}
            onTarget={setAnchorTarget}
            onSave={saveAnchor}
            onDelete={deleteAnchor}
            error={anchorError}
          />
        )}
      </section>
      <RightPanel
        preview={
          <>
            <PreviewPanel
              api={props.framesApi}
              sprite={sprite}
              info={framesInfo}
              tag={chosenTag(tags, animation)}
              clock={props.clock}
              background={background}
            />
            <PreviewTools
              background={background}
              onBackground={setBackground}
            />
          </>
        }
        agent={writing ? agentText : null}
        palette={
          <PalettePanel
            colors={palette}
            main={props.main}
            secondary={props.secondary}
            onMain={props.onMain}
            onSecondary={props.onSecondary}
            onEdit={() => setEditingPalette(true)}
          />
        }
        onion={
          <OnionPanel
            on={props.onion.on}
            onToggle={(on) => props.onOnion({ ...props.onion, on })}
            opacity={props.onion.opacity}
            onOpacity={(opacity) => props.onOnion({ ...props.onion, opacity })}
          />
        }
      />
      {editingPalette && (
        <PaletteEditorDialog
          api={props.paletteApi}
          name={info.paletteName}
          agentBusy={writing !== null}
          onClose={() => setEditingPalette(false)}
          onApplied={props.reload}
        />
      )}
      {notice && <Toast tone="error" message={notice} />}
      {!notice && message && <Toast message={message} />}
    </div>
  );
}
