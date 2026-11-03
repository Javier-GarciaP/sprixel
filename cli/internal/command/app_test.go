package command

import (
	"encoding/json"
	"os"
	"path/filepath"
	"strings"
	"testing"

	"github.com/Javier-GarciaP/sprixel/cli/internal/launcher"
	"github.com/Javier-GarciaP/sprixel/cli/internal/testapp"
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

// Checks the scenario "start" when the app is closed: the command starts it
// (with the project path made absolute), waits for the server and returns.
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
	if len(got) != 1 || got[0] != project {
		t.Errorf("app args = %v", got)
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
