// Package bridge connects an AI tool to the running Sprixel app. The AI tool
// speaks MCP over stdin and stdout (one JSON-RPC message per line); the app
// speaks MCP over HTTP on 127.0.0.1 with a token. The bridge copies messages
// from one to the other and does not interpret them, except `initialize`,
// where it makes sure the app gets a readable name of the AI client.
//
// Rules: stdout holds protocol messages and nothing else; every log goes to
// the log writer (stderr); the token is only ever sent in the Authorization
// header and is never logged.
package bridge

import (
	"bufio"
	"bytes"
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"net"
	"net/http"
	"strings"
	"sync"
	"time"

	"github.com/Javier-GarciaP/sprixel/cli/internal/runtime"
)

// Config says where the bridge reads, writes and finds the app.
type Config struct {
	// Dir is the Sprixel data folder (~/.sprixel).
	Dir string
	// In and Out are the MCP stream. Out only gets protocol messages.
	In  io.Reader
	Out io.Writer
	// Log gets every message for people (stderr). Never the token.
	Log io.Writer
	// RequestTimeout is how long one call may take (default 5 minutes).
	RequestTimeout time.Duration
	// Start starts the app when it is not running. It returns a channel that
	// gets one value when the app process ends (nil if it ended well). When
	// Start is nil the bridge only reports that the app is closed.
	Start func() (exited <-chan error, err error)
	// StartTimeout is how long to wait for the server of a new app (default
	// 20 seconds). PollEvery is how often runtime.json is checked (default
	// 100 milliseconds).
	StartTimeout time.Duration
	PollEvery    time.Duration
}

// Bridge is one running bridge.
type Bridge struct {
	cfg    Config
	client *http.Client
	wmu    sync.Mutex // one writer to Out at a time

	mu         sync.Mutex
	session    string          // Mcp-Session-Id of the app
	startMu    sync.Mutex      // one start of the app at a time
	initParams json.RawMessage // the initialize params, to start again after a restart
}

// New makes a bridge. Call Run to use it.
func New(cfg Config) *Bridge {
	if cfg.RequestTimeout == 0 {
		cfg.RequestTimeout = 5 * time.Minute
	}
	if cfg.StartTimeout == 0 {
		cfg.StartTimeout = 20 * time.Second
	}
	if cfg.PollEvery == 0 {
		cfg.PollEvery = 100 * time.Millisecond
	}
	if cfg.Log == nil {
		cfg.Log = io.Discard
	}
	return &Bridge{
		cfg: cfg,
		// The app closes the connection after each answer.
		client: &http.Client{Transport: &http.Transport{DisableKeepAlives: true}},
	}
}

// Run copies messages until the input closes, then waits for the calls in
// progress and returns. It never stops the app.
func (b *Bridge) Run(ctx context.Context) error {
	reader := bufio.NewReader(b.cfg.In)
	var wg sync.WaitGroup
	for {
		line, err := reader.ReadBytes('\n')
		if text := bytes.TrimSpace(line); len(text) > 0 {
			if isInitialize(text) {
				// The session must exist before the next message is sent, so
				// this one is answered first.
				b.handle(ctx, text)
				continue
			}
			wg.Add(1)
			// One goroutine per message, so a slow tool does not block the rest.
			go func(message []byte) {
				defer wg.Done()
				b.handle(ctx, message)
			}(text)
		}
		if err != nil {
			wg.Wait()
			b.endSession()
			if errors.Is(err, io.EOF) {
				return nil
			}
			return err
		}
	}
}

func isInitialize(line []byte) bool {
	var m message
	return json.Unmarshal(line, &m) == nil && m.Method == "initialize"
}

func (b *Bridge) logf(format string, args ...any) {
	fmt.Fprintf(b.cfg.Log, "sprixel mcp: "+format+"\n", args...)
}

// message is the part of a JSON-RPC message the bridge needs.
type message struct {
	ID     json.RawMessage `json:"id"`
	Method string          `json:"method"`
}

func (m message) isRequest() bool { return len(m.ID) > 0 && string(m.ID) != "null" }

func (b *Bridge) handle(ctx context.Context, line []byte) {
	var m message
	if err := json.Unmarshal(line, &m); err != nil {
		b.write(errorReply(nil, codeParse, "the message is not valid JSON"))
		return
	}
	if m.Method == "initialize" {
		line = b.prepareInitialize(line)
	}
	ctx, cancel := context.WithTimeout(ctx, b.cfg.RequestTimeout)
	defer cancel()

	answer, err := b.forward(ctx, m, line)
	switch {
	case err != nil && m.isRequest():
		b.logf("%v", err)
		b.write(errorReply(m.ID, codeApp, err.Error()))
	case err != nil:
		b.logf("a notification was not delivered: %v", err)
	case m.isRequest():
		b.write(answer)
	}
}

