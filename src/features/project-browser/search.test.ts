import { describe, expect, it } from "vitest";
import { sampleTree } from "./fixtures";
import { normalize, search } from "./search";

describe("normalize", () => {
  // Checks that case and accents do not matter.
  it("ignores case and accents", () => {
    expect(normalize("Árbol")).toBe("arbol");
    expect(normalize("ÑANDÚ")).toBe("nandu");
    expect(normalize("Goblin")).toBe(normalize("gOBLIN"));
  });
});

describe("search", () => {
  // Checks the spec scenario "Search by name": only sprites and folders whose
  // name contains the text are found, in every folder.
  it("finds the sprites and folders whose name contains the text", () => {
    const found = search(sampleTree(), "gob");

    expect(found.sprites.map((s) => s.name)).toEqual(["goblin"]);
    expect(found.folders).toEqual([]);
    const folders = search(sampleTree(), "ENEMI");
    expect(folders.folders.map((f) => f.name)).toEqual(["Enemigos"]);
    expect(folders.sprites).toEqual([]);
  });

  // Checks that the whole project is searched, not only the open folder.
  it("searches inside every folder", () => {
    const found = search(sampleTree(), "o");

    expect(found.sprites.map((s) => s.name).sort()).toEqual(
      ["goblin", "hero", "orc"].sort(),
    );
  });

  // Checks that a word with an accent is found without typing the accent.
  it("finds accented names without the accent", () => {
    const tree = sampleTree();
    tree.root.sprites.push({ ...tree.root.sprites[0], name: "Árbol" });

    expect(search(tree, "arbol").sprites.map((s) => s.name)).toEqual(["Árbol"]);
    expect(search(tree, "ÁRB").sprites).toHaveLength(1);
  });

  // Checks that an empty or blank text finds nothing (the UI then shows the tree).
  it("finds nothing for an empty text", () => {
    expect(search(sampleTree(), "")).toEqual({ folders: [], sprites: [] });
    expect(search(sampleTree(), "   ")).toEqual({ folders: [], sprites: [] });
  });
});
