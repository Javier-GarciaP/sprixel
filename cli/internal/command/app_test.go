package command

import (
	"encoding/json"
	"os"
	"path/filepath"
	"regexp"
	"runtime"
	"strconv"
	"strings"
	"testing"
	"time"

	"github.com/Javier-GarciaP/sprixel/cli/internal/launcher"
	"github.com/Javier-GarciaP/sprixel/cli/internal/testapp"
	"github.com/Javier-GarciaP/sprixel/cli/internal/version"
)

// appHome starts a fake app in a new home folder.
func appHome(t *testing.T) (string, *testapp.App) {
	t.Helper()
	home := t.TempDir()
	return home, testapp.Start(t, filepath.Join(home, ".sprixel"))
}

func appRun(t *testing.T, home string, args ...string) result {
	t.Helper()
	return setupIn(t, home, &fakeClaude{}, args...)
}

// Checks the scenario "Status when closed": it says so and the exit code is
// the "not running" code, also with --json (which stays valid JSON).
func TestStatusWhenClosed(t *testing.T) {
	home := t.TempDir()
	r := appRun(t, home, "status")
	if r.code != ExitNotRunning || !strings.Contains(r.stdout, "not running") {
		t.Errorf("code = %d, stdout = %q", r.code, r.stdout)
	}
	r = appRun(t, home, "status", "--json")
	var data map[string]any
	if err := json.Unmarshal([]byte(r.stdout), &data); err != nil || data["running"] != false || r.code != ExitNotRunning {
		t.Errorf("json = %q, %v", r.stdout, err)
	}
}

// Checks "status" for a running app: port and version are shown.
func TestStatusWhenRunning(t *testing.T) {
	home, app := appHome(t)
	r := appRun(t, home, "status")
	if r.code != ExitOK || !strings.Contains(r.stdout, "version "+testapp.Version) || !strings.Contains(r.stdout, "port") {
		t.Errorf("code = %d, stdout = %q", r.code, r.stdout)
	}
	var data map[string]any
	json.Unmarshal([]byte(appRun(t, home, "status", "--json").stdout), &data)
	if data["port"] != float64(app.Port()) || data["version"] != testapp.Version {
		t.Errorf("json = %v", data)
	}
}

// Checks "status" for a runtime.json left behind by a crashed app: the app
// is not running.
func TestStatusIgnoresAStaleRuntimeFile(t *testing.T) {
	home, app := appHome(t)
	app.Stop()
	os.WriteFile(filepath.Join(home, ".sprixel", "runtime.json"), []byte(`{"port":1,"pid":1}`), 0o600)
	if r := appRun(t, home, "status"); r.code != ExitNotRunning {
		t.Errorf("code = %d", r.code)
	}
}

// Checks the scenario "Call a tool": "sprixel tool list_sprites" prints the
// sprites. Against a running app.
func TestToolCallPrintsTheResult(t *testing.T) {
	home, _ := appHome(t)
	r := appRun(t, home, "tool", "list_sprites")
	if r.code != ExitOK || !strings.Contains(r.stdout, "goblin") {
		t.Errorf("code = %d, stdout = %q, stderr = %q", r.code, r.stdout, r.err)
	}
}

// Checks --json for tool: valid JSON only, with the tool's own JSON inside.
func TestToolCallJSON(t *testing.T) {
	home, _ := appHome(t)
	r := appRun(t, home, "tool", "list_sprites", "--json")
	var data struct {
		IsError bool `json:"is_error"`
		Result  struct{ Sprites []map[string]any }
	}
	if err := json.Unmarshal([]byte(r.stdout), &data); err != nil || data.IsError || len(data.Result.Sprites) != 1 {
		t.Errorf("stdout = %q, %v", r.stdout, err)
	}
}

// Checks the argument types: a string argument that looks like a number stays
// text, an integer is a number, a boolean and an array are read as JSON.
func TestToolArgumentsUseTheSchemaTypes(t *testing.T) {
	home, app := appHome(t)
	appRun(t, home, "tool", "get_sprite", "name=123", "frame=2")
	appRun(t, home, "tool", "set_pixels", "dry=true", `pixels=[{"x":1}]`)

	var bodies []string
	for _, call := range app.Calls() {
		if call.Method == "tools/call" {
			bodies = append(bodies, call.Body)
		}
	}
	if len(bodies) != 2 || !strings.Contains(bodies[0], `"name":"123"`) || !strings.Contains(bodies[0], `"frame":2`) {
		t.Errorf("get_sprite body = %v", bodies)
	}
	if !strings.Contains(bodies[1], `"dry":true`) || !strings.Contains(bodies[1], `"pixels":[{"x":1}]`) {
		t.Errorf("set_pixels body = %v", bodies)
	}
}

