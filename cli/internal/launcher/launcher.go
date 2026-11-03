// Package launcher finds the Sprixel app and starts it as a separate
// program, so it keeps running after the command that started it ends.
package launcher

import (
	"bytes"
	"errors"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"runtime"
	"strings"
	"sync"
)

// ErrNoDisplay means the computer has no screen for the app (for example a
// server over SSH).
var ErrNoDisplay = errors.New("Sprixel needs a desktop session: no screen was found. Open Sprixel on your desktop, then try again")

// ErrNotInstalled means the app was not found in any known place.
var ErrNotInstalled = errors.New("cannot find the Sprixel app. Install it, or set SPRIXEL_APP to the path of the app")

// Environment is what Find and CheckDisplay read from the computer. Tests
// replace it.
type Environment struct {
	GOOS   string
	Getenv func(string) string
	// Home is the home folder of the user.
	Home string
	// Exe is the path of the running sprixel command.
	Exe string
}

// System is the real environment.
func System(home string) Environment {
	exe, _ := os.Executable()
	return Environment{GOOS: runtime.GOOS, Getenv: os.Getenv, Home: home, Exe: exe}
}

// CheckDisplay fails with ErrNoDisplay on Linux when there is neither an X11
// nor a Wayland display. Windows and macOS always have a desktop.
func CheckDisplay(e Environment) error {
	if e.GOOS != "linux" {
		return nil
	}
	if e.Getenv("DISPLAY") == "" && e.Getenv("WAYLAND_DISPLAY") == "" {
		return ErrNoDisplay
	}
	return nil
}

// candidates are the places where the app may be, best first. The install
// scripts (spec install-and-release) put the app in one of them; SPRIXEL_APP
// always wins, and the build folder of this repository is found for people
// who build from source.
func candidates(e Environment) []string {
	var list []string
	if path := e.Getenv("SPRIXEL_APP"); path != "" {
		return []string{path}
	}
	if e.Exe != "" {
		dir := filepath.Dir(e.Exe)
		switch e.GOOS {
		case "windows":
			list = append(list, filepath.Join(dir, "Sprixel.exe"))
		default:
			list = append(list, filepath.Join(dir, "sprixel-app"))
		}
		// bin/sprixel built from source: the app is in src-tauri/target.
		for _, build := range []string{"release", "debug"} {
			name := "sprixel"
			if e.GOOS == "windows" {
				name = "sprixel.exe"
			}
			list = append(list, filepath.Join(dir, "..", "src-tauri", "target", build, name))
		}
	}
	switch e.GOOS {
	case "darwin":
		list = append(list,
			"/Applications/Sprixel.app",
			filepath.Join(e.Home, "Applications", "Sprixel.app"))
	case "windows":
		for _, base := range []string{e.Getenv("LOCALAPPDATA"), e.Getenv("ProgramFiles")} {
			if base != "" {
				list = append(list, filepath.Join(base, "Sprixel", "Sprixel.exe"))
			}
		}
	default:
		list = append(list,
			filepath.Join(e.Home, ".local", "bin", "sprixel-app"),
			filepath.Join(e.Home, ".local", "share", "sprixel", "Sprixel.AppImage"),
			"/usr/bin/sprixel-app",
			"/opt/Sprixel/sprixel")
	}
	return list
}

// Find returns the path of the app.
func Find(e Environment) (string, error) {
	for _, path := range candidates(e) {
		if info, err := os.Stat(path); err == nil && (!info.IsDir() || strings.HasSuffix(path, ".app")) {
			return path, nil
		}
	}
	return "", ErrNotInstalled
}

// Start runs the app without waiting for it. args go to the app. The result
// channel gets one value when the app process ends: nil when it ended well
// and an error when it failed (with the reason when it can be told, for
// example no display). On macOS an .app is opened with `open`.
func Start(path string, args ...string) (<-chan error, error) {
	name, full := path, args
	if strings.HasSuffix(path, ".app") {
		name, full = "open", append([]string{"-a", path, "--args"}, args...)
		if len(args) == 0 {
			full = []string{"-a", path}
		}
	}
	command := exec.Command(name, full...)
	var output tail
	command.Stdout, command.Stderr = &output, &output
	detach(command)
	if err := command.Start(); err != nil {
		return nil, fmt.Errorf("cannot start the Sprixel app: %w", err)
	}
	exited := make(chan error, 1)
	go func() {
		err := command.Wait()
		exited <- explainExit(err, output.String())
	}()
	return exited, nil
}

// explainExit says why the app ended, using the last lines it printed.
func explainExit(err error, output string) error {
	if err == nil {
		return nil
	}
	if strings.Contains(strings.ToLower(output), "display") {
		return ErrNoDisplay
	}
	return fmt.Errorf("the Sprixel app closed right after starting (%v)", err)
}

// tail keeps the last bytes written to it, so the output of an app that runs
// for a long time does not fill the memory.
type tail struct {
	mu  sync.Mutex
	buf bytes.Buffer
}

const tailSize = 4096

func (t *tail) Write(p []byte) (int, error) {
	t.mu.Lock()
	defer t.mu.Unlock()
	t.buf.Write(p)
	if extra := t.buf.Len() - tailSize; extra > 0 {
		t.buf.Next(extra)
	}
	return len(p), nil
}

func (t *tail) String() string {
	t.mu.Lock()
	defer t.mu.Unlock()
	return t.buf.String()
}
