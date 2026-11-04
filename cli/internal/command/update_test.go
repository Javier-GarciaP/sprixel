package command

import (
	"bytes"
	"encoding/json"
	"os"
	"path/filepath"
	"strings"
	"testing"

	"github.com/Javier-GarciaP/sprixel/cli/internal/release"
	"github.com/Javier-GarciaP/sprixel/cli/internal/testrelease"
	"github.com/Javier-GarciaP/sprixel/cli/internal/version"
)

// installed is a fake installation in a temporary home folder, plus a fake
// release served from a local folder. Nothing touches the real machine.
type installed struct {
	t       *testing.T
	home    string
	release string
	url     string // the address of the folder `release`
	layout  release.Layout
}

// newInstalled installs the fake 0.1.0 release and makes the command believe
// it is version 0.1.0.
func newInstalled(t *testing.T) *installed {
	t.Helper()
	old := version.Version
	version.Version = "0.1.0"
	t.Cleanup(func() { version.Version = old })

	in := &installed{t: t, home: t.TempDir(), release: t.TempDir()}
	in.url = testrelease.Serve(t, in.release)
	in.layout = release.LayoutFor("linux", in.home, "")
	first := t.TempDir()
	testrelease.Make(t, first, "0.1.0", testrelease.Options{})
	target := release.Target{OS: "linux", Arch: "x64"}
	files, err := release.Download(testrelease.Serve(t, first), target)
	if err != nil {
		t.Fatal(err)
	}
	if err := release.Apply("linux", target, in.layout, files); err != nil {
		t.Fatal(err)
	}
	return in
}

func (in *installed) publish(version string, opts testrelease.Options) {
	in.t.Helper()
	testrelease.Make(in.t, in.release, version, opts)
}

func (in *installed) env(stdin string, out, errOut *bytes.Buffer) Env {
	return Env{
		Stdin: strings.NewReader(stdin), Stdout: out, Stderr: errOut,
		Home: in.home, Exe: in.layout.Command,
		GOOS: "linux", GOARCH: "amd64",
		Getenv:   func(n string) string { return map[string]string{release.BaseEnv: in.url}[n] },
		LookPath: func(string) (string, error) { return "", os.ErrNotExist },
		Runner:   func(string, ...string) (string, error) { return "", os.ErrNotExist },
	}
}

func (in *installed) run(args ...string) result {
	return in.runWith("", args...)
}

func (in *installed) runWith(stdin string, args ...string) result {
	in.t.Helper()
	var out, errOut bytes.Buffer
	code := Run(args, in.env(stdin, &out, &errOut))
	return result{code, out.String(), errOut.String()}
}

func (in *installed) commandText() string {
	data, err := os.ReadFile(in.layout.Command)
	if err != nil {
		in.t.Fatal(err)
	}
	return string(data)
}

// Scenario "Up to date": the latest version is installed, so the command says
// so and changes nothing.
func TestUpdateWhenUpToDateChangesNothing(t *testing.T) {
	in := newInstalled(t)
	in.publish("0.1.0", testrelease.Options{})

	r := in.run("update")

	if r.code != ExitOK || !strings.Contains(r.stdout, "0.1.0 is up to date") {
		t.Errorf("code = %d, stdout = %q, stderr = %q", r.code, r.stdout, r.err)
	}
	if in.commandText() != testrelease.CommandText("0.1.0") {
		t.Error("the command was changed")
	}
}

// An older release on the server is never installed over a newer command.
func TestUpdateNeverGoesBackToAnOlderVersion(t *testing.T) {
	in := newInstalled(t)
	version.Version = "0.5.0"
	in.publish("0.4.0", testrelease.Options{})

	r := in.run("update")

	if !strings.Contains(r.stdout, "up to date") || in.commandText() != testrelease.CommandText("0.1.0") {
		t.Errorf("stdout = %q", r.stdout)
	}
}

// The upgrade case: the files are replaced and the message says from and to.
func TestUpdateInstallsANewerVersion(t *testing.T) {
	in := newInstalled(t)
	in.publish("0.2.0", testrelease.Options{})

	r := in.run("update")

	if r.code != ExitOK || !strings.Contains(r.stdout, "Updated Sprixel from 0.1.0 to 0.2.0.") {
		t.Fatalf("code = %d, stdout = %q, stderr = %q", r.code, r.stdout, r.err)
	}
	if in.commandText() != testrelease.CommandText("0.2.0") {
		t.Error("the command was not replaced")
	}
	app, _ := os.ReadFile(in.layout.App)
	if string(app) != testrelease.AppText("0.2.0") {
		t.Errorf("app = %q", app)
	}
}

