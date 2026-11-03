package clients

import (
	"encoding/json"
	"errors"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"testing"
)

// fakeClaude imitates the `claude mcp` command with a list of servers.
type fakeClaude struct {
	servers map[string]bool
	calls   []string
	missing bool
}

func (f *fakeClaude) run(name string, args ...string) (string, error) {
	f.calls = append(f.calls, strings.Join(args, " "))
	if f.missing {
		return "", exec.ErrNotFound
	}
	switch {
	case len(args) >= 3 && args[1] == "get":
		if f.servers[args[2]] {
			return args[2] + ":\n  Scope: User config\n  Type: stdio\n  Command: sprixel\n", nil
		}
		return "No MCP server found with name: " + args[2] + "\n", errors.New("exit status 1")
	case len(args) >= 2 && args[1] == "add":
		f.servers[args[4]] = true
		return "Added stdio MCP server " + args[4] + "\n", nil
	case len(args) >= 2 && args[1] == "remove":
		delete(f.servers, args[2])
		return "Removed\n", nil
	}
	return "", errors.New("unexpected call")
}

// Checks the scenario "Add for Claude Code": the client runs the documented
// command (user scope, "--" before the server command) and the server is then
// listed as registered.
func TestClaudeAddRunsTheDocumentedCommand(t *testing.T) {
	f := &fakeClaude{servers: map[string]bool{}}
	c := Claude{Run: f.run}

	result, err := c.Add("sprixel")
	if err != nil || result.Outcome != Added {
		t.Fatalf("Add = %+v, %v", result, err)
	}
	if want := "mcp add --scope user sprixel -- sprixel mcp"; f.calls[len(f.calls)-1] != want {
		t.Errorf("last call = %q, want %q", f.calls[len(f.calls)-1], want)
	}
	if state, _ := c.State(); state != StateYes {
		t.Errorf("state = %v, want registered", state)
	}
}

// Checks the scenario "Run twice": the second Add adds nothing and says it
// was already registered.
func TestClaudeAddTwiceDoesNotDuplicate(t *testing.T) {
	f := &fakeClaude{servers: map[string]bool{}}
	c := Claude{Run: f.run}
	c.Add("sprixel")
	adds := 0
	for _, call := range f.calls {
		if strings.HasPrefix(call, "mcp add") {
			adds++
		}
	}

	result, err := c.Add("sprixel")

	if err != nil || result.Outcome != AlreadyRegistered {
		t.Errorf("second Add = %+v, %v", result, err)
	}
	for _, call := range f.calls {
		if strings.HasPrefix(call, "mcp add") {
			adds--
		}
	}
	if adds != 0 {
		t.Error("the second Add ran `claude mcp add` again")
	}
}

// Checks Remove for Claude Code: it removes the user scope entry, and says
// there was nothing to do when it is not registered.
func TestClaudeRemove(t *testing.T) {
	f := &fakeClaude{servers: map[string]bool{"sprixel": true}}
	c := Claude{Run: f.run}

	if r, err := c.Remove(); err != nil || r.Outcome != Removed {
		t.Fatalf("Remove = %+v, %v", r, err)
	}
	if f.calls[len(f.calls)-1] != "mcp remove sprixel --scope user" {
		t.Errorf("last call = %q", f.calls[len(f.calls)-1])
	}
	if r, err := c.Remove(); err != nil || r.Outcome != NotRegistered {
		t.Errorf("second Remove = %+v, %v", r, err)
	}
}

// Checks that a computer without Claude Code gets a clear error from Add and
// "missing" from State (so "mcp list" and "doctor" can say so).
func TestClaudeNotInstalled(t *testing.T) {
	c := Claude{Run: (&fakeClaude{missing: true}).run}
	if _, err := c.Add("sprixel"); !errors.Is(err, ErrNotInstalled) {
		t.Errorf("Add err = %v", err)
	}
	if state, err := c.State(); err != nil || state != StateMissing {
		t.Errorf("State = %v, %v", state, err)
	}
}

