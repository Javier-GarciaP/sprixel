// Package testrelease writes a fake Sprixel release into a folder, for tests.
// It has the real file names and a real checksums.txt, but the "programs" are
// tiny text files. Nothing here uses the network.
package testrelease

import (
	"archive/tar"
	"archive/zip"
	"bytes"
	"compress/gzip"
	"crypto/sha256"
	"encoding/hex"
	"fmt"
	"net/http"
	"net/http/httptest"
	"os"
	"path/filepath"
	"sort"
	"strings"
	"testing"

	"github.com/Javier-GarciaP/sprixel/cli/internal/release"
)

// Options changes the fake release.
type Options struct {
	// WrongChecksum is the name of a file whose checksum is made wrong.
	WrongChecksum string
	// Without lists files that are left out of the folder.
	Without []string
	// RegisterMenuFails makes the fake Linux app fail when it is asked to
	// register its menu entry (like an app that cannot start).
	RegisterMenuFails bool
}

// Contents of the fake programs, so tests can tell the versions apart.
func CommandText(version string) string { return "command " + version + "\n" }
func AppText(version string) string     { return appText(version, false) }

// appText is the fake Linux app: a script. Asked to register its menu entry it
// notes the call in $HOME/.fake-app-calls and prints one line (or fails).
func appText(version string, registerMenuFails bool) string {
	register := "echo \"menu entry created: fake\"\n  exit 0"
	if registerMenuFails {
		register = "echo \"fuse: not found\" >&2\n  exit 1"
	}
	return "#!/bin/sh\nif [ \"$1\" = \"--register-menu\" ]; then\n  echo \"$@\" >> \"$HOME/.fake-app-calls\"\n  " + register + "\nfi\necho app " + version + "\n"
}

// Make writes the release of `version` into dir for every target.
func Make(t testing.TB, dir, version string, opts Options) {
	t.Helper()
	files := map[string][]byte{
		release.VersionFile: []byte(version + "\n"),
		release.LicenseFile: []byte("MIT License (fake)\n"),
		release.NoticesFile: []byte("Third-party notices (fake)\n"),
	}
	for _, target := range []release.Target{{OS: "linux", Arch: "x64"}, {OS: "macos", Arch: "arm64"}, {OS: "macos", Arch: "x64"}, {OS: "windows", Arch: "x64"}} {
		files[target.CLIName()] = []byte(CommandText(version))
		switch target.OS {
		case "linux":
			files[target.AppName()] = []byte(appText(version, opts.RegisterMenuFails))
		case "macos":
			files[target.AppName()] = appTarGz(t, version)
		default:
			files[target.AppName()] = appZip(t, version)
		}
	}
	if err := os.MkdirAll(dir, 0o755); err != nil {
		t.Fatal(err)
	}
	var names []string
	for name := range files {
		names = append(names, name)
	}
	sort.Strings(names)
	var sums strings.Builder
	for _, name := range names {
		sum := sha256.Sum256(files[name])
		hash := hex.EncodeToString(sum[:])
		if name == opts.WrongChecksum {
			hash = strings.Repeat("0", 64)
		}
		fmt.Fprintf(&sums, "%s  %s\n", hash, name)
	}
	files[release.ChecksumsFile] = []byte(sums.String())
	for name, data := range files {
		if contains(opts.Without, name) {
			continue
		}
		if err := os.WriteFile(filepath.Join(dir, name), data, 0o644); err != nil {
			t.Fatal(err)
		}
	}
}

func contains(list []string, s string) bool {
	for _, x := range list {
		if x == s {
			return true
		}
	}
	return false
}

func appTarGz(t testing.TB, version string) []byte {
	t.Helper()
	var buf bytes.Buffer
	gz := gzip.NewWriter(&buf)
	tw := tar.NewWriter(gz)
	body := []byte(AppText(version))
	for _, h := range []tar.Header{
		{Name: "Sprixel.app/", Typeflag: tar.TypeDir, Mode: 0o755},
		{Name: "Sprixel.app/Contents/MacOS/", Typeflag: tar.TypeDir, Mode: 0o755},
		{Name: "Sprixel.app/Contents/MacOS/sprixel", Typeflag: tar.TypeReg, Mode: 0o755, Size: int64(len(body))},
	} {
		if err := tw.WriteHeader(&h); err != nil {
			t.Fatal(err)
		}
		if h.Typeflag == tar.TypeReg {
			tw.Write(body)
		}
	}
	tw.Close()
	gz.Close()
	return buf.Bytes()
}

func appZip(t testing.TB, version string) []byte {
	t.Helper()
	var buf bytes.Buffer
	zw := zip.NewWriter(&buf)
	w, err := zw.Create("Sprixel.exe")
	if err != nil {
		t.Fatal(err)
	}
	w.Write([]byte(AppText(version)))
	zw.Close()
	return buf.Bytes()
}

// Serve serves a folder over HTTP on a free local port and returns its address.
// It stops when the test ends. This is the "release server" of the tests.
func Serve(t testing.TB, dir string) string {
	t.Helper()
	server := httptest.NewServer(http.FileServer(http.Dir(dir)))
	t.Cleanup(server.Close)
	return server.URL
}
