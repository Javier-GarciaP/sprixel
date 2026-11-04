import fs from "node:fs";
import { describe, expect, it } from "vitest";

const read = (path) => fs.readFileSync(path, "utf8");
const version = JSON.parse(read("package.json")).version;

describe("version", () => {
  it("is a MAJOR.MINOR.PATCH number", () => {
    expect(version).toMatch(/^\d+\.\d+\.\d+(-[0-9A-Za-z.-]+)?$/);
  });

  it("is the same in Cargo.toml", () => {
    const cargo = read("src-tauri/Cargo.toml").match(/^version = "(.+)"$/m);

    expect(cargo?.[1]).toBe(version);
  });

  it("is read by Tauri from package.json", () => {
    const tauri = JSON.parse(read("src-tauri/tauri.conf.json"));

    expect(tauri.version).toBe("../package.json");
  });
});