func readConfig(t *testing.T, home string) map[string]any {
	t.Helper()
	data, err := os.ReadFile(filepath.Join(home, ".config", "opencode", "opencode.json"))
	if err != nil {
		t.Fatal(err)
	}
	var config map[string]any
	if err := json.Unmarshal(data, &config); err != nil {
		t.Fatalf("not valid JSON: %v\n%s", err, data)
	}
	return config
}

func writeConfig(t *testing.T, home, name, text string) {
	t.Helper()
	dir := filepath.Join(home, ".config", "opencode")
	os.MkdirAll(dir, 0o755)
	if err := os.WriteFile(filepath.Join(dir, name), []byte(text), 0o600); err != nil {
		t.Fatal(err)
	}
}

// Checks Add for OpenCode on a computer with no config: the file is created
// with the documented shape (type local, command as an array).
func TestOpenCodeAddCreatesTheDocumentedEntry(t *testing.T) {
	home := t.TempDir()
	result, err := OpenCode{Home: home}.Add("sprixel")
	if err != nil || result.Outcome != Added {
		t.Fatalf("Add = %+v, %v", result, err)
	}

	config := readConfig(t, home)
	server := config["mcp"].(map[string]any)["sprixel"].(map[string]any)
	command := server["command"].([]any)
	if server["type"] != "local" || len(command) != 2 || command[0] != "sprixel" || command[1] != "mcp" || server["enabled"] != true {
		t.Errorf("entry = %v", server)
	}
	if config["$schema"] != schemaURL {
		t.Errorf("$schema = %v", config["$schema"])
	}
}

// Checks that Add keeps every other setting and every other MCP server, and
// that running it twice changes nothing (the file stays byte for byte equal).
func TestOpenCodeAddKeepsOtherSettingsAndIsIdempotent(t *testing.T) {
	home := t.TempDir()
	writeConfig(t, home, "opencode.json", `{
  "theme": "tokyonight",
  "model": "anthropic/claude-sonnet",
  "mcp": { "other": { "type": "remote", "url": "https://example.com/mcp" } }
}`)
	c := OpenCode{Home: home}
	if _, err := c.Add("sprixel"); err != nil {
		t.Fatal(err)
	}
	first, _ := os.ReadFile(c.path())

	result, err := c.Add("sprixel")
	second, _ := os.ReadFile(c.path())

	if err != nil || result.Outcome != AlreadyRegistered {
		t.Errorf("second Add = %+v, %v", result, err)
	}
	if string(first) != string(second) {
		t.Error("the file changed on the second Add")
	}
	config := readConfig(t, home)
	if config["theme"] != "tokyonight" || config["model"] != "anthropic/claude-sonnet" {
		t.Errorf("other settings lost: %v", config)
	}
	list := config["mcp"].(map[string]any)
	if list["other"] == nil || list["sprixel"] == nil || len(list) != 2 {
		t.Errorf("mcp = %v", list)
	}
}

// Checks that an entry with an old command is corrected, not duplicated.
func TestOpenCodeAddUpdatesAnOldEntry(t *testing.T) {
	home := t.TempDir()
	writeConfig(t, home, "opencode.json", `{"mcp":{"sprixel":{"type":"local","command":["old","mcp"]}}}`)
	result, err := OpenCode{Home: home}.Add("sprixel")
	if err != nil || result.Outcome != Updated {
		t.Fatalf("Add = %+v, %v", result, err)
	}
	cmd := readConfig(t, home)["mcp"].(map[string]any)["sprixel"].(map[string]any)["command"].([]any)
	if cmd[0] != "sprixel" {
		t.Errorf("command = %v", cmd)
	}
}

// Checks the scenario "Remove": only the sprixel server goes away; the other
// servers and settings stay; removing again says it was not registered.
func TestOpenCodeRemoveKeepsOtherServers(t *testing.T) {
	home := t.TempDir()
	writeConfig(t, home, "opencode.json", `{"theme":"x","mcp":{"other":{"type":"local","command":["a"]},"sprixel":{"type":"local","command":["sprixel","mcp"]}}}`)
	c := OpenCode{Home: home}

	if r, err := c.Remove(); err != nil || r.Outcome != Removed {
		t.Fatalf("Remove = %+v, %v", r, err)
	}
	config := readConfig(t, home)
	list := config["mcp"].(map[string]any)
	if _, gone := list["sprixel"]; gone || list["other"] == nil || config["theme"] != "x" {
		t.Errorf("config = %v", config)
	}
	if r, _ := c.Remove(); r.Outcome != NotRegistered {
		t.Errorf("second Remove = %+v", r)
	}
}

