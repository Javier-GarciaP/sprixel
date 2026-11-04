import { describe, expect, it } from "vitest";
import { extractNotes } from "./release-notes.mjs";

const changelog = `# Changelog

## [Unreleased]

### Added

- Something not released yet.

## [0.2.0] - 2026-12-01

### Added

- A new tool.

### Fixed

- A bug.

## [0.2.0-beta.1] - 2026-11-20

- A beta note.

## [0.1.0] - 2026-11-01

- First version.
`;

describe("release notes", () => {
  it("returns only the section of the version", () => {
    const notes = extractNotes(changelog, "0.2.0");

    expect(notes).toContain("- A new tool.");
    expect(notes).toContain("### Fixed");
    expect(notes).not.toContain("Something not released yet");
    expect(notes).not.toContain("A beta note");
    expect(notes).not.toContain("## [");
  });

  it("finds a pre-release section by its full version", () => {
    expect(extractNotes(changelog, "0.2.0-beta.1")).toBe("- A beta note.\n");
  });

  it("works for the last section of the file", () => {
    expect(extractNotes(changelog, "0.1.0")).toBe("- First version.\n");
  });

  it("fails when the version has no section", () => {
    expect(() => extractNotes(changelog, "0.3.0")).toThrow(
      /no section "## \[0\.3\.0\]"/,
    );
  });

  it("fails when the section is empty", () => {
    expect(() =>
      extractNotes("## [1.0.0] - 2027-01-01\n\n## [0.9.0]\n- x\n", "1.0.0"),
    ).toThrow(/empty/);
  });

  it("does not match a longer version that starts the same", () => {
    expect(() => extractNotes("## [0.2.0-beta.1]\n- x\n", "0.2.0")).toThrow();
  });
});
