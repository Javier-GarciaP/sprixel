package bridge

import (
	"bufio"
	"context"
	"encoding/json"
	"io"
	"strings"
	"testing"
	"time"

	"github.com/Javier-GarciaP/sprixel/cli/internal/testapp"
)

// session is an AI tool talking to a running bridge through pipes.
type session struct {
	t     *testing.T
	in    io.WriteCloser
	lines chan string
	done  chan error
	log   *strings.Builder
}

func startBridge(t *testing.T, dir string) *session {
	t.Helper()
	inReader, inWriter := io.Pipe()
	outReader, outWriter := io.Pipe()
	logs := &strings.Builder{}
	s := &session{t: t, in: inWriter, lines: make(chan string, 64), done: make(chan error, 1), log: logs}
	b := New(Config{Dir: dir, In: inReader, Out: outWriter, Log: logs, RequestTimeout: 10 * time.Second})
	go func() {
		err := b.Run(context.Background())
		outWriter.Close()
		s.done <- err
	}()
	go func() {
		scanner := bufio.NewScanner(outReader)
		scanner.Buffer(make([]byte, 1<<20), 1<<24)
		for scanner.Scan() {
			s.lines <- scanner.Text()
		}
		close(s.lines)
	}()
	t.Cleanup(func() { inWriter.Close() })
	return s
}

func (s *session) send(text string) {
	s.t.Helper()
	if _, err := io.WriteString(s.in, text+"\n"); err != nil {
		s.t.Fatal(err)
	}
}

// reply waits for the next line from the bridge and parses it.
func (s *session) reply() map[string]any {
	s.t.Helper()
	select {
	case line, ok := <-s.lines:
		if !ok {
			s.t.Fatal("the bridge closed its output")
		}
		var m map[string]any
		if err := json.Unmarshal([]byte(line), &m); err != nil {
			s.t.Fatalf("stdout line is not JSON: %q", line)
		}
		return m
	case <-time.After(10 * time.Second):
		s.t.Fatal("no answer from the bridge")
		return nil
	}
}

const initialize = `{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"claude-code","version":"2.0"}}}`

// Checks the scenario "List tools through the bridge": a client that
// initializes and asks for the tools gets the tool list of the app.
func TestListToolsThroughTheBridge(t *testing.T) {
	app := testapp.Start(t, t.TempDir())
	s := startBridge(t, app.Dir)

	s.send(initialize)
	if init := s.reply(); init["id"] != float64(1) || init["result"] == nil {
		t.Fatalf("initialize answer: %v", init)
	}
	s.send(`{"jsonrpc":"2.0","method":"notifications/initialized"}`)
	s.send(`{"jsonrpc":"2.0","id":2,"method":"tools/list"}`)
	list := s.reply()

	tools := list["result"].(map[string]any)["tools"].([]any)
	if len(tools) != len(testapp.Tools) {
		t.Fatalf("got %d tools, want %d", len(tools), len(testapp.Tools))
	}
	if first := tools[0].(map[string]any)["name"]; first != "list_sprites" {
		t.Errorf("first tool = %v", first)
	}
}

// Checks that a tool call works and that the answer keeps the request id.
func TestToolCallThroughTheBridge(t *testing.T) {
	app := testapp.Start(t, t.TempDir())
	s := startBridge(t, app.Dir)
	s.send(initialize)
	s.reply()

	s.send(`{"jsonrpc":"2.0","id":"abc","method":"tools/call","params":{"name":"list_sprites","arguments":{}}}`)
	got := s.reply()

	if got["id"] != "abc" {
		t.Errorf("id = %v, want abc", got["id"])
	}
	text := got["result"].(map[string]any)["content"].([]any)[0].(map[string]any)["text"].(string)
	if !strings.Contains(text, "goblin") {
		t.Errorf("tool text = %q", text)
	}
}

