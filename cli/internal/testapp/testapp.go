// Package testapp is a small fake of the Sprixel app for Go tests. It speaks
// the same HTTP and MCP rules as the real server (src-tauri/src/mcp): the
// token, the Host and Origin checks, one JSON-RPC request per POST and the
// Mcp-Session-Id header. It writes runtime.json and the token file in a data
// folder, like the app does. Only tests import it.
package testapp

import (
	"bytes"
	"encoding/base64"
	"encoding/json"
	"fmt"
	"image"
	"image/color"
	"image/png"
	"io"
	"net/http"
	"net/http/httptest"
	"os"
	"path/filepath"
	"strconv"
	"strings"
	"sync"
	"testing"
	"time"
)

// Token is the access token the fake app writes.
const Token = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"

// Version is the version the fake app reports.
const Version = "9.9.9"

// Call is one request the fake app received.
type Call struct {
	Method        string
	Authorization string
	Session       string
	Client        string // clientInfo.name of an initialize request
	ClientVersion string
	Body          string
}

// App is a running fake app.
type App struct {
	Dir string

	mu       sync.Mutex
	srv      *httptest.Server
	sessions map[string]string // session id -> client name
	calls    []Call
	nextID   int
	// Approved is the answer to tools/list and tools/call for every client.
	Approved bool
	// Block makes the tool "slow" wait until the app stops or restarts.
	release chan struct{}
}

// Start starts a fake app that writes its files in dir.
func Start(t *testing.T, dir string) *App {
	t.Helper()
	a := Prepare(t, dir)
	a.Open()
	return a
}

// Prepare makes a fake app that is closed: the token file exists (the app was
// run once) but nothing listens and there is no runtime.json. Open opens it.
func Prepare(t *testing.T, dir string) *App {
	t.Helper()
	a := &App{Dir: dir, Approved: true}
	if err := os.MkdirAll(dir, 0o700); err != nil {
		t.Fatal(err)
	}
	if err := os.WriteFile(filepath.Join(dir, "token"), []byte(Token), 0o600); err != nil {
		t.Fatal(err)
	}
	t.Cleanup(a.Stop)
	return a
}

// Open starts the server and writes runtime.json, like the app starting.
func (a *App) Open() { a.listen() }

func (a *App) listen() {
	a.mu.Lock()
	a.sessions = map[string]string{}
	a.release = make(chan struct{})
	a.srv = httptest.NewServer(http.HandlerFunc(a.serve))
	port := a.srv.Listener.Addr().(interface{ String() string }).String()
	a.mu.Unlock()
	port = port[strings.LastIndex(port, ":")+1:]
	text := fmt.Sprintf(`{"port": %s, "pid": %d, "version": %q}`, port, os.Getpid(), Version)
	if err := os.WriteFile(filepath.Join(a.Dir, "runtime.json"), []byte(text), 0o600); err != nil {
		panic(err)
	}
}

// Port is the port the app listens on now.
func (a *App) Port() int {
	a.mu.Lock()
	defer a.mu.Unlock()
	addr := a.srv.Listener.Addr().String()
	port, _ := strconv.Atoi(addr[strings.LastIndex(addr, ":")+1:])
	return port
}

// Stop closes the server and removes runtime.json, like the app does when it
// closes. Calls that wait for a slow tool end with a closed connection.
func (a *App) Stop() {
	a.mu.Lock()
	srv, release := a.srv, a.release
	a.srv, a.release = nil, nil
	a.mu.Unlock()
	if srv == nil {
		return
	}
	close(release)
	srv.CloseClientConnections()
	srv.Close()
	os.Remove(filepath.Join(a.Dir, "runtime.json"))
}

// Restart stops the app and starts it again on another port. The old
// sessions are gone, like after a real restart.
func (a *App) Restart() {
	a.Stop()
	a.listen()
}

// Calls returns the requests received so far.
func (a *App) Calls() []Call {
	a.mu.Lock()
	defer a.mu.Unlock()
	return append([]Call(nil), a.calls...)
}

// Clients returns the client names of the initialize requests, in order.
func (a *App) Clients() []string {
	var names []string
	for _, c := range a.Calls() {
		if c.Method == "initialize" {
			names = append(names, c.Client)
		}
	}
	return names
}

