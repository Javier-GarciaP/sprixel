// Package release downloads a Sprixel release and installs it. It is the Go
// twin of install.sh and install.ps1: same file names, same SHA-256 check,
// same folders. `sprixel update` uses it.
package release

import (
	"crypto/sha256"
	"encoding/hex"
	"errors"
	"fmt"
	"io"
	"net/http"
	"net/url"
	"path/filepath"
	"strconv"
	"strings"
	"time"
)

// Source is the home of the project. Every download starts here.
const Source = "https://github.com/Javier-GarciaP/sprixel"

// BaseEnv is the name of the variable that replaces the download folder. It is
// meant for tests (for example file:///tmp/release) and is documented in
// docs/usage.md.
const BaseEnv = "SPRIXEL_BASE_URL"

// Names of the files that are not different for each system.
const (
	ChecksumsFile = "checksums.txt"
	VersionFile   = "VERSION"
	LicenseFile   = "LICENSE"
	NoticesFile   = "THIRD-PARTY.md"
)

// ErrUnsupported means there is no release for this system.
var ErrUnsupported = errors.New("this system is not supported. Supported: Linux x64, macOS Apple silicon, macOS Intel and Windows x64")

// Target is a system we publish files for.
type Target struct {
	OS   string // linux, macos or windows
	Arch string // x64 or arm64
}

// TargetFor maps Go's GOOS and GOARCH to a Target.
func TargetFor(goos, goarch string) (Target, error) {
	t := Target{}
	switch goos {
	case "linux", "windows":
		t.OS = goos
	case "darwin":
		t.OS = "macos"
	}
	switch goarch {
	case "amd64":
		t.Arch = "x64"
	case "arm64":
		t.Arch = "arm64"
	}
	switch t {
	case Target{"linux", "x64"}, Target{"macos", "x64"}, Target{"macos", "arm64"}, Target{"windows", "x64"}:
		return t, nil
	}
	return Target{}, ErrUnsupported
}

// CLIName is the file name of the sprixel command for this target.
func (t Target) CLIName() string {
	if t.OS == "windows" {
		return "sprixel-windows-x64.exe"
	}
	return "sprixel-" + t.OS + "-" + t.Arch
}

// AppName is the file name of the app bundle for this target.
func (t Target) AppName() string {
	switch t.OS {
	case "linux":
		return "Sprixel-linux-" + t.Arch + ".AppImage"
	case "macos":
		return "Sprixel-macos-" + t.Arch + ".app.tar.gz"
	}
	return "Sprixel-windows-x64.zip"
}

// Files lists every file an install needs, in the order they are checked.
func (t Target) Files() []string {
	return []string{VersionFile, t.CLIName(), t.AppName(), LicenseFile, NoticesFile}
}

// BaseURL is the folder the files are downloaded from. A value in
// SPRIXEL_BASE_URL wins; then a version; then the latest release.
func BaseURL(getenv func(string) string, version string) string {
	if base := getenv(BaseEnv); base != "" {
		return strings.TrimRight(base, "/")
	}
	if version != "" {
		return Source + "/releases/download/v" + strings.TrimPrefix(version, "v")
	}
	return Source + "/releases/latest/download"
}

// Get downloads one file of the release. http, https and file URLs work; the
// last one is for tests.
func Get(base, name string) ([]byte, error) {
	client := &http.Client{Timeout: 5 * time.Minute, Transport: transport()}
	resp, err := client.Get(base + "/" + name)
	if err != nil {
		return nil, fmt.Errorf("cannot download %s/%s (are you online?)", base, name)
	}
	defer resp.Body.Close()
	if resp.StatusCode != http.StatusOK {
		return nil, fmt.Errorf("cannot download %s/%s: %s", base, name, resp.Status)
	}
	data, err := io.ReadAll(resp.Body)
	if err != nil {
		return nil, fmt.Errorf("the download of %s was interrupted: %w", name, err)
	}
	return data, nil
}

func transport() http.RoundTripper {
	t := http.DefaultTransport.(*http.Transport).Clone()
	t.RegisterProtocol("file", http.NewFileTransport(http.Dir("/")))
	return t
}

// FileURL turns a local folder into a file:// URL, the form BaseEnv takes.
func FileURL(dir string) string {
	return (&url.URL{Scheme: "file", Path: filepath.ToSlash(dir)}).String()
}