// Checks that a file with comments (JSONC) is never rewritten: the user gets
// an error that says what to do, and the file is untouched.
func TestOpenCodeRefusesAFileWithComments(t *testing.T) {
	home := t.TempDir()
	text := "{\n  // my theme\n  \"theme\": \"x\",\n}\n"
	writeConfig(t, home, "opencode.json", text)
	c := OpenCode{Home: home}

	_, err := c.Add("sprixel")

	if !errors.Is(err, ErrHasComments) {
		t.Fatalf("err = %v", err)
	}
	if got, _ := os.ReadFile(c.path()); string(got) != text {
		t.Error("the file was changed")
	}
	if !strings.Contains(c.Snippet("sprixel"), `"command"`) {
		t.Error("the snippet is missing")
	}
}

// Checks that opencode.jsonc is used when it is the only file.
func TestOpenCodeUsesJSONCFileWhenAlone(t *testing.T) {
	home := t.TempDir()
	writeConfig(t, home, "opencode.jsonc", `{"theme":"x"}`)
	c := OpenCode{Home: home}
	if _, err := c.Add("sprixel"); err != nil {
		t.Fatal(err)
	}
	if filepath.Base(c.path()) != "opencode.jsonc" {
		t.Errorf("path = %s", c.path())
	}
	if state, _ := c.State(); state != StateYes {
		t.Errorf("state = %v", state)
	}
}

// Checks the safety of the write: the file mode is kept, no temporary file is
// left behind, and a config that is not an object is refused.
func TestOpenCodeWriteKeepsModeAndLeavesNoTemporaryFile(t *testing.T) {
	home := t.TempDir()
	writeConfig(t, home, "opencode.json", `{"theme":"x"}`)
	c := OpenCode{Home: home}
	if _, err := c.Add("sprixel"); err != nil {
		t.Fatal(err)
	}
	entries, _ := os.ReadDir(filepath.Dir(c.path()))
	if len(entries) != 1 {
		t.Errorf("files left in the folder: %v", entries)
	}
	if info, _ := os.Stat(c.path()); info.Mode().Perm() != 0o600 && os.PathSeparator == '/' {
		t.Errorf("mode = %v, want 0600", info.Mode().Perm())
	}

	writeConfig(t, home, "opencode.json", `{"mcp": []}`)
	if _, err := c.Add("sprixel"); err == nil {
		t.Error("an mcp setting that is not an object must be refused")
	}
}

// Checks ByID: the two supported ids work and others do not.
func TestByID(t *testing.T) {
	for _, id := range IDs {
		if c, ok := ByID(id, t.TempDir(), nil); !ok || c.ID() != id {
			t.Errorf("ByID(%q)", id)
		}
	}
	if _, ok := ByID("cursor", "", nil); ok {
		t.Error("cursor is not supported")
	}
}

// A server that is registered but cannot connect makes `claude mcp get` exit
// with an error and print "Failed to connect". It is still registered: Add
// must not try to add it again and State must say so.
func TestClaudeRegisteredEvenWhenItFailsToConnect(t *testing.T) {
	run := func(name string, args ...string) (string, error) {
		return "sprixel:\n  Scope: User config\n  Status: ✘ Failed to connect\n  Type: stdio\n  Command: sprixel\n", errors.New("exit status 1")
	}
	c := Claude{Run: run}
	if state, err := c.State(); err != nil || state != StateYes {
		t.Errorf("State = %v, %v; want registered", state, err)
	}
	if result, err := c.Add("sprixel"); err != nil || result.Outcome != AlreadyRegistered {
		t.Errorf("Add = %+v, %v; want already registered", result, err)
	}
}
