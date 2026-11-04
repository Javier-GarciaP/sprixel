package command

import (
	"bytes"
	"encoding/json"
	"strings"
	"testing"

	"github.com/Javier-GarciaP/sprixel/cli/internal/version"
)

// result is what a test gets back from running the command.
type result struct {
	code        int
	stdout, err string
}

// run runs the command with a fresh home folder (or the given one).
func run(t *testing.T, args ...string) result {
	t.Helper()
	return runIn(t, t.TempDir(), args...)
}

func runIn(t *testing.T, home string, args ...string) result {
	t.Helper()
	var out, errOut bytes.Buffer
	code := Run(args, Env{Stdin: strings.NewReader(""), Stdout: &out, Stderr: &errOut, Home: home})
	return result{code, out.String(), errOut.String()}
}

// Checks that "version" prints the version with a newline and returns 0.
func TestVersionPrintsVersionAndReturnsZero(t *testing.T) {
	r := run(t, "version")
	if r.code != ExitOK {
		t.Errorf("exit code = %d, want 0", r.code)
	}
	if want := version.Version + "\n"; r.stdout != want {
		t.Errorf("output = %q, want %q", r.stdout, want)
	}
}

// Checks the documented codes: they are distinct and 0 means success.
func TestExitCodesAreDistinct(t *testing.T) {
	codes := map[int]string{}
	for name, code := range map[string]int{
		"ok": ExitOK, "failure": ExitFailure, "usage": ExitUsage,
		"not running": ExitNotRunning, "tool error": ExitToolError,
	} {
		if other, dup := codes[code]; dup {
			t.Errorf("%s and %s share the code %d", name, other, code)
		}
		codes[code] = name
	}
	if ExitOK != 0 {
		t.Error("success must be 0")
	}
}

// Checks that an unknown command is a usage error (code 2), names the
// command in the message and sends it to stderr, not stdout.
func TestUnknownCommandIsAUsageError(t *testing.T) {
	r := run(t, "nope")
	if r.code != ExitUsage {
		t.Errorf("exit code = %d, want %d", r.code, ExitUsage)
	}
	if !strings.Contains(r.err, "nope") || r.stdout != "" {
		t.Errorf("stderr = %q, stdout = %q", r.err, r.stdout)
	}
}

// Checks that no arguments is a usage error that shows the help on stderr.
func TestNoArgumentsShowsHelpAsUsageError(t *testing.T) {
	r := run(t)
	if r.code != ExitUsage || !strings.Contains(r.err, "usage: sprixel") {
		t.Errorf("code = %d, stderr = %q", r.code, r.err)
	}
}

// Checks that --help works for the program and for a command, prints to
// stdout and returns 0, also when it comes after the command name.
func TestHelpFlag(t *testing.T) {
	for _, args := range [][]string{{"--help"}, {"-h"}, {"version", "--help"}, {"--help", "version"}} {
		r := run(t, args...)
		if r.code != ExitOK || !strings.Contains(r.stdout, "usage:") {
			t.Errorf("%v: code = %d, stdout = %q", args, r.code, r.stdout)
		}
	}
	if r := run(t, "--help"); !strings.Contains(r.stdout, "version") || !strings.Contains(r.stdout, "exit codes") {
		t.Errorf("the help must list the commands and the exit codes: %q", r.stdout)
	}
}

// Checks that --json prints valid JSON only (the scenario "JSON output"),
// for a result and for an error, and that the flag works in any position.
func TestJSONOutputIsValidJSONOnly(t *testing.T) {
	r := run(t, "version", "--json")
	var ok map[string]string
	if err := json.Unmarshal([]byte(r.stdout), &ok); err != nil || ok["version"] != version.Version {
		t.Errorf("result: %q, %v", r.stdout, err)
	}
	if r.err != "" {
		t.Errorf("nothing may go to stderr, got %q", r.err)
	}

	r = run(t, "--json", "nope")
	var failed struct {
		Error struct{ Code, Message, Hint string }
	}
	if err := json.Unmarshal([]byte(r.stdout), &failed); err != nil {
		t.Fatalf("error: %q, %v", r.stdout, err)
	}
	if r.code != ExitUsage || failed.Error.Code != "usage" || failed.Error.Message == "" {
		t.Errorf("code = %d, error = %+v", r.code, failed.Error)
	}
}

// Checks that a command that takes no arguments refuses extra ones.
func TestExtraArgumentIsAUsageError(t *testing.T) {
	if r := run(t, "version", "extra"); r.code != ExitUsage {
		t.Errorf("exit code = %d, want %d", r.code, ExitUsage)
	}
}
