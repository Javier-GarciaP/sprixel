import { describe, expect, it } from "vitest";
import { buildColorTable } from "./colors";
import { TILES, tilePicture } from "./tiles";

const colors = buildColorTable([{ key: "R", hex: "#FF0000" }]);
const code = (key: string) => key.charCodeAt(0);

describe("tilePicture", () => {
  // Checks that a 2x1 tile "R." is repeated 3 x 3 times: the picture is 6
  // pixels wide and 3 tall, red on even columns, transparent on odd ones.
  it("repeats the tile three times on each side", () => {
    const picture = tilePicture(
      new Uint8Array([code("R"), code(".")]),
      2,
      1,
      colors,
    );
    expect(TILES).toBe(3);
    expect(picture.length).toBe(6 * 3 * 4);
    for (let y = 0; y < 3; y++) {
      for (let x = 0; x < 6; x++) {
        const o = (y * 6 + x) * 4;
        expect([...picture.slice(o, o + 4)]).toEqual(
          x % 2 === 0 ? [255, 0, 0, 255] : [0, 0, 0, 0],
        );
      }
    }
  });
});
