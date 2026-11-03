package doctor

import (
	"context"
	"errors"
	"os"
	"path/filepath"
	"strings"
	"testing"

	"github.com/Javier-GarciaP/sprixel/cli/internal/clients"
	"github.com/Javier-GarciaP/sprixel/cli/internal/testapp"
)

// fakeClient is an AI tool with a fixed state.
type fakeClient struct {
	id    string
	state clients.State
	err   error
}

func (f fakeClient) ID() string                         { return f.id }
func (f fakeClient) Title() string                      { return strings.ToUpper(f.id) }
func (f fakeClient) Add(string) (clients.Result, error) { return clients.Result{}, nil }
func (f fakeClient) Remove() (clients.Result, error)    { return clients.Result{}, nil }
func (f fakeClient) State() (clients.State, error)      { return f.state, f.err }
func (f fakeClient) Snippet(string) string              { return "" }

func deps(dir string, list ...clients.Client) Deps {
	return Deps{
		Dir: dir, Version: testapp.Version, Command: "sprixel",
		FindApp:    func() (string, error) { return "/apps/sprixel-app", nil },
		Clients:    list,
		HasProgram: func(string) bool { return true },
	}
}

func byName(checks []Check) map[string]Check {
	m := map[string]Check{}
	for _, c := range checks {
		m[c.Name] = c
	}
	return m
}

// Checks the scenario "Everything is fine": every check is OK (or skipped
// because it does not apply) and the report does not count as failed.
func TestEverythingIsFine(t *testing.T) {
	app := testapp.Start(t, t.TempDir())
	checks := Run(context.Background(), deps(app.Dir, fakeClient{id: "claude", state: clients.StateYes}, fakeClient{id: "opencode", state: clients.StateYes}))

	if Failed(checks) {
		t.Errorf("checks = %+v", checks)
	}
	for _, name := range []string{"installation", "app", "version", "token", "server", "claude", "opencode"} {
		if got := byName(checks)[name]; got.Status != OK {
			t.Errorf("%s = %+v", name, got)
		}
	}
}

// Checks the scenario "Token permissions are too open": the check fails and
// tells how to fix it. (Skipped where the system has no file modes.)
func TestOpenTokenFileFails(t *testing.T) {
	app := testapp.Start(t, t.TempDir())
	if err := os.Chmod(filepath.Join(app.Dir, "token"), 0o644); err != nil {
		t.Skip("cannot change modes here")
	}
	checks := Run(context.Background(), deps(app.Dir))
	got := byName(checks)["token permissions"]
	if got.Status == Skip {
		t.Skip("permissions are not checked on this system")
	}
	if got.Status != Fail || !strings.Contains(got.Fix, "chmod 600") || !Failed(checks) {
		t.Errorf("check = %+v", got)
	}
}

// Checks a failing run: the app is closed, so the app check fails with the
// command to start it, the server and version checks are skipped, and the
// report as a whole is failed.
func TestClosedAppFails(t *testing.T) {
	app := testapp.Prepare(t, t.TempDir())
	checks := Run(context.Background(), deps(app.Dir))

	m := byName(checks)
	if m["app"].Status != Fail || !strings.Contains(m["app"].Fix, "sprixel start") {
		t.Errorf("app = %+v", m["app"])
	}
	if m["server"].Status != Skip || m["version"].Status != Skip {
		t.Errorf("server = %+v, version = %+v", m["server"], m["version"])
	}
	if !Failed(checks) {
		t.Error("the report must be failed")
	}
}

// Checks that a token the app refuses is a failed server check, and that no
// message contains the token.
func TestRefusedTokenFails(t *testing.T) {
	app := testapp.Start(t, t.TempDir())
	os.WriteFile(filepath.Join(app.Dir, "token"), []byte("my-wrong-secret"), 0o600)
	checks := Run(context.Background(), deps(app.Dir))

	if got := byName(checks)["server"]; got.Status != Fail {
		t.Errorf("server = %+v", got)
	}
	for _, c := range checks {
		if strings.Contains(c.Detail+c.Fix, "my-wrong-secret") {
			t.Errorf("%s repeats the token", c.Name)
		}
	}
}

// Checks that an app and a command with different versions fail the version
// check.
func TestVersionMismatchFails(t *testing.T) {
	app := testapp.Start(t, t.TempDir())
	d := deps(app.Dir)
	d.Version = "0.0.1"
	if got := byName(Run(context.Background(), d))["version"]; got.Status != Fail || !strings.Contains(got.Detail, testapp.Version) {
		t.Errorf("version = %+v", got)
	}
}

// Checks the registration checks: not registered fails with the command to
// run, a client that is not installed is skipped, and an unreadable setup fails.
func TestRegistrationChecks(t *testing.T) {
	app := testapp.Start(t, t.TempDir())
	d := deps(app.Dir,
		fakeClient{id: "claude", state: clients.StateNo},
		fakeClient{id: "other", state: clients.StateMissing},
		fakeClient{id: "broken", err: errors.New("bad file\nmore")})
	m := byName(Run(context.Background(), d))

	if m["claude"].Status != Fail || m["claude"].Fix != "run `sprixel mcp add claude`" {
		t.Errorf("claude = %+v", m["claude"])
	}
	if m["other"].Status != Skip {
		t.Errorf("other = %+v", m["other"])
	}
	if m["broken"].Status != Fail || strings.Contains(m["broken"].Detail, "more") {
		t.Errorf("broken = %+v", m["broken"])
	}
}

// Checks that OpenCode is skipped when the program is not installed, and
// that no app and no token give the install and token failures.
func TestMissingPiecesFail(t *testing.T) {
	d := deps(t.TempDir(), fakeClient{id: "opencode", state: clients.StateNo})
	d.HasProgram = func(string) bool { return false }
	d.FindApp = func() (string, error) { return "", errors.New("none") }

	m := byName(Run(context.Background(), d))

	if m["opencode"].Status != Skip {
		t.Errorf("opencode = %+v", m["opencode"])
	}
	if m["installation"].Status != Fail || m["token"].Status != Fail {
		t.Errorf("installation = %+v, token = %+v", m["installation"], m["token"])
	}
}
