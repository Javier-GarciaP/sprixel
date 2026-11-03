import { describe, expect, it } from "vitest";
import { t, type TranslationKey } from "@/shared/lib/i18n";
import { gridMessage, shortGridMessage } from "./errors";

const es = (key: TranslationKey, params?: Record<string, string | number>) =>
  t("es", key, params);
const en = (key: TranslationKey, params?: Record<string, string | number>) =>
  t("en", key, params);
const error = (message: string) => ({
  code: "x",
  message,
  line: null,
  row: null,
  column: null,
});

// Every message the core writes, with the Spanish text the design shows.
const cases: [string, string, string?][] = [
  [
    "line 22 (row 8 of frame 1): 31 columns, expected 32",
    "Línea 22 (fila 8 del frame 1): tiene 31 columnas y se esperaban 32",
    "tiene 31 columnas y se esperaban 32",
  ],
  [
    'line 9 (row 2, column 4): key "Z" is not in palette main',
    'Línea 9 (fila 2, columna 4): la clave "Z" no está en la paleta main',
    'la clave "Z" no está en la paleta main',
  ],
  [
    "frame 2 has 7 rows, expected 8",
    "El frame 2 tiene 7 filas y se esperaban 8",
  ],
  [
    "frame 2 has 0 rows, expected at least 1",
    "El frame 2 no tiene filas; se espera al menos 1",
  ],
  [
    "frame 5 does not exist; the sprite has 4 frames",
    "El frame 5 no existe; el sprite tiene 4 frames",
  ],
  [
    "grid is 600x10; maximum is 512x512",
    "La matriz mide 600x10; el máximo es 512x512",
  ],
  [
    "region at (3, 4) of size 5x6 does not fit in 8x8",
    "La región en (3, 4) de 5x6 no cabe en 8x8",
  ],
  [
    "line 4: frame 1 appears twice",
    "Línea 4: el frame 1 aparece dos veces",
    "el frame 1 aparece dos veces",
  ],
  [
    'line 6: key "A" is in the legend twice',
    'Línea 6: la clave "A" está dos veces en la leyenda',
    'la clave "A" está dos veces en la leyenda',
  ],
  [
    'line 1: "palette:" needs a name',
    "Línea 1: «palette:» necesita un nombre",
    "«palette:» necesita un nombre",
  ],
  [
    'line 3: expected "palette:", "legend" or "frame", found "hello"',
    'Línea 3: se esperaba «palette:», «legend» o «frame» y se encontró "hello"',
    'se esperaba «palette:», «legend» o «frame» y se encontró "hello"',
  ],
  [
    'line 2: expected "frame <number>", found "frame x"',
    'Línea 2: se esperaba «frame <número>» y se encontró "frame x"',
    'se esperaba «frame <número>» y se encontró "frame x"',
  ],
  [
    'line 5: "ab" is not a key, a key is one printable character',
    'Línea 5: "ab" no es una clave, una clave es un carácter imprimible',
    '"ab" no es una clave, una clave es un carácter imprimible',
  ],
  [
    'line 5: "red" is not a color, expected #RRGGBB',
    'Línea 5: "red" no es un color, se espera #RRGGBB',
    '"red" no es un color, se espera #RRGGBB',
  ],
  [
    "line 5: the legend line for 'A' needs a name",
    "Línea 5: la línea de la leyenda de 'A' necesita un nombre",
    "la línea de la leyenda de 'A' necesita un nombre",
  ],
  [
    "line 7: the legend adds the color 'Q' but palette main already has 93 colors",
    "Línea 7: la leyenda añade el color 'Q' pero la paleta main ya tiene 93 colores",
    "la leyenda añade el color 'Q' pero la paleta main ya tiene 93 colores",
  ],
];

describe("grid error messages", () => {
  // Checks that every message of the core reads in Spanish with its numbers.
  it.each(cases)("translates %s", (message, full) => {
    expect(gridMessage(es, error(message))).toBe(full);
  });

  // Checks the short text of a line: the same sentence without "Línea N:",
  // which is what is shown on the line itself.
  it.each(cases.filter((c) => c[2]))("shortens %s", (message, _full, short) => {
    expect(shortGridMessage(es, error(message))).toBe(short);
  });

  // Checks that English shows the text of the core as it is, so the English
  // sentences are the same as the core's.
  it.each(cases)("keeps %s the same in English", (message) => {
    expect(gridMessage(en, error(message))).toBe(message);
  });

  // Checks that a message nobody knows is shown as the core wrote it, never
  // lost or blank.
  it("keeps an unknown message as it is", () => {
    expect(gridMessage(es, error("something new happened"))).toBe(
      "something new happened",
    );
    expect(shortGridMessage(es, error("line 3: something new"))).toBe(
      "something new",
    );
  });
});
