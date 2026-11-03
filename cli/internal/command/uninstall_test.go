package command

import (
	"bytes"
	"encoding/json"
	"os"
	"path/filepath"
	"strings"
	"testing"

	"github.com/Javier-GarciaP/sprixel/cli/internal/clients"
	"github.com/Javier-GarciaP/sprixel/cli/internal/release"
)

// The user's own files in the fake home. Uninstall must never remove the
// projects, and the data folder only when it is asked to.
var userFiles = map[string]string{
	"Sprites/hero/sprixel.json":           `{"name":"hero"}`,
	"Sprites/hero/.sprixel/trash/old.txt": "deleted sprite",
	"Documents/notes.txt":                 "notes",
}
var dataFiles = map[string]string{
	".sprixel/token":        "secret",
	".sprixel/clients.json": `{"approved":["claude"]}`,
	".sprixel/runtime.json": "",
}

func (in *installed) writeAll(files map[string]string) {
	in.t.Helper()
	for name, text := range files {
		path := filepath.Join(in.home, name)
		os.MkdirAll(filepath.Dir(path), 0o755)
		if err := os.WriteFile(path, []byte(text), 0o600); err != nil {
			in.t.Fatal(err)
		}
	}
}

func (in *installed) checkKept(files map[string]string) {
	in.t.Helper()
	for name, text := range files {
		data, err := os.ReadFile(filepath.Join(in.home, name))
		if err != nil || string(data) != text {
			in.t.Errorf("%s was changed or removed (err = %v)", name, err)
		}
	}
}

func (in *installed) exists(path string) bool {
	_, err := os.Lstat(path)
	return err == nil
}

// newReady is an installation with projects, data, a profile with the
// installer's line and an OpenCode registration.
func newReady(t *testing.T) *installed {
	in := newInstalled(t)
	in.writeAll(userFiles)
	in.writeAll(dataFiles)
	profile := filepath.Join(in.home, ".profile")
	os.WriteFile(profile, []byte("alias ll='ls -l'\n\n"+release.ProfileMarker+"\nexport PATH=\"$HOME/.local/bin:$PATH\"\n"), 0o644)
	if _, err := (clients.OpenCode{Home: in.home}).Add(in.layout.Command); err != nil {
		t.Fatal(err)
	}
	return in
}

// Scenario "Uninstall keeps projects": the user confirms, the program files
// are removed and the project folders are untouched.
func TestUninstallRemovesTheProgramAndKeepsProjects(t *testing.T) {
	in := newReady(t)

	r := in.runWith("y\nn\n", "uninstall")

	if r.code != ExitOK || !strings.Contains(r.stdout, "Sprixel was removed.") {
		t.Fatalf("code = %d, stdout = %q, stderr = %q", r.code, r.stdout, r.err)
	}
	for _, gone := range []string{in.layout.Command, in.layout.App, filepath.Join(in.layout.Share, "LICENSE"), in.layout.Share} {
		if in.exists(gone) {
			t.Errorf("%s was not removed", gone)
		}
	}
	in.checkKept(userFiles)
	in.checkKept(dataFiles) // the second question was answered "n"
}

func TestUninstallAsksBeforeAndRemovesNothingOnNo(t *testing.T) {
	in := newReady(t)

	for _, answer := range []string{"n\n", "\n", ""} {
		r := in.runWith(answer, "uninstall")
		if r.code != ExitOK || !strings.Contains(r.stdout, "Nothing was removed.") {
			t.Errorf("answer %q: code = %d, stdout = %q", answer, r.code, r.stdout)
		}
	}
	if !in.exists(in.layout.Command) || !in.exists(in.layout.App) {
		t.Error("files were removed without a yes")
	}
	if !strings.Contains(in.runWith("n\n", "uninstall").stdout, "Your projects are not touched") {
		t.Error("the plan does not say that projects stay")
	}
}

// The local data folder is a separate question: yes removes it, projects stay.
func TestUninstallRemovesTheDataFolderOnlyWhenAsked(t *testing.T) {
	in := newReady(t)

	r := in.runWith("y\ny\n", "uninstall")

	if r.code != ExitOK {
		t.Fatalf("stderr = %q", r.err)
	}
	if in.exists(filepath.Join(in.home, ".sprixel")) {
		t.Error("the data folder was not removed")
	}
	in.checkKept(userFiles)
}

// --yes confirms the uninstall but does not answer the data question.
func TestUninstallYesDoesNotRemoveData(t *testing.T) {
	in := newReady(t)

	r := in.runWith("", "uninstall", "--yes")

	if r.code != ExitOK {
		t.Fatalf("stderr = %q", r.err)
	}
	in.checkKept(dataFiles)
	in.checkKept(userFiles)
	if in.exists(in.layout.Command) {
		t.Error("the command is still there")
	}
}

func TestUninstallYesRemoveData(t *testing.T) {
	in := newReady(t)

	r := in.runWith("", "uninstall", "--yes", "--remove-data")

	if r.code != ExitOK || in.exists(filepath.Join(in.home, ".sprixel")) {
		t.Errorf("code = %d, stderr = %q", r.code, r.err)
	}
	in.checkKept(userFiles)
}