// Checks the error code of a tool error (exit code 4) and that the message
// goes to stderr.
func TestToolErrorHasItsOwnExitCode(t *testing.T) {
	home, _ := appHome(t)
	r := appRun(t, home, "tool", "get_sprite", "name=ghost")
	if r.code != ExitToolError || !strings.Contains(r.err, "not_found") || r.stdout != "" {
		t.Errorf("code = %d, stdout = %q, stderr = %q", r.code, r.stdout, r.err)
	}
}

// Checks the usage errors of tool: no name, an unknown tool (the message lists
// the real ones), a bad argument, and a number that is not a number.
func TestToolUsageErrors(t *testing.T) {
	home, _ := appHome(t)
	cases := [][]string{{"tool"}, {"tool", "nope"}, {"tool", "get_sprite", "name"}, {"tool", "get_sprite", "name=goblin", "frame=two"}}
	for _, args := range cases {
		if r := appRun(t, home, args...); r.code != ExitUsage {
			t.Errorf("%v: code = %d, stderr = %q", args, r.code, r.err)
		}
	}
	if r := appRun(t, home, "tool", "nope"); !strings.Contains(r.err, "list_sprites") {
		t.Errorf("the message must list the tools: %q", r.err)
	}
}

// Checks that tool and tools say "not running" with code 3 when the app is
// closed, and do not start the app.
func TestToolsWhenClosed(t *testing.T) {
	home := t.TempDir()
	testapp.Prepare(t, filepath.Join(home, ".sprixel"))
	for _, args := range [][]string{{"tools"}, {"tool", "list_sprites"}} {
		r := appRun(t, home, args...)
		if r.code != ExitNotRunning || !strings.Contains(r.err, "sprixel start") {
			t.Errorf("%v: code = %d, stderr = %q", args, r.code, r.err)
		}
	}
}

// Checks "tools": one line per tool; and that a client that is not allowed
// gets an explanation of how to allow it, and a tool call shows the app's
// own approval message.
func TestToolsListAndApproval(t *testing.T) {
	home, app := appHome(t)
	r := appRun(t, home, "tools")
	if r.code != ExitOK || !strings.Contains(r.stdout, "list_sprites") || !strings.Contains(r.stdout, "Lists the sprites.") {
		t.Errorf("code = %d, stdout = %q", r.code, r.stdout)
	}

	app.SetApproved(false)
	if r := appRun(t, home, "tools"); r.code != ExitFailure || !strings.Contains(r.err, "Sprixel CLI") {
		t.Errorf("tools: code = %d, stderr = %q", r.code, r.err)
	}
	if r := appRun(t, home, "tool", "list_sprites"); r.code != ExitToolError || !strings.Contains(r.err, "awaiting_approval") {
		t.Errorf("tool: code = %d, stderr = %q", r.code, r.err)
	}
}

// Checks "start" when the app is closed: the command starts it and waits for
// the server. The app does not read a project path yet, so a path is checked
// but NOT passed to the app, and the output says the project was not opened.
func TestStartStartsTheApp(t *testing.T) {
	home := t.TempDir()
	app := testapp.Prepare(t, filepath.Join(home, ".sprixel"))
	project := filepath.Join(home, "game")
	os.MkdirAll(project, 0o755)
	os.WriteFile(filepath.Join(project, "sprixel.json"), []byte("{}"), 0o644)
	var got []string

	var out, errOut strings.Builder
	code := Run([]string{"start", project}, Env{Stdout: &out, Stderr: &errOut, Home: home,
		StartApp: func(args ...string) (<-chan error, error) {
			got = args
			app.Open()
			return make(chan error), nil
		}})

	if code != ExitOK || !strings.Contains(out.String(), "started") {
		t.Errorf("code = %d, stdout = %q, stderr = %q", code, out.String(), errOut.String())
	}
	if len(got) != 0 {
		t.Errorf("app args = %v, want none", got)
	}
	if !strings.Contains(out.String(), "note: the app does not open a project from the command line yet") || !strings.Contains(out.String(), project+" was not opened") {
		t.Errorf("stdout = %q", out.String())
	}
}