// JSON-RPC error codes the bridge uses.
const (
	codeParse = -32700
	codeApp   = -32000 // the app cannot be reached or refused the call
)

// forward sends one message to the app and returns the answer as one line.
// If the app is closed it is started first. If the app was restarted since
// `initialize`, the session is started again and the message is sent once more
// (the app never ran it, so nothing is repeated).
func (b *Bridge) forward(ctx context.Context, m message, line []byte) ([]byte, error) {
	if err := b.ensureApp(ctx); err != nil {
		return nil, err
	}
	session := b.currentSession()
	status, newSession, body, err := b.post(ctx, line, session)
	if err != nil {
		return nil, err
	}
	if m.Method != "initialize" && isNoSession(status, body) && b.hasInitialize() {
		b.logf("the app restarted, starting a new session")
		if err := b.reinitialize(ctx, session); err != nil {
			return nil, err
		}
		status, newSession, body, err = b.post(ctx, line, b.currentSession())
		if err != nil {
			return nil, err
		}
	}
	if m.Method == "initialize" && newSession != "" {
		b.mu.Lock()
		b.session = newSession
		b.mu.Unlock()
	}
	return answerFor(m, status, body)
}

// isNoSession tells the app's answer "send initialize first": it was given a
// session id it does not know (the app restarted) and ran nothing.
func isNoSession(status int, body []byte) bool {
	if status != http.StatusBadRequest {
		return false
	}
	var parsed struct {
		Error struct {
			Code int `json:"code"`
		} `json:"error"`
	}
	return json.Unmarshal(body, &parsed) == nil && parsed.Error.Code == -32002
}

func (b *Bridge) hasInitialize() bool {
	b.mu.Lock()
	defer b.mu.Unlock()
	return b.initParams != nil
}

// reinitialize starts a new session with the first initialize of the AI tool.
// stale is the session that failed: if another call already replaced it,
// nothing is done.
func (b *Bridge) reinitialize(ctx context.Context, stale string) error {
	b.startMu.Lock()
	defer b.startMu.Unlock()
	if b.currentSession() != stale {
		return nil
	}
	b.mu.Lock()
	params := b.initParams
	b.mu.Unlock()
	request, _ := json.Marshal(map[string]any{"jsonrpc": "2.0", "id": "sprixel-reconnect", "method": "initialize", "params": params})
	status, session, body, err := b.post(ctx, request, "")
	if err != nil {
		return err
	}
	if session == "" {
		_, err := answerFor(message{ID: json.RawMessage(`"sprixel-reconnect"`)}, status, body)
		if err == nil {
			err = errors.New("the Sprixel app did not start a new session")
		}
		return err
	}
	b.mu.Lock()
	b.session = session
	b.mu.Unlock()
	notice, _ := json.Marshal(map[string]any{"jsonrpc": "2.0", "method": "notifications/initialized"})
	b.post(ctx, notice, session)
	return nil
}

// ensureApp makes sure the app server is reachable. If not, it starts the app
// and waits for the server (StartTimeout). Only one start happens at a time.
func (b *Bridge) ensureApp(ctx context.Context) error {
	if info, err := runtime.ReadInfo(b.cfg.Dir); err == nil && runtime.Reachable(info) {
		return nil
	}
	b.startMu.Lock()
	defer b.startMu.Unlock()
	if info, err := runtime.ReadInfo(b.cfg.Dir); err == nil && runtime.Reachable(info) {
		return nil // another call started it while this one waited
	}
	if b.cfg.Start == nil {
		return runtime.ErrNotRunning
	}
	b.logf("the Sprixel app is not running, starting it")
	exited, err := b.cfg.Start()
	if err != nil {
		return err
	}
	_, err = runtime.WaitReady(ctx, b.cfg.Dir, exited, b.cfg.StartTimeout, b.cfg.PollEvery)
	return err
}

// answerFor turns what the app answered into a line for the AI tool.
func answerFor(m message, status int, body []byte) ([]byte, error) {
	body = bytes.TrimSpace(body)
	if !m.isRequest() {
		return nil, nil
	}
	var parsed struct {
		JSONRPC string `json:"jsonrpc"`
		Error   *struct {
			Code    string `json:"code"`
			Message string `json:"message"`
		} `json:"error"`
	}
	if json.Unmarshal(body, &parsed) == nil && parsed.JSONRPC != "" {
		var compact bytes.Buffer
		if json.Compact(&compact, body) == nil {
			return compact.Bytes(), nil
		}
	}
	// An HTTP level refusal: {"error":{"code":"unauthorized","message":"..."}}.
	if parsed.Error != nil && parsed.Error.Message != "" {
		return nil, fmt.Errorf("the Sprixel app refused the request (%s): %s", parsed.Error.Code, parsed.Error.Message)
	}
	return nil, fmt.Errorf("the Sprixel app answered with HTTP status %d and no MCP message", status)
}