// With a wrong checksum nothing is installed, and the old files stay.
func TestUpdateStopsOnAWrongChecksum(t *testing.T) {
	in := newInstalled(t)
	in.publish("0.2.0", testrelease.Options{WrongChecksum: "Sprixel-linux-x64.AppImage"})

	r := in.run("update")

	if r.code != ExitFailure || !strings.Contains(r.err, "does not match") {
		t.Errorf("code = %d, stderr = %q", r.code, r.err)
	}
	if in.commandText() != testrelease.CommandText("0.1.0") {
		t.Error("the command was replaced after a failed check")
	}
}

// --check only tells; it does not install.
func TestUpdateCheckOnlyReportsTheNewVersion(t *testing.T) {
	in := newInstalled(t)
	in.publish("0.2.0", testrelease.Options{})

	r := in.run("update", "--check")

	if r.code != ExitOK || !strings.Contains(r.stdout, "0.2.0") || in.commandText() != testrelease.CommandText("0.1.0") {
		t.Errorf("code = %d, stdout = %q", r.code, r.stdout)
	}
}

func TestUpdateJSON(t *testing.T) {
	in := newInstalled(t)
	in.publish("0.2.0", testrelease.Options{})

	r := in.run("update", "--json")

	var got map[string]any
	if err := json.Unmarshal([]byte(r.stdout), &got); err != nil {
		t.Fatalf("stdout is not JSON: %q", r.stdout)
	}
	if got["updated"] != true || got["from"] != "0.1.0" || got["to"] != "0.2.0" {
		t.Errorf("json = %v", got)
	}
}

// A sprixel built from source (not in the install folder) is not replaced.
func TestUpdateRefusesACopyThatIsNotInstalled(t *testing.T) {
	in := newInstalled(t)
	in.publish("0.2.0", testrelease.Options{})
	built := filepath.Join(t.TempDir(), "bin", "sprixel")
	os.MkdirAll(filepath.Dir(built), 0o755)
	os.WriteFile(built, []byte("my build"), 0o755)
	var out, errOut bytes.Buffer
	env := in.env("", &out, &errOut)
	env.Exe = built

	code := Run([]string{"update"}, env)

	if code != ExitFailure || !strings.Contains(errOut.String(), "not put here by the installer") {
		t.Errorf("code = %d, stderr = %q", code, errOut.String())
	}
	if data, _ := os.ReadFile(built); string(data) != "my build" {
		t.Error("the build from source was replaced")
	}
}

func TestUpdateReportsAServerThatCannotBeReached(t *testing.T) {
	in := newInstalled(t) // nothing is published

	r := in.run("update")

	if r.code != ExitFailure || !strings.Contains(r.err, "cannot download") {
		t.Errorf("code = %d, stderr = %q", r.code, r.err)
	}
}

func TestUpdateOnAnUnsupportedSystem(t *testing.T) {
	in := newInstalled(t)
	var out, errOut bytes.Buffer
	env := in.env("", &out, &errOut)
	env.GOARCH = "riscv64"

	if code := Run([]string{"update"}, env); code != ExitFailure || !strings.Contains(errOut.String(), "Supported") {
		t.Errorf("code = %d, stderr = %q", code, errOut.String())
	}
}

func TestUpdateRejectsArguments(t *testing.T) {
	in := newInstalled(t)
	if r := in.run("update", "now"); r.code != ExitUsage {
		t.Errorf("code = %d", r.code)
	}
}

// Update replaces program files only: projects and ~/.sprixel stay.
func TestUpdateKeepsProjectsAndSettings(t *testing.T) {
	in := newInstalled(t)
	in.publish("0.2.0", testrelease.Options{})
	mine := map[string]string{".sprixel/token": "t", ".sprixel/clients.json": "{}", "Sprites/hero/sprixel.json": "{}"}
	for name, text := range mine {
		path := filepath.Join(in.home, name)
		os.MkdirAll(filepath.Dir(path), 0o755)
		os.WriteFile(path, []byte(text), 0o600)
	}

	if r := in.run("update"); r.code != ExitOK {
		t.Fatalf("stderr = %q", r.err)
	}
	for name, text := range mine {
		if data, _ := os.ReadFile(filepath.Join(in.home, name)); string(data) != text {
			t.Errorf("%s changed", name)
		}
	}
}