// Checks that "start" on a running app does nothing, and that a path that is
// not a project is a usage error that starts nothing.
func TestStartWhenRunningAndBadPath(t *testing.T) {
	home, _ := appHome(t)
	if r := appRun(t, home, "start"); r.code != ExitOK || !strings.Contains(r.stdout, "already running") {
		t.Errorf("code = %d, stdout = %q", r.code, r.stdout)
	}
	if r := appRun(t, home, "start", home); r.code != ExitUsage || !strings.Contains(r.err, "not a Sprixel project") {
		t.Errorf("code = %d, stderr = %q", r.code, r.err)
	}
	if r := appRun(t, home, "start", filepath.Join(home, "missing")); r.code != ExitUsage {
		t.Errorf("code = %d", r.code)
	}
}

// Checks that "start" with no screen explains it needs a desktop session.
func TestStartWithoutDisplay(t *testing.T) {
	home := t.TempDir()
	var out, errOut strings.Builder
	code := Run([]string{"start"}, Env{Stdout: &out, Stderr: &errOut, Home: home,
		StartApp: func(args ...string) (<-chan error, error) { return nil, errNoDisplayForTest }})
	if code != ExitFailure || !strings.Contains(errOut.String(), "desktop session") {
		t.Errorf("code = %d, stderr = %q", code, errOut.String())
	}
}

var errNoDisplayForTest = launcher.ErrNoDisplay

// Checks the scenario "Many calls": every `sprixel tool`, `sprixel tools` and
// failing call ends its session, so 20 runs leave none open (the app refuses
// new clients at 16). Also a call that fails (unknown tool) must close.
func TestCommandsCloseTheirSession(t *testing.T) {
	home, app := appHome(t)
	for i := 0; i < 20; i++ {
		if r := appRun(t, home, "tool", "list_sprites"); r.code != ExitOK {
			t.Fatalf("call %d: code = %d, stderr = %q", i, r.code, r.err)
		}
	}
	appRun(t, home, "tools")
	appRun(t, home, "tool", "no_such_tool")
	if n := app.OpenSessions(); n != 0 {
		t.Errorf("open sessions = %d, want 0", n)
	}
}

// Checks the scenario "Old build": `sprixel start` prints the path and the
// version of the app; when the version differs from the command it warns and
// names SPRIXEL_APP. With the same version there is no warning. It also works
// when the app was already running, and --json carries the same facts.
func TestStartPrintsPathAndVersion(t *testing.T) {
	home := t.TempDir()
	app := testapp.Start(t, filepath.Join(home, ".sprixel"))
	_ = app
	fake := filepath.Join(home, "my-sprixel-app")
	os.WriteFile(fake, []byte("x"), 0o755)
	t.Setenv("SPRIXEL_APP", fake)
	old := version.Version
	t.Cleanup(func() { version.Version = old })

	version.Version = "1.0.0"
	r := appRun(t, home, "start")
	for _, want := range []string{"version " + testapp.Version, "app: " + fake, "warning:", "1.0.0", "SPRIXEL_APP"} {
		if !strings.Contains(r.stdout, want) {
			t.Errorf("missing %q in %q", want, r.stdout)
		}
	}
	r = appRun(t, home, "start", "--json")
	var data map[string]any
	if err := json.Unmarshal([]byte(r.stdout), &data); err != nil || data["path"] != fake || data["version"] != testapp.Version || data["warning"] == nil {
		t.Errorf("json = %q, %v", r.stdout, err)
	}

	version.Version = testapp.Version
	if r := appRun(t, home, "start"); strings.Contains(r.stdout, "warning") || !strings.Contains(r.stdout, "app: "+fake) {
		t.Errorf("same version: %q", r.stdout)
	}
}