// OpenSessions returns how many sessions are open now.
func (a *App) OpenSessions() int {
	a.mu.Lock()
	defer a.mu.Unlock()
	return len(a.sessions)
}

// SetApproved changes whether clients are allowed.
func (a *App) SetApproved(ok bool) {
	a.mu.Lock()
	a.Approved = ok
	a.mu.Unlock()
}

func fail(w http.ResponseWriter, status int, code, message string) {
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(status)
	fmt.Fprintf(w, `{"error":{"code":%q,"message":%q}}`, code, message)
}

type rpc struct {
	ID     json.RawMessage `json:"id"`
	Method string          `json:"method"`
	Params struct {
		Name       string         `json:"name"`
		Arguments  map[string]any `json:"arguments"`
		ClientInfo struct {
			Name    string `json:"name"`
			Version string `json:"version"`
		} `json:"clientInfo"`
	} `json:"params"`
}

func (a *App) serve(w http.ResponseWriter, r *http.Request) {
	if r.Header.Get("Origin") != "" {
		fail(w, 403, "forbidden", "requests from web pages are not allowed")
		return
	}
	if r.Header.Get("Authorization") != "Bearer "+Token {
		fail(w, 401, "unauthorized", "send the access token as `Authorization: Bearer <token>`")
		return
	}
	if r.URL.Path != "/mcp" {
		fail(w, 404, "not_found", "the only path is /mcp")
		return
	}
	session := r.Header.Get("Mcp-Session-Id")
	if r.Method == http.MethodDelete {
		a.mu.Lock()
		delete(a.sessions, session)
		a.calls = append(a.calls, Call{Method: "DELETE", Authorization: r.Header.Get("Authorization"), Session: session})
		a.mu.Unlock()
		w.WriteHeader(200)
		return
	}
	if r.Method != http.MethodPost {
		fail(w, 405, "method_not_allowed", "use POST")
		return
	}
	body, _ := io.ReadAll(r.Body)
	var msg rpc
	if err := json.Unmarshal(body, &msg); err != nil {
		reply(w, nil, "", map[string]any{"jsonrpc": "2.0", "id": nil, "error": map[string]any{"code": -32700, "message": "the body is not valid JSON"}})
		return
	}
	a.mu.Lock()
	a.calls = append(a.calls, Call{
		Method: msg.Method, Authorization: r.Header.Get("Authorization"), Session: session,
		Client: msg.Params.ClientInfo.Name, ClientVersion: msg.Params.ClientInfo.Version, Body: string(body),
	})
	client, known := a.sessions[session]
	approved, release := a.Approved, a.release
	a.mu.Unlock()

	if len(msg.ID) == 0 || string(msg.ID) == "null" {
		w.WriteHeader(202)
		return
	}
	id := msg.ID
	result := func(v any) { reply(w, id, "", map[string]any{"jsonrpc": "2.0", "id": id, "result": v}) }
	switch msg.Method {
	case "initialize":
		a.mu.Lock()
		a.nextID++
		sid := "session-" + strconv.Itoa(a.nextID)
		a.sessions[sid] = msg.Params.ClientInfo.Name
		a.mu.Unlock()
		reply(w, id, sid, map[string]any{"jsonrpc": "2.0", "id": id, "result": map[string]any{
			"protocolVersion": "2025-06-18",
			"capabilities":    map[string]any{"tools": map[string]any{}},
			"serverInfo":      map[string]any{"name": "sprixel", "version": Version},
		}})
	case "ping":
		result(map[string]any{})
	case "tools/list", "tools/call":
		if !known {
			w.Header().Set("Content-Type", "application/json")
			w.WriteHeader(400)
			fmt.Fprintf(w, `{"jsonrpc":"2.0","id":%s,"error":{"code":-32002,"message":"no session: send `+"`initialize`"+` first and use the Mcp-Session-Id header"}}`, id)
			return
		}
		if msg.Method == "tools/list" {
			if !approved {
				result(map[string]any{"tools": []any{}})
				return
			}
			result(map[string]any{"tools": Tools})
			return
		}
		result(a.callTool(msg.Params.Name, msg.Params.Arguments, client, approved, release))
	default:
		reply(w, id, "", map[string]any{"jsonrpc": "2.0", "id": id, "error": map[string]any{"code": -32601, "message": "unknown method"}})
	}
}

