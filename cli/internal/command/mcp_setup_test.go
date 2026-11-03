package command

import (
	"encoding/json"
	"errors"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"testing"
)

// fakeClaude is a `claude` command that keeps its servers in memory.
type fakeClaude struct {
	has   bool
	calls []string
}

func (f *fakeClaude) run(name string, args ...string) (string, error) {
	f.calls = append(f.calls, strings.Join(args, " "))
	switch args[1] {
	case "get":
		if f.has {
			return "sprixel:\n  Scope: User config\n", nil
		}
		return "No MCP server found\n", errors.New("exit status 1")
	case "add":
		f.has = true
	case "remove":
		f.has = false
	}
	return "ok\n", nil
}

func setup(t *testing.T, f *fakeClaude, args ...string) result {
	t.Helper()
	return setupIn(t, t.TempDir(), f, args...)
}

func setupIn(t *testing.T, home string, f *fakeClaude, args ...string) result {
	t.Helper()
	var out, errOut strings.Builder
	exe := filepath.Join(home, "bin", "sprixel")
	code := Run(args, Env{Stdin: strings.NewReader(""), Stdout: &out, Stderr: &errOut, Home: home,
		Exe: exe, LookPath: func(string) (string, error) { return "", exec.ErrNotFound }, Runner: f.run})
	return result{code, out.String(), errOut.String()}
}

// Checks the scenario "Add for Claude Code" through the command: it runs
// `claude mcp add`, tells what changed and how to undo it.
func TestMCPAddClaude(t *testing.T) {
	f := &fakeClaude{}
	r := setup(t, f, "mcp", "add", "claude")

	if r.code != ExitOK || !strings.Contains(r.stdout, "undo:    sprixel mcp remove claude") {
		t.Errorf("code = %d, stdout = %q, stderr = %q", r.code, r.stdout, r.err)
	}
	if !f.has {
		t.Error("claude mcp add was not run")
	}
}

// Checks the scenario "Run twice": the second run says it is already
// registered and runs `claude mcp add` only once.
func TestMCPAddTwice(t *testing.T) {
	f := &fakeClaude{}
	home := t.TempDir()
	setupIn(t, home, f, "mcp", "add", "claude")
	r := setupIn(t, home, f, "mcp", "add", "claude")

	if r.code != ExitOK || !strings.Contains(r.stdout, "already registered") {
		t.Errorf("stdout = %q", r.stdout)
	}
	adds := 0
	for _, call := range f.calls {
		if strings.HasPrefix(call, "mcp add") {
			adds++
		}
	}
	if adds != 1 {
		t.Errorf("claude mcp add ran %d times", adds)
	}
}

// Checks the scenarios "First add" and "Already registered" of the skill:
// the first add writes ~/.claude/skills/sprixel/SKILL.md (in a temporary
// home), and running it again, when Sprixel is already registered, repairs a
// file that is missing or old.
func TestMCPAddClaudeInstallsTheSkill(t *testing.T) {
	f := &fakeClaude{}
	home := t.TempDir()
	path := filepath.Join(home, ".claude", "skills", "sprixel", "SKILL.md")

	r := setupIn(t, home, f, "mcp", "add", "claude")
	data, err := os.ReadFile(path)
	if r.code != ExitOK || err != nil || !strings.HasPrefix(string(data), "---\nname: sprixel") {
		t.Fatalf("first add: code = %d, err = %v, stderr = %q", r.code, err, r.err)
	}
	if !strings.Contains(r.stdout, "skill:   "+path) {
		t.Errorf("stdout = %q", r.stdout)
	}

	os.WriteFile(path, []byte("old"), 0o644)
	r = setupIn(t, home, f, "mcp", "add", "claude")
	data, _ = os.ReadFile(path)
	if !strings.Contains(r.stdout, "already registered") || !strings.HasPrefix(string(data), "---\nname: sprixel") {
		t.Errorf("second add did not write the skill: %q, %q", r.stdout, data)
	}

	os.Remove(path)
	setupIn(t, home, f, "mcp", "add", "claude", "--json")
	if _, err := os.Stat(path); err != nil {
		t.Error("a missing skill was not written again")
	}
}

// Checks that OpenCode gets no skill: only Claude Code has this feature.
func TestMCPAddOpenCodeInstallsNoSkill(t *testing.T) {
	home := t.TempDir()
	setupIn(t, home, &fakeClaude{}, "mcp", "add", "opencode")
	if _, err := os.Stat(filepath.Join(home, ".claude")); err == nil {
		t.Error("a .claude folder was made")
	}
}

