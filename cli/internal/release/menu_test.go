package release

import (
	"os"
	"path/filepath"
	"runtime"
	"strings"
	"testing"
	"time"
)

// Checks the files the app writes for its menu entry on Linux: the .desktop
// file and the icon, both in the user's own folders.
func TestMenuFilesOnLinux(t *testing.T) {
	got := MenuFilesFor("linux", "/home/ana", "", "")
	want := []string{
		filepath.Join("/home/ana", ".local", "share", "applications", "sprixel.desktop"),
		filepath.Join("/home/ana", ".local", "share", "icons", "hicolor", "256x256", "apps", "sprixel.png"),
	}

	if strings.Join(got, "|") != strings.Join(want, "|") {
		t.Errorf("got %v, want %v", got, want)
	}
}

// Checks the Windows files: the Start Menu shortcut and the marker file that
// the app keeps next to its exe.
func TestMenuFilesOnWindows(t *testing.T) {
	got := MenuFilesFor("windows", `C:\Users\ana`, `C:\Users\ana\AppData\Roaming`, `C:\Users\ana\AppData\Local`)

	want := []string{
		filepath.Join(`C:\Users\ana\AppData\Roaming`, "Microsoft", "Windows", "Start Menu", "Programs", "Sprixel.lnk"),
		filepath.Join(`C:\Users\ana\AppData\Local`, "Sprixel", "menu-entry.txt"),
	}
	if strings.Join(got, "|") != strings.Join(want, "|") {
		t.Errorf("got %v, want %v", got, want)
	}
}

// Checks that a missing APPDATA falls back to the usual place under the home.
func TestMenuFilesOnWindowsWithoutAppData(t *testing.T) {
	got := MenuFilesFor("windows", `C:\Users\ana`, "", `C:\Users\ana\AppData\Local`)

	if !strings.HasPrefix(got[0], filepath.Join(`C:\Users\ana`, "AppData", "Roaming")) {
		t.Errorf("shortcut = %q, want it under AppData\\Roaming", got[0])
	}
}

// Checks that macOS has no extra files: the app bundle is the menu entry.
func TestMenuFilesOnMacAreNone(t *testing.T) {
	if got := MenuFilesFor("darwin", "/Users/ana", "", ""); len(got) != 0 {
		t.Errorf("got %v, want none", got)
	}
}

func fakeApp(t *testing.T, body string) (home string, l Layout) {
	t.Helper()
	if runtime.GOOS == "windows" {
		t.Skip("the fake app is a shell script")
	}
	home = t.TempDir()
	l = LayoutFor("linux", home, "")
	if err := os.MkdirAll(filepath.Dir(l.App), 0o755); err != nil {
		t.Fatal(err)
	}
	if err := os.WriteFile(l.App, []byte("#!/bin/sh\n"+body), 0o755); err != nil {
		t.Fatal(err)
	}
	return home, l
}

// Checks that the app is asked with --register-menu, with the home folder of
// the installation, and that the last line it prints comes back.
func TestRegisterMenuAsksTheAppAndReturnsItsLine(t *testing.T) {
	home, l := fakeApp(t, `echo "$@" > "$HOME/args"; echo first; echo "menu entry created: x"`)

	line, err := RegisterMenu("linux", home, l, 10*time.Second)

	if err != nil || line != "menu entry created: x" {
		t.Fatalf("line = %q, err = %v", line, err)
	}
	args, _ := os.ReadFile(filepath.Join(home, "args"))
	if strings.TrimSpace(string(args)) != "--register-menu" {
		t.Errorf("args = %q", args)
	}
}

// Checks that an app that fails is an error with what it said, so the caller
// can print one warning.
func TestRegisterMenuReportsAFailingApp(t *testing.T) {
	home, l := fakeApp(t, `echo "fuse: not found" >&2; exit 1`)

	_, err := RegisterMenu("linux", home, l, 10*time.Second)

	if err == nil || !strings.Contains(err.Error(), "fuse: not found") {
		t.Errorf("err = %v, want it to say what the app printed", err)
	}
}

// Checks that an app that never ends is stopped at the time limit.
func TestRegisterMenuStopsAnAppThatHangs(t *testing.T) {
	home, l := fakeApp(t, `sleep 30`)
	start := time.Now()

	_, err := RegisterMenu("linux", home, l, 300*time.Millisecond)

	if err == nil || time.Since(start) > 10*time.Second {
		t.Errorf("err = %v after %v, want an error well before 30 s", err, time.Since(start))
	}
}

// Checks that on macOS nothing is run: there is no entry to write.
func TestRegisterMenuDoesNothingOnMac(t *testing.T) {
	line, err := RegisterMenu("darwin", t.TempDir(), Layout{App: "/does/not/exist"}, time.Second)

	if err != nil || line != "" {
		t.Errorf("line = %q, err = %v, want nothing", line, err)
	}
}