func reply(w http.ResponseWriter, _ json.RawMessage, session string, body any) {
	w.Header().Set("Content-Type", "application/json")
	if session != "" {
		w.Header().Set("Mcp-Session-Id", session)
	}
	json.NewEncoder(w).Encode(body)
}

// Tools is the tool list of the fake app.
var Tools = []map[string]any{
	{"name": "list_sprites", "description": "Lists the sprites.", "inputSchema": map[string]any{
		"type": "object", "properties": map[string]any{"folder": map[string]any{"type": "string"}}}},
	{"name": "get_sprite", "description": "Reads one sprite.", "inputSchema": map[string]any{
		"type": "object", "required": []string{"name"}, "properties": map[string]any{
			"name": map[string]any{"type": "string"}, "frame": map[string]any{"type": "integer"}}}},
	{"name": "set_pixels", "description": "Sets pixels.", "inputSchema": map[string]any{
		"type": "object", "properties": map[string]any{
			"pixels": map[string]any{"type": "array"}, "dry": map[string]any{"type": "boolean"}}}},
	{"name": "render_preview", "description": "Renders a frame as a PNG image.", "inputSchema": map[string]any{
		"type": "object", "properties": map[string]any{
			"name": map[string]any{"type": "string"}, "frame": map[string]any{"type": "integer"}}}},
	{"name": "slow", "description": "Waits until the app closes (tests only).", "inputSchema": map[string]any{"type": "object"}},
}

func (a *App) callTool(name string, args map[string]any, client string, approved bool, release chan struct{}) map[string]any {
	text := func(v any, isError bool) map[string]any {
		bytes, _ := json.Marshal(v)
		return map[string]any{"content": []any{map[string]any{"type": "text", "text": string(bytes)}}, "isError": isError}
	}
	if !approved {
		return text(map[string]any{"code": "awaiting_approval", "message": "the user has not allowed this client yet; ask them to press Allow in the Sprixel app, then try again"}, true)
	}
	switch name {
	case "render_preview":
		// A real 1x1 PNG, then the info text, like the app sends it.
		var png bytes.Buffer
		pixel := image.NewRGBA(image.Rect(0, 0, 1, 1))
		pixel.Set(0, 0, color.RGBA{R: 255, A: 255})
		pngEncode(&png, pixel)
		frame := args["frame"]
		if frame == nil {
			frame = 1 // the real tool starts at frame 1
		}
		info, _ := json.Marshal(map[string]any{"name": args["name"], "frame": frame, "revision": 7})
		return map[string]any{"isError": false, "content": []any{
			map[string]any{"type": "image", "data": base64.StdEncoding.EncodeToString(png.Bytes()), "mimeType": "image/png"},
			map[string]any{"type": "text", "text": string(info)},
		}}
	case "list_sprites":
		return text(map[string]any{"revision": 1, "sprites": []any{map[string]any{"name": "goblin", "width": 16, "height": 16}}}, false)
	case "get_sprite":
		if args["name"] != "goblin" {
			return text(map[string]any{"code": "not_found", "message": fmt.Sprintf("there is no sprite %v", args["name"])}, true)
		}
		return text(map[string]any{"name": "goblin", "frame": args["frame"], "client": client}, false)
	case "set_pixels":
		// wait_ms lets a test make early calls slower than late ones.
		if ms, ok := args["wait_ms"].(float64); ok {
			time.Sleep(time.Duration(ms) * time.Millisecond)
		}
		return text(map[string]any{"args": args}, false)
	case "slow":
		<-release
		return text(map[string]any{"code": "internal_error", "message": "the app stopped"}, true)
	}
	return text(map[string]any{"code": "unknown_tool", "message": fmt.Sprintf("there is no tool %q", name)}, true)
}

func pngEncode(w *bytes.Buffer, img image.Image) { png.Encode(w, img) }