// Checks the scenario "No token argument": the bridge needs no arguments
// (Config has none for the token); it sends the token of the file in the
// Authorization header of every request, and never writes it to the log.
func TestTokenFromFileIsSentAndNotLogged(t *testing.T) {
	app := testapp.Start(t, t.TempDir())
	s := startBridge(t, app.Dir)
	s.send(initialize)
	s.reply()
	s.send(`{"jsonrpc":"2.0","id":2,"method":"tools/list"}`)
	s.reply()

	for _, call := range app.Calls() {
		if call.Authorization != "Bearer "+testapp.Token {
			t.Errorf("%s: Authorization = %q", call.Method, call.Authorization)
		}
	}
	if len(app.Calls()) < 2 {
		t.Fatal("the app saw no request")
	}
	if strings.Contains(s.log.String(), testapp.Token) {
		t.Error("the token was written to the log")
	}
}

// Checks the scenario "Client name": Claude Code sends the id "claude-code"
// and the app sees "Claude Code"; a name the bridge does not know is kept;
// no name becomes a generic one. The version always passes through.
func TestClientNameIsPassedToTheApp(t *testing.T) {
	cases := map[string]string{
		`{"name":"claude-code","version":"2.0"}`: "Claude Code",
		`{"name":"opencode","version":"2.0"}`:    "OpenCode",
		`{"name":"my-tool","version":"2.0"}`:     "my-tool",
		`{"version":"2.0"}`:                      "MCP client",
	}
	for info, want := range cases {
		app := testapp.Start(t, t.TempDir())
		s := startBridge(t, app.Dir)
		s.send(`{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"clientInfo":` + info + `}}`)
		s.reply()

		names := app.Clients()
		if len(names) != 1 || names[0] != want {
			t.Errorf("%s: app saw %v, want %q", info, names, want)
		}
		if v := app.Calls()[0].ClientVersion; v != "2.0" {
			t.Errorf("%s: version = %q", info, v)
		}
	}
}

// Checks that the Mcp-Session-Id from initialize is used on later requests.
func TestSessionIdIsUsedAfterInitialize(t *testing.T) {
	app := testapp.Start(t, t.TempDir())
	s := startBridge(t, app.Dir)
	s.send(initialize)
	s.reply()
	s.send(`{"jsonrpc":"2.0","id":2,"method":"tools/list"}`)
	s.reply()

	calls := app.Calls()
	if calls[0].Session != "" || calls[len(calls)-1].Session == "" {
		t.Errorf("sessions: %q then %q", calls[0].Session, calls[len(calls)-1].Session)
	}
}

// Checks the scenario "AI tool closes": when the input closes, the bridge
// answers what is pending, exits without an error, and the app keeps running.
func TestBridgeExitsWhenInputClosesAndAppKeepsRunning(t *testing.T) {
	app := testapp.Start(t, t.TempDir())
	s := startBridge(t, app.Dir)
	s.send(initialize)
	s.reply()

	s.in.Close()
	select {
	case err := <-s.done:
		if err != nil {
			t.Errorf("Run returned %v", err)
		}
	case <-time.After(10 * time.Second):
		t.Fatal("the bridge did not exit")
	}
	// The app still answers.
	other := startBridge(t, app.Dir)
	other.send(initialize)
	if other.reply()["result"] == nil {
		t.Error("the app does not answer after the bridge closed")
	}
}

// Checks that a request that is not valid JSON gets a JSON-RPC parse error
// instead of silence or a crash.
func TestInvalidJSONGetsAParseError(t *testing.T) {
	app := testapp.Start(t, t.TempDir())
	s := startBridge(t, app.Dir)
	s.send(`this is not json`)
	got := s.reply()
	if got["error"].(map[string]any)["code"] != float64(codeParse) {
		t.Errorf("answer: %v", got)
	}
}

// Checks that an error of the app that is not JSON-RPC (a refused token)
// becomes a JSON-RPC error with the id of the request and an explanation.
func TestHTTPRefusalBecomesAnMCPError(t *testing.T) {
	app := testapp.Start(t, t.TempDir())
	s := startBridge(t, app.Dir)
	if err := writeFile(app.Dir+"/token", "wrong"); err != nil {
		t.Fatal(err)
	}
	s.send(initialize)
	got := s.reply()

	e := got["error"].(map[string]any)
	if got["id"] != float64(1) || !strings.Contains(e["message"].(string), "unauthorized") {
		t.Errorf("answer: %v", got)
	}
	if strings.Contains(e["message"].(string), "wrong") {
		t.Error("the message repeats the token")
	}
}
