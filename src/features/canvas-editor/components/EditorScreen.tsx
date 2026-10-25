import { useEffect, useMemo, useRef, useState } from "react";
import { Toast } from "@/shared/components";
import {
  FramesColumn,
  PreviewPanel,
  chosenTag,
  coreFramesApi,
  rangeOf,
  useFramesInfo,
  type Clock,
  type FramesApi,
} from "@/features/frames-animations";
import { useT } from "@/shared/lib/i18n";
import { usePreferences } from "@/shared/lib/preferences";
import { coreEditorApi } from "../api/editor";
import type { EditorApi, SpriteRef } from "../api/types";
import {
  useEditorCommands,
  type CommandHandlers,
} from "../hooks/useEditorCommands";
import { sameSprite, useAgentActivity } from "../hooks/useAgentActivity";
import { useSpriteData, isBlank } from "../hooks/useSpriteData";
import { tools } from "../tools/tools";
import { ColorBook } from "../controller/colorBook";
import { DrawingController } from "../controller/DrawingController";
import type { Scene } from "../render/renderer";
import type { ToolId } from "../tools/tools";
import { useCanvasView } from "../view/useCanvasView";
import { CanvasArea } from "./CanvasArea";
import type { PixelCanvasHandle } from "./PixelCanvas";
import { CanvasBar } from "./CanvasBar";
import { CanvasStatusBar } from "./CanvasStatusBar";
import { DEFAULT_OPACITY, OnionPanel } from "./OnionPanel";
import { PalettePanel } from "./PalettePanel";
import { RightPanel } from "./RightPanel";
import { ToolsColumn } from "./ToolsColumn";
import type { PixelChange, SpriteInfo } from "../api/types";

type Props = {
  // The sprite to edit, or null when none is open.
  sprite: SpriteRef | null;
  // The calls to the core. The tests give a fake one.
  api?: EditorApi;
  // Gives the app what the editor commands do (the app owns the keys).
  // The calls for frames and animations. The tests give a fake one.
  framesApi?: FramesApi;
  // The clock of the preview. The tests give a manual one.
  clock?: Clock;
  onCommands?: (handlers: CommandHandlers | null) => void;
};

export function EditorScreen({
  sprite,
  api = coreEditorApi,
  framesApi = coreFramesApi,
  clock,
  onCommands,
}: Props) {
  const t = useT();
  if (!sprite) {
    return (
      <div className="flex h-full flex-col items-center justify-center gap-1 text-center">
        <p className="font-title text-lg font-bold">{t("editor.noSprite")}</p>
        <p className="text-sm text-text-muted">{t("editor.noSpriteHint")}</p>
      </div>
    );
  }
  // Another sprite starts from the first frame, with nothing left over.
  return (
    <OpenSprite
      key={JSON.stringify(sprite)}
      sprite={sprite}
      api={api}
      framesApi={framesApi}
      clock={clock}
      onCommands={onCommands}
    />
  );
}

function OpenSprite({
  sprite,
  api,
  framesApi,
  clock,
  onCommands,
}: {
  sprite: SpriteRef;
  api: EditorApi;
  framesApi: FramesApi;
  clock?: Clock;
  onCommands?: (handlers: CommandHandlers | null) => void;
}) {
  const t = useT();
  const { preferences } = usePreferences();
  const [frame, setFrame] = useState(1);
  const [tool, setTool] = useState<ToolId>("pencil");
  const [main, setMain] = useState("#000000");
  const [secondary, setSecondary] = useState("#FFFFFF");
  const [filled, setFilled] = useState(false);
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
      framesApi={framesApi}
      clock={clock}
      onFramesChanged={(select) => {
        if (select) setFrame(select);
        reload();
      }}
      onCommands={onCommands}
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
      onion={onion}
      onOnion={setOnion}
      pixelGrid={preferences.pixelGrid}
    />
  );
}

type ReadyProps = {
  api: EditorApi;
  clock?: Clock;
  framesApi: FramesApi;
  onFramesChanged: (select?: number) => void;
  onCommands?: (handlers: CommandHandlers | null) => void;
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
  const controller = useMemo(
    () =>
      new DrawingController(
        info.width,
        info.height,
        pixels,
        new ColorBook(info.palette),
        {
          onDirty: (area) => canvasRef.current?.invalidate(area),
          onCommit: (changes) => send.current(changes),
          onPick: (hex, button) =>
            button === 2 ? props.onSecondary(hex) : props.onMain(hex),
        },
      ),
    // A new copy of the pixels is a new controller.
    [pixels],
  );
  // The palette of the sprite as the core has it. A stroke can add colors.
  const [palette, setPalette] = useState(info.palette);
  const [history, setHistory] = useState({
    canUndo: info.canUndo,
    canRedo: info.canRedo,
  });
  const [notice, setNotice] = useState<string | null>(null);
  useEffect(() => {
    setPalette(info.palette);
    setHistory({ canUndo: info.canUndo, canRedo: info.canRedo });
  }, [info]);
  useEffect(() => {
    if (!notice) return;
    const timer = setTimeout(() => setNotice(null), 5000);
    return () => clearTimeout(timer);
  }, [notice]);
  // The agent that is writing, and the strokes held while it works on the
  // frame the user is drawing on. They are sent when the agent ends.
  const { activity, endCount } = useAgentActivity(props.api);
  const writing =
    activity && sameSprite(activity.sprite, sprite) ? activity : null;
  const busy = writing !== null && writing.frame === props.frame;
  const busyRef = useRef(false);
  busyRef.current = busy;
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
  }, [controller, props.tool, props.main, props.secondary, props.filled]);

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
        tags={tags}
        animation={animation}
        onAnimation={chooseAnimation}
      />
      <section className="flex min-w-0 flex-1 flex-col">
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
          agent={
            writing && {
              text: agentText,
              onStop: () => void props.api.stopAgent(),
            }
          }
        />
        <div
          data-testid={writing ? "agent-border" : undefined}
          className={`flex min-h-0 flex-1 ${writing ? "outline-2 -outline-offset-2 outline-amber" : ""}`}
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
            onStrokeStart={(pixel, button, shift) =>
              controller.pointerDown(pixel.x, pixel.y, button, { shift })
            }
            onStrokeMove={(pixel, shift) =>
              controller.pointerMove(pixel.x, pixel.y, { shift })
            }
            onStrokeEnd={() => controller.pointerUp()}
          />
        </div>
        {isBlank(pixels) && (
          <p className="mx-auto mb-3 rounded-md border border-amber bg-surface px-4 py-2 text-xs text-text-muted">
            {t("editor.hint")}
          </p>
        )}
        <CanvasStatusBar
          position={position}
          zoom={canvas.view.zoom}
          onZoomIn={() => canvas.zoomIn()}
          onZoomOut={() => canvas.zoomOut()}
          onFit={canvas.fit}
        />
      </section>
      <RightPanel
        preview={
          <PreviewPanel
            api={props.framesApi}
            sprite={sprite}
            info={framesInfo}
            tag={chosenTag(tags, animation)}
            clock={props.clock}
          />
        }
        agent={writing ? agentText : null}
        palette={
          <PalettePanel
            colors={palette}
            main={props.main}
            secondary={props.secondary}
            onMain={props.onMain}
            onSecondary={props.onSecondary}
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
      {notice && <Toast tone="error" message={notice} />}
    </div>
  );
}
