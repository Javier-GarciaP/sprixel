package main

import (
	"bytes"
	"encoding/json"
	"os"
	"os/exec"
	"path/filepath"
	"runtime"
	"strings"
	"testing"

	"github.com/Javier-GarciaP/sprixel/cli/internal/testapp"
)

// build compiles the real sprixel command, so the test checks the same
// program that AI tools start.
func build(t *testing.T) string {
	t.Helper()
	name := "sprixel"
	if runtime.GOOS == "windows" {
		name += ".exe"
	}
	path := filepath.Join(t.TempDir(), name)
	if out, err := exec.Command("go", "build", "-o", path, ".").CombinedOutput(); err != nil {
		t.Fatalf("go build: %v\n%s", err, out)
	}
	return path
}

// runBridge starts `sprixel mcp` with the given home folder, writes the
// input and returns what it wrote to stdout and to stderr.
func runBridge(t *testing.T, exe, home, input string) (stdout, stderr string) {
	t.Helper()
	command := exec.Command(exe, "mcp")
	command.Env = append(os.Environ(),
		"HOME="+home, "USERPROFILE="+home,
		// An app that is not there: the bridge must not start a real one.
		"SPRIXEL_APP="+filepath.Join(home, "no-such-app"))
	var out, errOut bytes.Buffer
	command.Stdin, command.Stdout, command.Stderr = strings.NewReader(input), &out, &errOut
	if err := command.Run(); err != nil {
		t.Fatalf("sprixel mcp: %v\nstderr: %s", err, errOut.String())
	}
	return out.String(), errOut.String()
}

// requireProtocolOnly fails if any line of stdout is not a JSON-RPC message.
func requireProtocolOnly(t *testing.T, stdout string, wantLines int) {
	t.Helper()
	if stdout != "" && !strings.HasSuffix(stdout, "\n") {
		t.Errorf("stdout does not end with a newline: %q", stdout)
	}
	lines := strings.Split(strings.TrimSuffix(stdout, "\n"), "\n")
	if stdout == "" {
		lines = nil
	}
	if len(lines) != wantLines {
		t.Errorf("stdout has %d lines, want %d:\n%s", len(lines), wantLines, stdout)
	}
	for _, line := range lines {
		var m struct {
			JSONRPC string          `json:"jsonrpc"`
			Result  json.RawMessage `json:"result"`
			Error   json.RawMessage `json:"error"`
		}
		if err := json.Unmarshal([]byte(line), &m); err != nil || m.JSONRPC != "2.0" || (m.Result == nil && m.Error == nil) {
			t.Errorf("stdout has a line that is not a protocol message: %q", line)
		}
	}
}

// Checks the rule "stdout holds only protocol messages". It runs the real
// command in the situations that make a program print by mistake: a normal
// session, a bad message, a request the app refuses and a closed app. Every
// line on stdout must be a JSON-RPC message; the logs must be on stderr; and
// the token must be in neither of them.
func TestNothingButProtocolMessagesReachesStdout(t *testing.T) {
	exe := build(t)
	input := strings.Join([]string{
		`{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"clientInfo":{"name":"claude-code","version":"1"}}}`,
		`{"jsonrpc":"2.0","method":"notifications/initialized"}`,
		`{"jsonrpc":"2.0","id":2,"method":"tools/list"}`,
		`{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"list_sprites","arguments":{}}}`,
		`{"jsonrpc":"2.0","id":4,"method":"no/such/method"}`,
		`this is not json`,
	}, "\n") + "\n"

	t.Run("app running", func(t *testing.T) {
		home := t.TempDir()
		testapp.Start(t, filepath.Join(home, ".sprixel"))
		stdout, stderr := runBridge(t, exe, home, input)

		requireProtocolOnly(t, stdout, 5) // the notification has no answer
		if strings.Contains(stdout+stderr, testapp.Token) {
			t.Error("the token was printed")
		}
	})

	t.Run("app closed", func(t *testing.T) {
		home := t.TempDir()
		testapp.Prepare(t, filepath.Join(home, ".sprixel"))
		stdout, stderr := runBridge(t, exe, home, input)

		requireProtocolOnly(t, stdout, 5)
		if !strings.Contains(stderr, "sprixel mcp:") {
			t.Errorf("the logs must go to stderr, got %q", stderr)
		}
		if strings.Contains(stdout+stderr, testapp.Token) {
			t.Error("the token was printed")
		}
	})

	t.Run("no data folder at all", func(t *testing.T) {
		stdout, _ := runBridge(t, exe, t.TempDir(), input)
		requireProtocolOnly(t, stdout, 5)
	})
}
