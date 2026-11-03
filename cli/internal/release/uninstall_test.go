package release_test

import (
	"os"
	"path/filepath"
	"testing"

	"github.com/Javier-GarciaP/sprixel/cli/internal/release"
)

func TestRemoveProfileLinesKeepsEverythingElse(t *testing.T) {
	path := filepath.Join(t.TempDir(), ".profile")
	before := "export EDITOR=vim\n"
	added := "\n" + release.ProfileMarker + "\nexport PATH=\"$HOME/.local/bin:$PATH\"\n"
	after := "alias ll='ls -l'\n"
	os.WriteFile(path, []byte(before+added+after), 0o640)

	changed, err := release.RemoveProfileLines(path)

	if err != nil || !changed {
		t.Fatalf("changed = %v, err = %v", changed, err)
	}
	if got := read(t, path); got != before+after {
		t.Errorf("profile = %q", got)
	}
	if info, _ := os.Stat(path); info.Mode().Perm() != 0o640 {
		t.Errorf("mode = %v", info.Mode())
	}
}

func TestRemoveProfileLinesIgnoresOtherFiles(t *testing.T) {
	dir := t.TempDir()
	plain := filepath.Join(dir, ".profile")
	os.WriteFile(plain, []byte("export PATH=\"$HOME/bin:$PATH\"\n"), 0o644)

	if changed, err := release.RemoveProfileLines(plain); changed || err != nil {
		t.Errorf("a profile without the marker changed: %v %v", changed, err)
	}
	if changed, err := release.RemoveProfileLines(filepath.Join(dir, "missing")); changed || err != nil {
		t.Errorf("a missing file: %v %v", changed, err)
	}
	if read(t, plain) != "export PATH=\"$HOME/bin:$PATH\"\n" {
		t.Error("the user's own PATH line was removed")
	}
}

// Uninstall removes the app, the command and the notices, but a file of the
// user that happens to be in the share folder keeps the folder alive.
func TestUninstallKeepsUnknownFilesInTheShareFolder(t *testing.T) {
	home := t.TempDir()
	l := release.LayoutFor("linux", home, "")
	if err := release.Apply("linux", linux, l, download(t, "0.2.0", linux)); err != nil {
		t.Fatal(err)
	}
	mine := filepath.Join(l.Share, "my-notes.txt")
	os.WriteFile(mine, []byte("mine"), 0o644)

	removed, err := release.Uninstall("linux", l)

	if err != nil || len(removed) == 0 {
		t.Fatalf("removed = %v, err = %v", removed, err)
	}
	if read(t, mine) != "mine" {
		t.Error("an unknown file was removed")
	}
	if _, err := os.Stat(l.Command); err == nil {
		t.Error("the command is still there")
	}
}

func TestUninstallMacRemovesTheAppFolder(t *testing.T) {
	home := t.TempDir()
	target := release.Target{OS: "macos", Arch: "arm64"}
	l := release.LayoutFor("darwin", home, "")
	if err := release.Apply("darwin", target, l, download(t, "0.2.0", target)); err != nil {
		t.Fatal(err)
	}

	if _, err := release.Uninstall("darwin", l); err != nil {
		t.Fatal(err)
	}
	if _, err := os.Stat(l.App); err == nil {
		t.Error("Sprixel.app is still there")
	}
}

func TestUninstallWithNothingInstalledIsNotAnError(t *testing.T) {
	l := release.LayoutFor("linux", t.TempDir(), "")

	if removed, err := release.Uninstall("linux", l); err != nil || len(removed) != 0 {
		t.Errorf("removed = %v, err = %v", removed, err)
	}
}