// The PATH line is removed; the other lines of the profile stay.
func TestUninstallCleansOnlyTheInstallersProfileLines(t *testing.T) {
	in := newReady(t)

	in.runWith("", "uninstall", "--yes")

	data, _ := os.ReadFile(filepath.Join(in.home, ".profile"))
	if string(data) != "alias ll='ls -l'\n" {
		t.Errorf("profile = %q", data)
	}
}

// The registration in the AI tools is removed.
func TestUninstallRemovesAIRegistrations(t *testing.T) {
	in := newReady(t)
	config := filepath.Join(in.home, ".config", "opencode", "opencode.json")
	if data, _ := os.ReadFile(config); !strings.Contains(string(data), "sprixel") {
		t.Fatal("the test registration is missing")
	}

	r := in.runWith("", "uninstall", "--yes")

	if data, _ := os.ReadFile(config); strings.Contains(string(data), `"sprixel"`) {
		t.Errorf("OpenCode still has Sprixel: %s", data)
	}
	if !strings.Contains(r.stdout, "OpenCode") {
		t.Errorf("stdout = %q", r.stdout)
	}
}

func TestUninstallJSON(t *testing.T) {
	in := newReady(t)

	r := in.runWith("", "uninstall", "--yes", "--json")

	var got struct {
		Uninstalled bool     `json:"uninstalled"`
		DataRemoved bool     `json:"dataRemoved"`
		Removed     []string `json:"removed"`
	}
	if err := json.Unmarshal([]byte(r.stdout), &got); err != nil {
		t.Fatalf("stdout is not JSON: %q", r.stdout)
	}
	if !got.Uninstalled || got.DataRemoved || len(got.Removed) == 0 {
		t.Errorf("json = %+v", got)
	}
}

// JSON mode cannot ask, so it needs --yes.
func TestUninstallJSONWithoutYesIsAUsageError(t *testing.T) {
	in := newReady(t)

	if r := in.run("uninstall", "--json"); r.code != ExitUsage || !in.exists(in.layout.Command) {
		t.Errorf("code = %d", r.code)
	}
}

func TestUninstallRefusesACopyThatIsNotInstalled(t *testing.T) {
	in := newReady(t)
	built := filepath.Join(t.TempDir(), "sprixel")
	os.WriteFile(built, []byte("my build"), 0o755)
	var out, errOut bytes.Buffer
	env := in.env("y\n", &out, &errOut)
	env.Exe = built

	if code := Run([]string{"uninstall", "--yes"}, env); code != ExitFailure {
		t.Errorf("code = %d", code)
	}
	if !in.exists(in.layout.Command) || !in.exists(built) {
		t.Error("something was removed")
	}
}

func TestUninstallRejectsUnknownArguments(t *testing.T) {
	in := newReady(t)
	if r := in.run("uninstall", "--all"); r.code != ExitUsage {
		t.Errorf("code = %d", r.code)
	}
}

// Windows: the PATH is edited through PowerShell, the app is Sprixel.exe and
// the command is in a bin folder of its own. The commands are checked with a
// fake runner (this test does not run PowerShell).
func TestUninstallWindowsLayoutAndPath(t *testing.T) {
	home := t.TempDir()
	local := filepath.Join(home, "AppData", "Local")
	l := release.LayoutFor("windows", home, local)
	for _, p := range []string{l.Command, l.App, filepath.Join(l.Share, "LICENSE")} {
		os.MkdirAll(filepath.Dir(p), 0o755)
		os.WriteFile(p, []byte("x"), 0o755)
	}
	project := filepath.Join(home, "Sprites", "hero.txt")
	os.MkdirAll(filepath.Dir(project), 0o755)
	os.WriteFile(project, []byte("pixels"), 0o644)

	var calls []string
	var out, errOut bytes.Buffer
	code := Run([]string{"uninstall", "--yes"}, Env{
		Stdin: strings.NewReader(""), Stdout: &out, Stderr: &errOut,
		Home: home, Exe: l.Command, GOOS: "windows", GOARCH: "amd64",
		Getenv:   func(n string) string { return map[string]string{"LOCALAPPDATA": local}[n] },
		LookPath: func(string) (string, error) { return "", os.ErrNotExist },
		Runner: func(name string, args ...string) (string, error) {
			calls = append(calls, name+" "+strings.Join(args, " "))
			return "", nil
		},
	})

	if code != ExitOK {
		t.Fatalf("code = %d, stderr = %q", code, errOut.String())
	}
	if len(calls) == 0 || !strings.HasPrefix(calls[len(calls)-1], "powershell") || !strings.Contains(calls[len(calls)-1], filepath.Dir(l.Command)) {
		t.Errorf("PATH was not edited with PowerShell: %v", calls)
	}
	for _, p := range []string{l.Command, l.App, l.Share} {
		if _, err := os.Lstat(p); err == nil {
			t.Errorf("%s was not removed", p)
		}
	}
	if data, _ := os.ReadFile(project); string(data) != "pixels" {
		t.Error("the project changed")
	}
}
