import process from "node:process";
import { execFileSync, spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { appName, cliName } from "./release-files.mjs";
import { stage } from "./stage-release.mjs";

// A fake repository with the files that `tauri build` and `build:cli` write.
let root, out;
beforeEach(() => {
  root = fs.mkdtempSync(path.join(os.tmpdir(), "sprixel-stage-"));
  out = path.join(root, "out");
});
afterEach(() => fs.rmSync(root, { recursive: true, force: true }));

const write = (file, text = "x") => {
  const full = path.join(root, file);
  fs.mkdirSync(path.dirname(full), { recursive: true });
  fs.writeFileSync(full, text);
};

describe("stage-release", () => {
  it("names the Linux files after the release", () => {
    write("bin/sprixel", "cli");
    write(
      "src-tauri/target/release/bundle/appimage/Sprixel_0.1.0_amd64.AppImage",
      "app",
    );

    stage(root, { os: "linux", arch: "x64" }, out);

    expect(fs.readFileSync(path.join(out, "sprixel-linux-x64"), "utf8")).toBe(
      "cli",
    );
    expect(
      fs.readFileSync(path.join(out, "Sprixel-linux-x64.AppImage"), "utf8"),
    ).toBe("app");
    expect(
      fs.statSync(path.join(out, "sprixel-linux-x64")).mode & 0o100,
    ).toBeTruthy();
  });

  it("packs Sprixel.app into a tar.gz on macOS", () => {
    write("bin/sprixel");
    write(
      "src-tauri/target/release/bundle/macos/Sprixel.app/Contents/MacOS/sprixel",
      "app",
    );

    stage(root, { os: "macos", arch: "arm64" }, out);

    const name = appName({ os: "macos", arch: "arm64" });
    const list = execFileSync("tar", ["-tzf", path.join(out, name)], {
      encoding: "utf8",
    });
    expect(name).toBe("Sprixel-macos-arm64.app.tar.gz");
    expect(list).toContain("Sprixel.app/Contents/MacOS/sprixel");
    expect(fs.existsSync(path.join(out, "sprixel-macos-arm64"))).toBe(true);
  });

  it.skipIf(process.platform === "win32")(
    "packs Sprixel.exe into a zip on Windows",
    () => {
      write("bin/sprixel.exe", "cli");
      write("src-tauri/target/release/sprixel.exe", "app");

      stage(root, { os: "windows", arch: "x64" }, out);

      const zip = path.join(out, "Sprixel-windows-x64.zip");
      expect(
        execFileSync("unzip", ["-l", zip], { encoding: "utf8" }),
      ).toContain("Sprixel.exe");
      expect(
        fs.readFileSync(
          path.join(out, cliName({ os: "windows", arch: "x64" })),
          "utf8",
        ),
      ).toBe("cli");
      expect(fs.readdirSync(out).sort()).toEqual([
        "Sprixel-windows-x64.zip",
        "sprixel-windows-x64.exe",
      ]);
    },
  );

  it("fails with a clear message when the build output is missing", () => {
    write("bin/sprixel");

    expect(() => stage(root, { os: "linux", arch: "x64" }, out)).toThrow(
      /AppImage was not found/,
    );
  });

  it("fails when the command was not built", () => {
    expect(() => stage(root, { os: "linux", arch: "x64" }, out)).toThrow(
      /build:cli/,
    );
  });

  it("the command line rejects an unknown system", () => {
    const r = spawnSync(
      "node",
      [path.resolve("scripts/stage-release.mjs"), "plan9", "x64", out],
      { encoding: "utf8" },
    );

    expect(r.status).toBe(2);
  });
});
