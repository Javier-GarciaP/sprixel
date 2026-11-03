package command

import (
	"encoding/json"
	"os"
	"path/filepath"
	"strings"
	"testing"

	"github.com/Javier-GarciaP/sprixel/cli/internal/testapp"
	"github.com/Javier-GarciaP/sprixel/cli/internal/version"
)

// Checks a passing run through the command: one line per check, the app and
// the clients are OK, and the exit code is 0. The fake app reports the version
// of the command.
func TestDoctorPassing(t *testing.T) {
	home := t.TempDir()
	testapp.Start(t, filepath.Join(home, ".sprixel"))
	// The fake app reports its own version; make the command match it.
	old := version.Version
	version.Version = testapp.Version
	t.Cleanup(func() { version.Version = old })
	t.Setenv("SPRIXEL_APP", os.Args[0]) // any file that exists: the app is "installed"
	dir := filepath.Join(home, ".config", "opencode")
	os.MkdirAll(dir, 0o755)
	os.WriteFile(filepath.Join(dir, "opencode.json"), []byte(`{"mcp":{"sprixel":{"type":"local","command":["sprixel","mcp"],"enabled":true}}}`), 0o600)
	f := &fakeClaude{has: true}

	r := setupIn(t, home, f, "doctor")

	for _, line := range []string{"OK    app", "OK    version", "OK    server", "OK    claude", "OK    opencode"} {
		if !strings.Contains(r.stdout, line) {
			t.Errorf("missing %q in %q", line, r.stdout)
		}
	}
	if r.code != ExitOK {
		t.Errorf("code = %d, stdout = %q", r.code, r.stdout)
	}
}

// Checks a failing run through the command: the app is closed, so there is a
// FAIL line with its fix, the exit code is non-zero, and --json is valid.
func TestDoctorFailing(t *testing.T) {
	home := t.TempDir()
	testapp.Prepare(t, filepath.Join(home, ".sprixel"))
	t.Setenv("SPRIXEL_APP", os.Args[0])

	r := setupIn(t, home, &fakeClaude{}, "doctor")
	if r.code != ExitFailure || !strings.Contains(r.stdout, "FAIL  app") || !strings.Contains(r.stdout, "fix: run `sprixel start`") {
		t.Errorf("code = %d, stdout = %q", r.code, r.stdout)
	}

	r = setupIn(t, home, &fakeClaude{}, "doctor", "--json")
	var report struct {
		OK     bool
		Checks []map[string]string
	}
	if err := json.Unmarshal([]byte(r.stdout), &report); err != nil || report.OK || len(report.Checks) < 7 {
		t.Errorf("json = %q, %v", r.stdout, err)
	}
}