// Checks the same facts when the command itself starts the app.
func TestStartedAppIsDescribedToo(t *testing.T) {
	home := t.TempDir()
	app := testapp.Prepare(t, filepath.Join(home, ".sprixel"))
	fake := filepath.Join(home, "my-sprixel-app")
	os.WriteFile(fake, []byte("x"), 0o755)
	t.Setenv("SPRIXEL_APP", fake)
	var out, errOut strings.Builder
	code := Run([]string{"start"}, Env{Stdout: &out, Stderr: &errOut, Home: home,
		StartApp: func(...string) (<-chan error, error) { app.Open(); return make(chan error), nil }})
	if code != ExitOK || !strings.Contains(out.String(), "Sprixel started (port") || !strings.Contains(out.String(), "app: "+fake) {
		t.Errorf("code = %d, stdout = %q, stderr = %q", code, out.String(), errOut.String())
	}
}

// previewFiles lists the PNG files of the previews folder of a fake home.
func previewFiles(t *testing.T, home string) []string {
	t.Helper()
	entries, err := os.ReadDir(filepath.Join(home, ".sprixel", "previews"))
	if err != nil {
		t.Fatal(err)
	}
	var names []string
	for _, e := range entries {
		names = append(names, e.Name())
	}
	return names
}

// Checks the scenario "Through the command line" of the spec agent-live-design:
// a write tool (set_pixels) returns an image with its answer; the command saves
// the image as a PNG file, prints its path, and still prints the JSON answer
// with the new revision.
func TestToolSavesTheImageOfAWrite(t *testing.T) {
	home, _ := appHome(t)

	r := appRun(t, home, "tool", "set_pixels", "name=goblin", "preview=true")

	path := filepath.Join(home, ".sprixel", "previews", "goblin-1-9.png")
	if r.code != ExitOK || !strings.Contains(r.stdout, "image saved: "+path) || !strings.Contains(r.stdout, `"revision": 9`) {
		t.Fatalf("code = %d, stdout = %q, stderr = %q", r.code, r.stdout, r.err)
	}
	data, err := os.ReadFile(path)
	if err != nil || !strings.HasPrefix(string(data), "\x89PNG") {
		t.Fatalf("the file is not a PNG: %v", err)
	}
}

// Checks the scenario "Preview from the shell": the image of render_preview is
// saved as a real PNG named after the sprite, frame and revision, in
// ~/.sprixel/previews, only the user can read it, and the output prints its path.
func TestToolSavesTheImageToAFile(t *testing.T) {
	home, _ := appHome(t)

	r := appRun(t, home, "tool", "render_preview", "name=goblin", "frame=2")

	path := filepath.Join(home, ".sprixel", "previews", "goblin-2-7.png")
	if r.code != ExitOK || !strings.Contains(r.stdout, "image saved: "+path) {
		t.Fatalf("code = %d, stdout = %q, stderr = %q", r.code, r.stdout, r.err)
	}
	data, err := os.ReadFile(path)
	if err != nil || !strings.HasPrefix(string(data), "\x89PNG") {
		t.Fatalf("the file is not a PNG: %v", err)
	}
	if runtime.GOOS != "windows" {
		for p, want := range map[string]os.FileMode{path: 0o600, filepath.Dir(path): 0o700} {
			if info, _ := os.Stat(p); info.Mode().Perm() != want {
				t.Errorf("%s mode = %v, want %v", p, info.Mode().Perm(), want)
			}
		}
	}
	if len(previewFiles(t, home)) != 1 {
		t.Errorf("files = %v (no temporary file may stay)", previewFiles(t, home))
	}
}

// Checks task 2.4 of the spec agent-previews: render_frames and render_onion
// images are saved too. The file name has the sprite, the kind, the revision
// and a hash of the image, so two different ranges never share a file, and the
// same request twice gives the same file.
func TestToolSavesFramesAndOnionImages(t *testing.T) {
	home, _ := appHome(t)

	a := appRun(t, home, "tool", "render_frames", "name=goblin", "from=1", "to=2")
	b := appRun(t, home, "tool", "render_frames", "name=goblin", "from=1", "to=3")
	c := appRun(t, home, "tool", "render_onion", "name=goblin")
	again := appRun(t, home, "tool", "render_frames", "name=goblin", "from=1", "to=2")

	for _, r := range []result{a, b, c} {
		if r.code != ExitOK || !strings.Contains(r.stdout, "image saved: ") {
			t.Fatalf("code = %d, stdout = %q, stderr = %q", r.code, r.stdout, r.err)
		}
	}
	files := previewFiles(t, home)
	if len(files) != 3 {
		t.Fatalf("files = %v, want 3 different files", files)
	}
	for _, f := range files {
		if !regexp.MustCompile(`^goblin-(frames|onion)-7-[0-9a-f]{8}\.png$`).MatchString(f) {
			t.Errorf("unexpected file name %q", f)
		}
	}
	if a.stdout != again.stdout {
		t.Errorf("same request, different output: %q and %q", a.stdout, again.stdout)
	}
}