// LatestVersion reads the VERSION file of a release.
func LatestVersion(base string) (string, error) {
	data, err := Get(base, VersionFile)
	if err != nil {
		return "", err
	}
	v := strings.TrimSpace(string(data))
	if !validVersion(v) {
		return "", fmt.Errorf("the VERSION file of the release is not a version: %q", v)
	}
	return v, nil
}

// ParseChecksums reads a checksums.txt (`sha256sum` format: hash, two spaces,
// file name) into a map from file name to hash.
func ParseChecksums(data []byte) map[string]string {
	sums := map[string]string{}
	for _, line := range strings.Split(string(data), "\n") {
		fields := strings.Fields(line)
		if len(fields) == 2 {
			sums[strings.TrimPrefix(fields[1], "*")] = strings.ToLower(fields[0])
		}
	}
	return sums
}

// Download fetches the files of a target and checks every SHA-256. It returns
// the files only when all of them are right, so nothing is installed from a
// damaged download.
func Download(base string, t Target) (map[string][]byte, error) {
	raw, err := Get(base, ChecksumsFile)
	if err != nil {
		return nil, err
	}
	sums := ParseChecksums(raw)
	files := map[string][]byte{}
	for _, name := range t.Files() {
		data, err := Get(base, name)
		if err != nil {
			return nil, err
		}
		want, ok := sums[name]
		if !ok {
			return nil, fmt.Errorf("checksums.txt has no checksum for %s. Nothing was installed", name)
		}
		sum := sha256.Sum256(data)
		if hex.EncodeToString(sum[:]) != want {
			return nil, fmt.Errorf("the checksum of %s does not match. The download is damaged or was changed. Nothing was installed", name)
		}
		files[name] = data
	}
	return files, nil
}

func validVersion(v string) bool {
	_, _, ok := parseVersion(v)
	return ok
}

// parseVersion splits "1.2.3-rc.1" into [1 2 3] and "rc.1".
func parseVersion(v string) (nums [3]int, pre string, ok bool) {
	core, pre, _ := strings.Cut(strings.TrimPrefix(v, "v"), "-")
	parts := strings.Split(core, ".")
	if len(parts) != 3 {
		return nums, "", false
	}
	for i, p := range parts {
		n, err := strconv.Atoi(p)
		if err != nil || n < 0 {
			return nums, "", false
		}
		nums[i] = n
	}
	return nums, pre, true
}

// Newer reports whether version a is newer than version b. A pre-release is
// older than the same version without the suffix. Versions that cannot be
// read are never "newer".
func Newer(a, b string) bool {
	an, apre, aok := parseVersion(a)
	bn, bpre, bok := parseVersion(b)
	if !aok || !bok {
		return false
	}
	for i := range an {
		if an[i] != bn[i] {
			return an[i] > bn[i]
		}
	}
	switch {
	case apre == bpre:
		return false
	case apre == "":
		return true
	case bpre == "":
		return false
	}
	return apre > bpre
}

// Layout is where an installation puts its files.
type Layout struct {
	// Command is the sprixel command, App the app (a file, or the .app folder
	// on macOS) and Share the folder with the license and the notices.
	Command, App, Share string
	// Profiles are the files where the Unix installer may add the PATH line.
	Profiles []string
}

// LayoutFor gives the folders of the installers for a system. localAppData is
// only used on Windows.
func LayoutFor(goos, home, localAppData string) Layout {
	share := filepath.Join(home, ".local", "share", "sprixel")
	bin := filepath.Join(home, ".local", "bin", "sprixel")
	profiles := []string{filepath.Join(home, ".profile"), filepath.Join(home, ".zprofile"), filepath.Join(home, ".bash_profile")}
	switch goos {
	case "windows":
		root := filepath.Join(localAppData, "Sprixel")
		return Layout{Command: filepath.Join(root, "bin", "sprixel.exe"), App: filepath.Join(root, "Sprixel.exe"), Share: root}
	case "darwin":
		return Layout{Command: bin, App: filepath.Join(home, "Applications", "Sprixel.app"), Share: share, Profiles: profiles}
	}
	return Layout{Command: bin, App: filepath.Join(home, ".local", "bin", "sprixel-app"), Share: share, Profiles: profiles}
}
