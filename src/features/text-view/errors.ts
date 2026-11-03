import type { TranslationKey } from "@/shared/lib/i18n";
import type { GridError } from "./types";

type Translate = (
  key: TranslationKey,
  params?: Record<string, string | number>,
) => string;

// The core writes its messages in English (an AI reads them too). Here every
// sentence it can write is matched and shown with the language of the app.
// `names` gives a name to each number or word the pattern catches; `short` is
// the same sentence without "Line N:", for the line itself.
const sentences: {
  match: RegExp;
  names: string[];
  full: TranslationKey;
  short?: TranslationKey;
}[] = [
  {
    match:
      /^line (\d+) \(row (\d+) of frame (\d+)\): (\d+) columns, expected (\d+)$/,
    names: ["line", "row", "frame", "found", "expected"],
    full: "grid.error.rowLength",
    short: "grid.error.rowLength.short",
  },
  {
    match:
      /^line (\d+) \(row (\d+), column (\d+)\): key "(.+)" is not in palette (.+)$/,
    names: ["line", "row", "column", "key", "palette"],
    full: "grid.error.unknownKey",
    short: "grid.error.unknownKey.short",
  },
  {
    match: /^frame (\d+) has (\d+) rows, expected (\d+)$/,
    names: ["frame", "found", "expected"],
    full: "grid.error.rowCount",
  },
  {
    match: /^frame (\d+) has 0 rows, expected at least 1$/,
    names: ["frame"],
    full: "grid.error.noRows",
  },
  {
    match: /^frame (\d+) does not exist; the sprite has (\d+) frames$/,
    names: ["frame", "count"],
    full: "grid.error.noFrame",
  },
  {
    match: /^grid is (\d+)x(\d+); maximum is (\d+)x\d+$/,
    names: ["width", "height", "max"],
    full: "grid.error.tooBig",
  },
  {
    match:
      /^region at \((\d+), (\d+)\) of size (\d+)x(\d+) does not fit in (\d+)x(\d+)$/,
    names: ["x", "y", "width", "height", "spriteWidth", "spriteHeight"],
    full: "grid.error.region",
  },
  {
    match: /^line (\d+): frame (\d+) appears twice$/,
    names: ["line", "frame"],
    full: "grid.error.frameTwice",
    short: "grid.error.frameTwice.short",
  },
  {
    match: /^line (\d+): key (.+) is in the legend twice$/,
    names: ["line", "key"],
    full: "grid.error.keyTwice",
    short: "grid.error.keyTwice.short",
  },
  {
    match: /^line (\d+): "palette:" needs a name$/,
    names: ["line"],
    full: "grid.error.paletteName",
    short: "grid.error.paletteName.short",
  },
  {
    match: /^line (\d+): expected "palette:", "legend" or "frame", found (.+)$/,
    names: ["line", "found"],
    full: "grid.error.expectedSection",
    short: "grid.error.expectedSection.short",
  },
  {
    match: /^line (\d+): expected "frame <number>", found (.+)$/,
    names: ["line", "found"],
    full: "grid.error.expectedFrame",
    short: "grid.error.expectedFrame.short",
  },
  {
    match: /^line (\d+): (.+) is not a key, a key is one printable character$/,
    names: ["line", "key"],
    full: "grid.error.notKey",
    short: "grid.error.notKey.short",
  },
  {
    match: /^line (\d+): (.+) is not a color, expected #RRGGBB$/,
    names: ["line", "word"],
    full: "grid.error.notColor",
    short: "grid.error.notColor.short",
  },
  {
    match: /^line (\d+): the legend line for (.+) needs a name$/,
    names: ["line", "key"],
    full: "grid.error.legendName",
    short: "grid.error.legendName.short",
  },
  {
    match:
      /^line (\d+): the legend adds the color (.+) but palette (.+) already has (\d+) colors$/,
    names: ["line", "key", "palette", "count"],
    full: "grid.error.paletteFull",
    short: "grid.error.paletteFull.short",
  },
];

function translate(t: Translate, message: string, short: boolean) {
  for (const sentence of sentences) {
    const found = sentence.match.exec(message);
    if (!found) continue;
    const params: Record<string, string> = {};
    sentence.names.forEach((name, i) => (params[name] = found[i + 1]));
    return t((short && sentence.short) || sentence.full, params);
  }
  // A sentence nobody knows: the text of the core, as it is.
  return short ? message.replace(/^line \d+( \([^)]*\))?: /, "") : message;
}

// The whole message, for the status line.
export const gridMessage = (t: Translate, error: GridError) =>
  translate(t, error.message, false);

// The message without "Line N:", for the line that has the problem.
export const shortGridMessage = (t: Translate, error: GridError) =>
  translate(t, error.message, true);
