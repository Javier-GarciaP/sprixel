import type { CoreFailure, ProjectTree, SpriteTarget } from "@/shared/lib/core";

// Which sprites to export. The names are the ones of the Rust enum `ScopeArg`.
export type Scope =
  | { kind: "project" }
  | { kind: "folder"; path: string[] }
  | { kind: "sprites"; sprites: SpriteTarget[] };

export type ScopeKind = Scope["kind"];

// `sheet` is "Sheet + JSON" and `png` is "PNG files". GIF comes after the demo.
export type Format = "sheet" | "png";

export type Options = {
  scale: 1 | 2 | 4 | 8;
  spacing: number;
  // `null` is the default pattern.
  naming: string | null;
};

// What an export would do (see `SummaryInfo` in the core).
export type Summary = {
  sprites: number;
  frames: number;
  files: string[];
  sheetWidth: number;
  sheetHeight: number;
};

// The preview of the sheet: RGBA pixels, row by row.
export type Preview = {
  width: number;
  height: number;
  rgba: Uint8ClampedArray;
};

export type Written = { output: string; files: string[] };

// Everything the Export tab asks of the Rust core. The real calls are in
// `api.ts`; the tests use a fake with the same shape.
export interface ExportApi {
  loadTree(): Promise<ProjectTree>;
  summary(scope: Scope, format: Format, options: Options): Promise<Summary>;
  preview(scope: Scope, options: Options): Promise<Preview>;
  // Rejects with a `CoreFailure` (`no_permission`, `no_space`, ...).
  run(
    scope: Scope,
    format: Format,
    options: Options,
    output: string,
  ): Promise<Written>;
  reveal(output: string): Promise<void>;
  onChanged(listener: () => void): Promise<() => void>;
}

export type { CoreFailure };
