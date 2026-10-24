import { useEffect, useMemo, useRef, useState } from "react";
import { useT } from "@/shared/lib/i18n";
import { usePreferences } from "@/shared/lib/preferences";
import { coreEditorApi } from "../api/editor";
import type { EditorApi, SpriteRef } from "../api/types";
import {
  useEditorCommands,
  type CommandHandlers,
} from "../hooks/useEditorCommands";
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
import { FramesColumn } from "./FramesColumn";
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
  onCommands?: (handlers: CommandHandlers | null) => void;
};

export function EditorScreen({
  sprite,
  api = coreEditorApi,
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
  return <OpenSprite sprite={sprite} api={api} onCommands={onCommands} />;
}

function OpenSprite({
  sprite,
  api,
  onCommands,
}: {
  sprite: SpriteRef;
  api: EditorApi;
  onCommands?: (handlers: CommandHandlers | null) => void;
}) {
  const t = useT();
  const { preferences } = usePreferences();
  const [frame, setFrame] = useState(1);
  const [tool, setTool] = useState<ToolId>("pencil");
  const [main, setMain] = useState("#000000");
  const [secondary, setSecondary] = useState("#FFFFFF");
  const [filled, setFilled] = useState(false);
  const [onion, setOnion] = useState({ on: false, opacity: DEFAULT_OPACITY });
  const { data, reload } = useSpriteData(api, sprite, frame, onion.on);

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
      onCommands={onCommands}
      reload={reload}
      sprite={sprite}
      info={data.info}
      pixels={data.pixels}
      previous={data.previous}
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
  const canvas = useCanvasView(info.width, info.height);
  const [position, setPosition] = useState<{ x: number; y: number } | null>(
    null,
  );

  // The drawing controller works on `pixels` for as long as this copy of the
  // frame lasts. A new copy from the core makes a new controller.
  const canvasRef = useRef<PixelCanvasHandle>(null);
  const send = useRef<(changes: PixelChange[]) => void>(() => {});
  send.current = (changes) => {
    props.api
      .applyChanges(sprite, props.frame, changes)
      .catch(() => props.reload());
  };
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
  useEffect(() => {
    controller.tool = props.tool;
    controller.main = props.main;
    controller.secondary = props.secondary;
    controller.filled = props.filled;
  }, [controller, props.tool, props.main, props.secondary, props.filled]);

  useEditorCommands(props.onCommands, {
    ...Object.fromEntries(
      tools.map((tool) => [`tool.${tool.id}`, () => props.onTool(tool.id)]),
    ),
    "colors.swap": props.onSwap,
    "view.zoomIn": () => canvas.zoomIn(),
    "view.zoomOut": () => canvas.zoomOut(),
    "view.fit": canvas.fit,
  });

  const scene = useMemo<Scene>(
    () => ({
      width: info.width,
      height: info.height,
      pixels,
      colors: controller.colors.table,
      onion: null,
      grid: props.pixelGrid,
      // The controller owns the selection and repaints its area.
      get selection() {
        return controller.selection;
      },
    }),
    [info.width, info.height, pixels, controller, props.pixelGrid],
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
        frameCount={info.frameCount}
        frame={props.frame}
        onFrame={props.onFrame}
      />
      <section className="flex min-w-0 flex-1 flex-col">
        <CanvasBar
          name={sprite.name}
          width={info.width}
          height={info.height}
          tool={props.tool}
          canUndo={info.canUndo}
          canRedo={info.canRedo}
          onUndo={() => {}}
          onRedo={() => {}}
          filled={props.filled}
          onFilled={props.onFilled}
        />
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
        palette={
          <PalettePanel
            colors={info.palette}
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
    </div>
  );
}
