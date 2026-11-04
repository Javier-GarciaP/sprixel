package launcher

import (
	"errors"
	"os"
	"path/filepath"
	"testing"
	"time"
)

func env(goos string, vars map[string]string) Environment {
	return Environment{GOOS: goos, Getenv: func(k string) string { return vars[k] }}
}

// Checks the "No desktop session" rule: Linux without X11 or Wayland has no
// screen; with either of them, or on Windows and macOS, there is one.
func TestCheckDisplay(t *testing.T) {
	cases := []struct {
		goos string
		vars map[string]string
		fail bool
	}{
		{"linux", nil, true},
		{"linux", map[string]string{"DISPLAY": ":0"}, false},
		{"linux", map[string]string{"WAYLAND_DISPLAY": "wayland-0"}, false},
		{"windows", nil, false},
		{"darwin", nil, false},
	}
	for _, c := range cases {
		err := CheckDisplay(env(c.goos, c.vars))
		if c.fail != errors.Is(err, ErrNoDisplay) {
			t.Errorf("%s %v: err = %v", c.goos, c.vars, err)
		}
	}
}

func touch(t *testing.T, path string) {
	t.Helper()
	if err := os.MkdirAll(filepath.Dir(path), 0o755); err != nil {
		t.Fatal(err)
	}
	if err := os.WriteFile(path, []byte("x"), 0o755); err != nil {
		t.Fatal(err)
	}
}

// Checks that SPRIXEL_APP wins over every other place, and that a wrong path
// is "not installed" instead of silently using another one.
func TestFindUsesSprixelAppFirst(t *testing.T) {
	dir := t.TempDir()
	custom := filepath.Join(dir, "custom")
	touch(t, custom)
	touch(t, filepath.Join(dir, ".local", "bin", "sprixel-app"))

	e := Environment{GOOS: "linux", Home: dir, Getenv: func(k string) string {
		if k == "SPRIXEL_APP" {
			return custom
		}
		return ""
	}}
	if got, err := Find(e); err != nil || got != custom {
		t.Errorf("Find = %q, %v", got, err)
	}
	e.Getenv = func(k string) string {
		if k == "SPRIXEL_APP" {
			return filepath.Join(dir, "missing")
		}
		return ""
	}
	if _, err := Find(e); !errors.Is(err, ErrNotInstalled) {
		t.Errorf("err = %v, want ErrNotInstalled", err)
	}
}

// Checks the known places: next to the command, and in the home folder.
func TestFindKnownPlaces(t *testing.T) {
	home := t.TempDir()
	bin := filepath.Join(t.TempDir(), "bin")
	e := Environment{GOOS: "linux", Home: home, Exe: filepath.Join(bin, "sprixel"), Getenv: func(string) string { return "" }}
	if _, err := Find(e); !errors.Is(err, ErrNotInstalled) {
		t.Fatalf("empty computer: err = %v", err)
	}

	local := filepath.Join(home, ".local", "bin", "sprixel-app")
	touch(t, local)
	if got, _ := Find(e); got != local {
		t.Errorf("home place: got %q", got)
	}

	beside := filepath.Join(bin, "sprixel-app")
	touch(t, beside)
	if got, _ := Find(e); got != beside {
		t.Errorf("next to the command: got %q", got)
	}
}

// Checks that the app of a source build (src-tauri/target/release) is found
// from bin/sprixel.
func TestFindSourceBuild(t *testing.T) {
	root := t.TempDir()
	app := filepath.Join(root, "src-tauri", "target", "release", "sprixel")
	touch(t, app)
	e := Environment{GOOS: "linux", Home: t.TempDir(), Exe: filepath.Join(root, "bin", "sprixel"), Getenv: func(string) string { return "" }}

	got, err := Find(e)
	if err != nil {
		t.Fatal(err)
	}
	if want, _ := filepath.Abs(app); filepath.Clean(got) != want {
		t.Errorf("Find = %q, want %q", got, want)
	}
}

// The helper process: the tests start this same test binary as a fake app.
func TestHelperApp(t *testing.T) {
	switch os.Getenv("SPRIXEL_TEST_APP") {
	case "":
		t.Skip("only runs as a child process")
	case "ok":
		os.Exit(0)
	case "display":
		os.Stderr.WriteString("Gtk-WARNING: cannot open display: \n")
		os.Exit(1)
	case "crash":
		os.Stderr.WriteString("something else\n")
		os.Exit(3)
	}
}

func wait(t *testing.T, mode string) error {
	t.Helper()
	t.Setenv("SPRIXEL_TEST_APP", mode)
	exited, err := Start(os.Args[0], "-test.run=^TestHelperApp$")
	if err != nil {
		t.Fatal(err)
	}
	select {
	case err := <-exited:
		return err
	case <-time.After(30 * time.Second):
		t.Fatal("the process did not end")
		return nil
	}
}

// Checks Start: a normal exit gives nil, an exit that mentions the display
// gives ErrNoDisplay, and another failure gives an explained error.
func TestStartReportsHowTheAppEnded(t *testing.T) {
	if err := wait(t, "ok"); err != nil {
		t.Errorf("ok: %v", err)
	}
	if err := wait(t, "display"); !errors.Is(err, ErrNoDisplay) {
		t.Errorf("display: %v", err)
	}
	if err := wait(t, "crash"); err == nil || errors.Is(err, ErrNoDisplay) {
		t.Errorf("crash: %v", err)
	}
}

// Checks that a program that does not exist is an error from Start itself.
func TestStartMissingProgram(t *testing.T) {
	if _, err := Start(filepath.Join(t.TempDir(), "nothing")); err == nil {
		t.Error("expected an error")
	}
}

// A source build has a release and a debug app. The one built last is the one
// the person is working with: an old release build must not win just because
// release comes first in the list.
func TestFindSourceBuildPrefersTheNewestApp(t *testing.T) {
	root := t.TempDir()
	release := filepath.Join(root, "src-tauri", "target", "release", "sprixel")
	debug := filepath.Join(root, "src-tauri", "target", "debug", "sprixel")
	touch(t, release)
	touch(t, debug)
	old := time.Now().Add(-48 * time.Hour)
	if err := os.Chtimes(release, old, old); err != nil {
		t.Fatal(err)
	}
	e := Environment{GOOS: "linux", Home: t.TempDir(), Exe: filepath.Join(root, "bin", "sprixel"), Getenv: func(string) string { return "" }}

	got, err := Find(e)
	if err != nil {
		t.Fatal(err)
	}
	if want, _ := filepath.Abs(debug); filepath.Clean(got) != want {
		t.Errorf("Find = %q, want the newer debug build %q", got, want)
	}
}