// Checks the registered command: when sprixel is not on the PATH, the full
// path of the program is registered, so the AI tool can start it. This is also
// the Windows case (a path with a drive letter and backslashes is one argument).
func TestMCPAddUsesTheFullPathWhenNotOnPath(t *testing.T) {
	f := &fakeClaude{}
	home := t.TempDir()
	setupIn(t, home, f, "mcp", "add", "claude")
	want := "mcp add --scope user sprixel -- " + filepath.Join(home, "bin", "sprixel") + " mcp"
	if got := f.calls[len(f.calls)-1]; got != want {
		t.Errorf("call = %q, want %q", got, want)
	}
}

// Checks that "sprixel" is registered when the PATH finds this same program.
func TestMCPAddUsesTheNameWhenOnPath(t *testing.T) {
	home := t.TempDir()
	exe := filepath.Join(home, "sprixel")
	os.WriteFile(exe, []byte("x"), 0o755)
	f := &fakeClaude{}
	var out strings.Builder
	Run([]string{"mcp", "add", "claude"}, Env{Stdout: &out, Stderr: &out, Home: home, Exe: exe, Runner: f.run,
		LookPath: func(string) (string, error) { return exe, nil }})
	if got := f.calls[len(f.calls)-1]; got != "mcp add --scope user sprixel -- sprixel mcp" {
		t.Errorf("call = %q", got)
	}
}

// Checks the scenario "Remove" for OpenCode through the command: the sprixel
// server goes and another server stays.
func TestMCPRemoveOpenCode(t *testing.T) {
	home := t.TempDir()
	dir := filepath.Join(home, ".config", "opencode")
	os.MkdirAll(dir, 0o755)
	os.WriteFile(filepath.Join(dir, "opencode.json"), []byte(`{"mcp":{"keep":{"type":"local","command":["a"]}}}`), 0o644)
	f := &fakeClaude{}
	if r := setupIn(t, home, f, "mcp", "add", "opencode"); r.code != ExitOK {
		t.Fatalf("add: %+v", r)
	}

	r := setupIn(t, home, f, "mcp", "remove", "opencode")

	if r.code != ExitOK {
		t.Fatalf("remove: %+v", r)
	}
	data, _ := os.ReadFile(filepath.Join(dir, "opencode.json"))
	if strings.Contains(string(data), "sprixel") || !strings.Contains(string(data), "keep") {
		t.Errorf("config = %s", data)
	}
}

// Checks "mcp list": one line per client with its state, and valid JSON with
// --json. Claude is registered here, OpenCode is not.
func TestMCPList(t *testing.T) {
	f := &fakeClaude{has: true}
	r := setup(t, f, "mcp", "list")
	if r.code != ExitOK || !strings.Contains(r.stdout, "Claude Code") || !strings.Contains(r.stdout, "not registered") {
		t.Errorf("stdout = %q", r.stdout)
	}

	r = setup(t, f, "mcp", "list", "--json")
	var data struct{ Clients []map[string]string }
	if err := json.Unmarshal([]byte(r.stdout), &data); err != nil || len(data.Clients) != 2 || data.Clients[0]["state"] != "registered" {
		t.Errorf("json = %q, %v", r.stdout, err)
	}
}

// Checks the usage errors: no client, an unknown client, an unknown word.
func TestMCPSetupUsageErrors(t *testing.T) {
	for _, args := range [][]string{{"mcp", "add"}, {"mcp", "add", "cursor"}, {"mcp", "frobnicate"}, {"mcp", "list", "x"}} {
		if r := setup(t, &fakeClaude{}, args...); r.code != ExitUsage {
			t.Errorf("%v: code = %d, want %d", args, r.code, ExitUsage)
		}
	}
}

// Checks that a JSONC OpenCode file gives a failure with the text to add by
// hand, and does not touch the file.
func TestMCPAddOpenCodeWithCommentsShowsTheSnippet(t *testing.T) {
	home := t.TempDir()
	dir := filepath.Join(home, ".config", "opencode")
	os.MkdirAll(dir, 0o755)
	text := "{ // note\n}"
	os.WriteFile(filepath.Join(dir, "opencode.json"), []byte(text), 0o644)

	r := setupIn(t, home, &fakeClaude{}, "mcp", "add", "opencode")

	if r.code != ExitFailure || !strings.Contains(r.err, `"command"`) {
		t.Errorf("code = %d, stderr = %q", r.code, r.err)
	}
	if got, _ := os.ReadFile(filepath.Join(dir, "opencode.json")); string(got) != text {
		t.Error("the file was changed")
	}
}
