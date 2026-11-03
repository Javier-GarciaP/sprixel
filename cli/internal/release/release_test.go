package release_test

import (
	"os"
	"path/filepath"
	"strings"
	"testing"

	"github.com/Javier-GarciaP/sprixel/cli/internal/release"
	"github.com/Javier-GarciaP/sprixel/cli/internal/testrelease"
)

var linux = release.Target{OS: "linux", Arch: "x64"}

// A download whose bytes match checksums.txt is accepted.
func TestDownloadChecksOutAndReturnsTheFiles(t *testing.T) {
	dir := t.TempDir()
	testrelease.Make(t, dir, "0.2.0", testrelease.Options{})

	files, err := release.Download(release.FileURL(dir), linux)
	if err != nil {
		t.Fatal(err)
	}
	if string(files[linux.CLIName()]) != testrelease.CommandText("0.2.0") {
		t.Errorf("wrong command content: %q", files[linux.CLIName()])
	}
}

// Scenario "Corrupted download": one wrong checksum, and nothing is returned.
func TestDownloadStopsOnAWrongChecksum(t *testing.T) {
	dir := t.TempDir()
	testrelease.Make(t, dir, "0.2.0", testrelease.Options{WrongChecksum: linux.AppName()})

	files, err := release.Download(release.FileURL(dir), linux)
	if err == nil || !strings.Contains(err.Error(), "does not match") || !strings.Contains(err.Error(), linux.AppName()) {
		t.Fatalf("err = %v, want a checksum error naming the file", err)
	}
	if files != nil {
		t.Error("files must not be returned after a failed check")
	}
}

// A file that changed after the checksums were made is also refused.
func TestDownloadStopsWhenAFileWasChanged(t *testing.T) {
	dir := t.TempDir()
	testrelease.Make(t, dir, "0.2.0", testrelease.Options{})
	os.WriteFile(filepath.Join(dir, linux.CLIName()), []byte("evil"), 0o644)

	if _, err := release.Download(release.FileURL(dir), linux); err == nil || !strings.Contains(err.Error(), "does not match") {
		t.Fatalf("err = %v", err)
	}
}

// A file with no line in checksums.txt cannot be trusted.
func TestDownloadStopsWhenChecksumsDoNotListAFile(t *testing.T) {
	dir := t.TempDir()
	testrelease.Make(t, dir, "0.2.0", testrelease.Options{})
	sums := filepath.Join(dir, "checksums.txt")
	data, _ := os.ReadFile(sums)
	var keep []string
	for _, line := range strings.Split(string(data), "\n") {
		if !strings.HasSuffix(line, "  LICENSE") {
			keep = append(keep, line)
		}
	}
	os.WriteFile(sums, []byte(strings.Join(keep, "\n")), 0o644)

	if _, err := release.Download(release.FileURL(dir), linux); err == nil || !strings.Contains(err.Error(), "no checksum for LICENSE") {
		t.Fatalf("err = %v", err)
	}
}

func TestDownloadReportsAMissingFile(t *testing.T) {
	dir := t.TempDir()
	testrelease.Make(t, dir, "0.2.0", testrelease.Options{Without: []string{"LICENSE"}})

	if _, err := release.Download(release.FileURL(dir), linux); err == nil || !strings.Contains(err.Error(), "cannot download") {
		t.Fatalf("err = %v", err)
	}
}

func TestTargetFor(t *testing.T) {
	cases := []struct {
		goos, goarch, cli, app string
		ok                     bool
	}{
		{"linux", "amd64", "sprixel-linux-x64", "Sprixel-linux-x64.AppImage", true},
		{"darwin", "arm64", "sprixel-macos-arm64", "Sprixel-macos-arm64.app.tar.gz", true},
		{"darwin", "amd64", "sprixel-macos-x64", "Sprixel-macos-x64.app.tar.gz", true},
		{"windows", "amd64", "sprixel-windows-x64.exe", "Sprixel-windows-x64.zip", true},
		{"windows", "arm64", "", "", false},
		{"linux", "arm64", "", "", false},
		{"freebsd", "amd64", "", "", false},
	}
	for _, c := range cases {
		got, err := release.TargetFor(c.goos, c.goarch)
		if (err == nil) != c.ok {
			t.Errorf("%s/%s: err = %v", c.goos, c.goarch, err)
			continue
		}
		if c.ok && (got.CLIName() != c.cli || got.AppName() != c.app) {
			t.Errorf("%s/%s: got %s and %s", c.goos, c.goarch, got.CLIName(), got.AppName())
		}
	}
}

func TestNewer(t *testing.T) {
	cases := []struct {
		a, b string
		want bool
	}{
		{"0.2.0", "0.1.0", true},
		{"0.1.0", "0.1.0", false},
		{"0.1.0", "0.2.0", false},
		{"0.10.0", "0.9.0", true},
		{"1.0.0", "1.0.0-rc.1", true},
		{"1.0.0-rc.1", "1.0.0", false},
		{"0.3.0-beta.1", "0.3.0-alpha.2", true},
		{"nonsense", "0.1.0", false},
	}
	for _, c := range cases {
		if got := release.Newer(c.a, c.b); got != c.want {
			t.Errorf("Newer(%q, %q) = %v, want %v", c.a, c.b, got, c.want)
		}
	}
}

func TestBaseURL(t *testing.T) {
	none := func(string) string { return "" }
	if got := release.BaseURL(none, ""); got != release.Source+"/releases/latest/download" {
		t.Errorf("latest: %s", got)
	}
	if got := release.BaseURL(none, "v0.2.0"); got != release.Source+"/releases/download/v0.2.0" {
		t.Errorf("version: %s", got)
	}
	env := func(string) string { return "file:///tmp/r/" }
	if got := release.BaseURL(env, "0.2.0"); got != "file:///tmp/r" {
		t.Errorf("override: %s", got)
	}
}