// Checks that --json keeps the base64 data and adds the path.
func TestToolImageJSONHasDataAndPath(t *testing.T) {
	home, _ := appHome(t)
	r := appRun(t, home, "tool", "render_preview", "name=goblin", "--json")
	var out struct {
		Images []map[string]string
	}
	if err := json.Unmarshal([]byte(r.stdout), &out); err != nil || len(out.Images) != 1 {
		t.Fatalf("json = %q, %v", r.stdout, err)
	}
	image := out.Images[0]
	if image["data"] == "" || image["mime_type"] != "image/png" || image["path"] != filepath.Join(home, ".sprixel", "previews", "goblin-1-7.png") {
		t.Errorf("image = %v", image)
	}
}

// Checks that a name from the app cannot leave the folder or break the file name.
func TestPreviewFileNameIsSafe(t *testing.T) {
	home, _ := appHome(t)
	appRun(t, home, "tool", "render_preview", "name=../../evil name")
	for _, name := range previewFiles(t, home) {
		if strings.ContainsAny(name, `/\ `) || !strings.HasSuffix(name, ".png") {
			t.Errorf("unsafe name %q", name)
		}
	}
	if _, err := os.Stat(filepath.Join(home, "evil name-1-7.png")); err == nil {
		t.Error("a file was written outside the previews folder")
	}
}

// Checks the scenario "Many previews": after 25 previews only the 20 newest
// files stay; the oldest ones are deleted, and other files are not touched.
func TestOnlyTheTwentyNewestPreviewsStay(t *testing.T) {
	home, _ := appHome(t)
	dir := filepath.Join(home, ".sprixel", "previews")
	os.MkdirAll(dir, 0o700)
	os.WriteFile(filepath.Join(dir, "notes.txt"), []byte("mine"), 0o600)
	start := time.Now().Add(-time.Hour)
	for i := 1; i <= 25; i++ {
		appRun(t, home, "tool", "render_preview", "name=s"+strconv.Itoa(i))
		// Give each file its own time, so "newest" does not depend on the clock.
		os.Chtimes(filepath.Join(dir, "s"+strconv.Itoa(i)+"-1-7.png"), start, start.Add(time.Duration(i)*time.Second))
	}

	names := previewFiles(t, home)
	png := 0
	for _, n := range names {
		if strings.HasSuffix(n, ".png") {
			png++
		}
	}
	if png != 20 {
		t.Fatalf("png files = %d, want 20 (%v)", png, names)
	}
	for i := 1; i <= 5; i++ {
		if _, err := os.Stat(filepath.Join(dir, "s"+strconv.Itoa(i)+"-1-7.png")); err == nil {
			t.Errorf("s%d is old and must be deleted", i)
		}
	}
	if _, err := os.Stat(filepath.Join(dir, "notes.txt")); err != nil {
		t.Error("a file that is not a preview was deleted")
	}
}

// Checks that a folder that cannot be made does not fail the call: the tool
// worked, so the command warns on stderr, keeps the old base64 hint and exits 0.
func TestToolImageSaveFailureIsOnlyAWarning(t *testing.T) {
	home, _ := appHome(t)
	// "previews" is a file, so the folder cannot be created.
	os.WriteFile(filepath.Join(home, ".sprixel", "previews"), []byte("x"), 0o600)

	r := appRun(t, home, "tool", "render_preview", "name=goblin")

	if r.code != ExitOK || !strings.Contains(r.err, "could not save the image") || !strings.Contains(r.stdout, "base64 characters") {
		t.Errorf("code = %d, stdout = %q, stderr = %q", r.code, r.stdout, r.err)
	}
}