// post sends one body to the app. It reads runtime.json and the token every
// time, so a restarted app (a new port) is found without any extra step.
func (b *Bridge) post(ctx context.Context, body []byte, session string) (status int, newSession string, answer []byte, err error) {
	info, err := runtime.ReadInfo(b.cfg.Dir)
	if err != nil {
		return 0, "", nil, err
	}
	token, err := runtime.ReadToken(b.cfg.Dir)
	if err != nil {
		return 0, "", nil, err
	}
	request, err := http.NewRequestWithContext(ctx, http.MethodPost, "http://"+info.Addr()+"/mcp", bytes.NewReader(body))
	if err != nil {
		return 0, "", nil, err
	}
	request.Header.Set("Content-Type", "application/json")
	request.Header.Set("Authorization", "Bearer "+token)
	if session != "" {
		request.Header.Set("Mcp-Session-Id", session)
	}
	response, err := b.client.Do(request)
	if err != nil {
		return 0, "", nil, describe(err)
	}
	defer response.Body.Close()
	answer, err = io.ReadAll(io.LimitReader(response.Body, 64<<20))
	if err != nil {
		return 0, "", nil, describe(err)
	}
	return response.StatusCode, response.Header.Get("Mcp-Session-Id"), answer, nil
}

// describe makes a network error readable. Go's own text can include the
// address but never the token (the token is only in a header).
func describe(err error) error {
	var timeout net.Error
	switch {
	case errors.Is(err, context.DeadlineExceeded) || (errors.As(err, &timeout) && timeout.Timeout()):
		return errors.New("the Sprixel app did not answer in time")
	case errors.Is(err, context.Canceled):
		return errors.New("the call was cancelled")
	}
	return fmt.Errorf("lost the connection to the Sprixel app (%s); the call was not confirmed", shortReason(err))
}

func shortReason(err error) string {
	text := err.Error()
	if i := strings.LastIndex(text, ": "); i >= 0 {
		text = text[i+2:]
	}
	return text
}

func (b *Bridge) currentSession() string {
	b.mu.Lock()
	defer b.mu.Unlock()
	return b.session
}

// endSession tells the app the session is over. It is best effort and short.
func (b *Bridge) endSession() {
	session := b.currentSession()
	if session == "" {
		return
	}
	info, err := runtime.ReadInfo(b.cfg.Dir)
	if err != nil {
		return
	}
	token, err := runtime.ReadToken(b.cfg.Dir)
	if err != nil {
		return
	}
	ctx, cancel := context.WithTimeout(context.Background(), 2*time.Second)
	defer cancel()
	request, err := http.NewRequestWithContext(ctx, http.MethodDelete, "http://"+info.Addr()+"/mcp", nil)
	if err != nil {
		return
	}
	request.Header.Set("Authorization", "Bearer "+token)
	request.Header.Set("Mcp-Session-Id", session)
	if response, err := b.client.Do(request); err == nil {
		response.Body.Close()
	}
}

// write sends one message to the AI tool: one line.
func (b *Bridge) write(line []byte) {
	b.wmu.Lock()
	defer b.wmu.Unlock()
	b.cfg.Out.Write(append(bytes.TrimSpace(line), '\n'))
}

// errorReply is a JSON-RPC error for a request.
func errorReply(id json.RawMessage, code int, text string) []byte {
	if len(id) == 0 {
		id = json.RawMessage("null")
	}
	out, _ := json.Marshal(map[string]any{
		"jsonrpc": "2.0",
		"id":      id,
		"error":   map[string]any{"code": code, "message": "sprixel: " + text},
	})
	return out
}

// clientNames turns the ids that AI tools send into the names people know.
var clientNames = map[string]string{
	"claude-code": "Claude Code",
	"claude-ai":   "Claude",
	"opencode":    "OpenCode",
}

// prepareInitialize remembers the initialize call (to start the session
// again after the app restarts) and gives the app a readable client name.
func (b *Bridge) prepareInitialize(line []byte) []byte {
	var whole map[string]json.RawMessage
	if json.Unmarshal(line, &whole) != nil {
		return line
	}
	var params map[string]json.RawMessage
	if json.Unmarshal(whole["params"], &params) != nil || params == nil {
		params = map[string]json.RawMessage{}
	}
	var info map[string]any
	if json.Unmarshal(params["clientInfo"], &info) != nil || info == nil {
		info = map[string]any{}
	}
	name, _ := info["name"].(string)
	switch pretty, known := clientNames[strings.ToLower(strings.TrimSpace(name))]; {
	case known:
		info["name"] = pretty
	case strings.TrimSpace(name) == "":
		info["name"] = "MCP client"
	}
	params["clientInfo"], _ = json.Marshal(info)
	whole["params"], _ = json.Marshal(params)
	out, err := json.Marshal(whole)
	if err != nil {
		return line
	}
	b.mu.Lock()
	b.initParams = whole["params"]
	b.mu.Unlock()
	return out
}
