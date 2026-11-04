package runtime

import (
	"errors"
	"os"
	"path/filepath"
	"strings"
	"testing"
)

const goodToken = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"

func write(t *testing.T, dir, name, text string) {
	t.Helper()
	if err := os.WriteFile(filepath.Join(dir, name), []byte(text), 0o600); err != nil {
		t.Fatal(err)
	}
}

// Checks that the data folder is ".sprixel" inside the home folder, built
// with the path separator of the system (so it also works on Windows).
func TestDirIsDotSprixelInHome(t *testing.T) {
	home := filepath.Join("some", "home")
	want := filepath.Join(home, ".sprixel")
	if got := Dir(home); got != want {
		t.Errorf("Dir = %q, want %q", got, want)
	}
}

// Checks that Home uses the home variable of the system (HOME on Linux and
// macOS, USERPROFILE on Windows), as the app does.
func TestHomeReadsTheHomeVariable(t *testing.T) {
	want := t.TempDir()
	t.Setenv("HOME", want)
	t.Setenv("USERPROFILE", want)
	got, err := Home()
	if err != nil || got != want {
		t.Errorf("Home = %q, %v; want %q", got, err, want)
	}
}

// Checks that runtime.json is read: the port and the process id. The
// version is optional (older apps do not write it).
func TestReadInfoReadsPortPidAndVersion(t *testing.T) {
	dir := t.TempDir()
	write(t, dir, "runtime.json", `{"port": 4567, "pid": 99, "version": "1.2.3"}`)

	info, err := ReadInfo(dir)
	if err != nil {
		t.Fatal(err)
	}
	if info.Port != 4567 || info.PID != 99 || info.Version != "1.2.3" {
		t.Errorf("info = %+v", info)
	}
	if info.Addr() != "127.0.0.1:4567" {
		t.Errorf("Addr = %q", info.Addr())
	}

	write(t, dir, "runtime.json", `{"port": 4567, "pid": 99}`)
	if info, err = ReadInfo(dir); err != nil || info.Version != "" {
		t.Errorf("without version: %+v, %v", info, err)
	}
}

// Checks that a missing runtime.json means "the app is not running", and
// that a broken file or a bad port is a different, explained error.
func TestReadInfoErrors(t *testing.T) {
	dir := t.TempDir()
	if _, err := ReadInfo(dir); !errors.Is(err, ErrNotRunning) {
		t.Errorf("missing file: err = %v, want ErrNotRunning", err)
	}
	for _, text := range []string{"", "not json", `{"port": 0}`, `{"port": 70000}`, `{"pid": 1}`} {
		write(t, dir, "runtime.json", text)
		_, err := ReadInfo(dir)
		if err == nil || errors.Is(err, ErrNotRunning) {
			t.Errorf("%q: err = %v, want a damaged-file error", text, err)
		}
	}
}

// Checks that the token is read and trimmed, and that a missing or empty
// file gives ErrNoToken.
func TestReadToken(t *testing.T) {
	dir := t.TempDir()
	if _, err := ReadToken(dir); !errors.Is(err, ErrNoToken) {
		t.Errorf("missing: err = %v, want ErrNoToken", err)
	}
	write(t, dir, "token", "\n")
	if _, err := ReadToken(dir); !errors.Is(err, ErrNoToken) {
		t.Errorf("empty: err = %v, want ErrNoToken", err)
	}
	write(t, dir, "token", goodToken+"\n")
	if got, err := ReadToken(dir); err != nil || got != goodToken {
		t.Errorf("ReadToken = %q, %v", got, err)
	}
}

// Checks that an error about the token never contains the token itself.
func TestTokenErrorsDoNotLeakTheToken(t *testing.T) {
	dir := t.TempDir()
	write(t, dir, "token", goodToken)
	if err := os.Chmod(filepath.Join(dir, "token"), 0); err != nil {
		t.Skip("cannot change the mode on this system")
	}
	if _, err := ReadToken(dir); err != nil && strings.Contains(err.Error(), goodToken) {
		t.Error("the error contains the token")
	}
}

// Checks the permission test: a token that group or others can read is
// "open". On Windows the check is not possible and it says so.
func TestTokenIsOpen(t *testing.T) {
	dir := t.TempDir()
	write(t, dir, "token", goodToken)
	open, known, err := TokenIsOpen(dir)
	if err != nil {
		t.Fatal(err)
	}
	if !known {
		t.Skip("permissions cannot be checked on this system")
	}
	if open {
		t.Error("a 0600 file must not be open")
	}
	if err := os.Chmod(filepath.Join(dir, "token"), 0o644); err != nil {
		t.Fatal(err)
	}
	if open, _, _ = TokenIsOpen(dir); !open {
		t.Error("a 0644 file must be open")
	}
}
