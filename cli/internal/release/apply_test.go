package release_test

import (
	"archive/tar"
	"bytes"
	"compress/gzip"
	"os"
	"path/filepath"
	"runtime"
	"strings"
	"testing"

	"github.com/Javier-GarciaP/sprixel/cli/internal/release"
	"github.com/Javier-GarciaP/sprixel/cli/internal/testrelease"
)

func read(t *testing.T, path string) string {
	t.Helper()
	data, err := os.ReadFile(path)
	if err != nil {
		t.Fatal(err)
	}
	return string(data)
}

func download(t *testing.T, version string, target release.Target) map[string][]byte {
	t.Helper()
	dir := t.TempDir()
	testrelease.Make(t, dir, version, testrelease.Options{})
	files, err := release.Download(testrelease.Serve(t, dir), target)
	if err != nil {
		t.Fatal(err)
	}
	return files
}

// Applying twice upgrades the files in place (the Linux layout).
func TestApplyLinuxUpgradesInPlace(t *testing.T) {
	home := t.TempDir()
	l := release.LayoutFor("linux", home, "")
	for _, version := range []string{"0.2.0", "0.3.0"} {
		if err := release.Apply("linux", linux, l, download(t, version, linux)); err != nil {
			t.Fatal(err)
		}
	}

	if got := read(t, l.Command); got != testrelease.CommandText("0.3.0") {
		t.Errorf("command = %q", got)
	}
	if got := read(t, l.App); got != testrelease.AppText("0.3.0") {
		t.Errorf("app = %q", got)
	}
	if !strings.Contains(read(t, filepath.Join(l.Share, "LICENSE")), "MIT") {
		t.Error("the license was not installed")
	}
	if info, _ := os.Stat(l.Command); runtime.GOOS != "windows" && info.Mode()&0o100 == 0 {
		t.Error("the command is not executable")
	}
	for _, left := range []string{l.Command + ".new", l.Command + ".old"} {
		if _, err := os.Stat(left); err == nil {
			t.Errorf("%s was left behind", left)
		}
	}
}

// macOS: the old Sprixel.app is replaced by the new one.
func TestApplyMacReplacesTheAppFolder(t *testing.T) {
	home := t.TempDir()
	target := release.Target{OS: "macos", Arch: "arm64"}
	l := release.LayoutFor("darwin", home, "")
	for _, version := range []string{"0.2.0", "0.3.0"} {
		if err := release.Apply("darwin", target, l, download(t, version, target)); err != nil {
			t.Fatal(err)
		}
	}

	exe := filepath.Join(l.App, "Contents", "MacOS", "sprixel")
	if got := read(t, exe); got != testrelease.AppText("0.3.0") {
		t.Errorf("app = %q", got)
	}
	if got := read(t, l.Command); got != testrelease.CommandText("0.3.0") {
		t.Errorf("command = %q", got)
	}
	entries, _ := os.ReadDir(filepath.Dir(l.App))
	if len(entries) != 1 {
		t.Errorf("leftovers next to the app: %v", entries)
	}
}

// Windows: the app comes out of the zip, and the old files are moved aside.
func TestApplyWindowsExtractsTheZip(t *testing.T) {
	local := t.TempDir()
	target := release.Target{OS: "windows", Arch: "x64"}
	l := release.LayoutFor("windows", "", local)
	for _, version := range []string{"0.2.0", "0.3.0"} {
		if err := release.Apply("windows", target, l, download(t, version, target)); err != nil {
			t.Fatal(err)
		}
	}

	if got := read(t, l.App); got != testrelease.AppText("0.3.0") {
		t.Errorf("app = %q", got)
	}
	if filepath.Base(l.Command) != "sprixel.exe" || filepath.Base(filepath.Dir(l.Command)) != "bin" {
		t.Errorf("command path = %s", l.Command)
	}
	if got := read(t, l.Command); got != testrelease.CommandText("0.3.0") {
		t.Errorf("command = %q", got)
	}
}

// A damaged app archive stops before the working command is replaced.
func TestApplyKeepsTheOldCommandWhenTheAppArchiveIsBad(t *testing.T) {
	home := t.TempDir()
	target := release.Target{OS: "macos", Arch: "arm64"}
	l := release.LayoutFor("darwin", home, "")
	if err := release.Apply("darwin", target, l, download(t, "0.2.0", target)); err != nil {
		t.Fatal(err)
	}
	bad := download(t, "0.3.0", target)
	bad[target.AppName()] = []byte("not a tar")

	if err := release.Apply("darwin", target, l, bad); err == nil {
		t.Fatal("want an error")
	}
	if got := read(t, l.Command); got != testrelease.CommandText("0.2.0") {
		t.Errorf("the command was replaced: %q", got)
	}
	if got := read(t, filepath.Join(l.App, "Contents", "MacOS", "sprixel")); got != testrelease.AppText("0.2.0") {
		t.Errorf("the app was replaced: %q", got)
	}
}

// An archive that tries to write outside the app folder is refused.
func TestApplyRefusesAnUnsafeArchive(t *testing.T) {
	var buf bytes.Buffer
	gz := gzip.NewWriter(&buf)
	tw := tar.NewWriter(gz)
	tw.WriteHeader(&tar.Header{Name: "../../escape.txt", Typeflag: tar.TypeReg, Mode: 0o644, Size: 1})
	tw.Write([]byte("x"))
	tw.Close()
	gz.Close()

	home := t.TempDir()
	target := release.Target{OS: "macos", Arch: "arm64"}
	files := download(t, "0.2.0", target)
	files[target.AppName()] = buf.Bytes()

	err := release.Apply("darwin", target, release.LayoutFor("darwin", home, ""), files)
	if err == nil || !strings.Contains(err.Error(), "unsafe") {
		t.Fatalf("err = %v", err)
	}
	if _, statErr := os.Stat(filepath.Join(filepath.Dir(home), "escape.txt")); statErr == nil {
		t.Error("the archive wrote outside the folder")
	}
}

// Apply never writes in ~/.sprixel or in project folders.
func TestApplyLeavesTheUsersFilesAlone(t *testing.T) {
	home := t.TempDir()
	token := filepath.Join(home, ".sprixel", "token")
	project := filepath.Join(home, "Sprites", "hero.txt")
	for _, p := range []string{token, project} {
		os.MkdirAll(filepath.Dir(p), 0o755)
		os.WriteFile(p, []byte("mine"), 0o600)
	}

	if err := release.Apply("linux", linux, release.LayoutFor("linux", home, ""), download(t, "0.2.0", linux)); err != nil {
		t.Fatal(err)
	}
	for _, p := range []string{token, project} {
		if read(t, p) != "mine" {
			t.Errorf("%s changed", p)
		}
	}
}
