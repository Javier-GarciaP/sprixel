// Package appclient is a small MCP client for the commands that talk to the
// app from the terminal (status, doctor, tools, tool). It follows the same
// rules as any AI client: the token from the shared folder, an initialize call
// with a client name, and the approval of the person in the app.
package appclient

import (
	"bytes"
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"net/http"
	"time"

	"github.com/Javier-GarciaP/sprixel/cli/internal/runtime"
)

// ClientName is how the app lists this command.
const ClientName = "Sprixel CLI"

// ErrNotRunning means the app is closed or does not answer.
var ErrNotRunning = errors.New("the Sprixel app is not running")

// ServerError is a refusal of the app: an HTTP error, or a JSON-RPC error.
type ServerError struct {
	Status  int
	Code    string
	Message string
}

func (e *ServerError) Error() string {
	if e.Code != "" {
		return fmt.Sprintf("the Sprixel app refused the request (%s): %s", e.Code, e.Message)
	}
	return "the Sprixel app refused the request: " + e.Message
}

// Client talks to the app. The zero value is not usable: set Dir.
type Client struct {
	Dir     string
	Version string // version of the command, sent as the client version
	HTTP    *http.Client

	session string
}

func (c *Client) http() *http.Client {
	if c.HTTP != nil {
		return c.HTTP
	}
	return &http.Client{Timeout: 5 * time.Minute, Transport: &http.Transport{DisableKeepAlives: true}}
}

// post sends one JSON-RPC request and returns the "result".
func (c *Client) post(ctx context.Context, method string, params any, withSession bool) (json.RawMessage, http.Header, error) {
	info, err := runtime.ReadInfo(c.Dir)
	if errors.Is(err, runtime.ErrNotRunning) {
		return nil, nil, ErrNotRunning
	}
	if err != nil {
		return nil, nil, err
	}
	token, err := runtime.ReadToken(c.Dir)
	if err != nil {
		return nil, nil, err
	}
	body, _ := json.Marshal(map[string]any{"jsonrpc": "2.0", "id": 1, "method": method, "params": params})
	request, err := http.NewRequestWithContext(ctx, http.MethodPost, "http://"+info.Addr()+"/mcp", bytes.NewReader(body))
	if err != nil {
		return nil, nil, err
	}
	request.Header.Set("Content-Type", "application/json")
	request.Header.Set("Authorization", "Bearer "+token)
	if withSession && c.session != "" {
		request.Header.Set("Mcp-Session-Id", c.session)
	}
	response, err := c.http().Do(request)
	if err != nil {
		if errors.Is(err, context.DeadlineExceeded) {
			return nil, nil, errors.New("the Sprixel app did not answer in time")
		}
		// Nothing listens on the port: runtime.json is left from an app that closed.
		return nil, nil, ErrNotRunning
	}
	defer response.Body.Close()
	data, err := io.ReadAll(io.LimitReader(response.Body, 64<<20))
	if err != nil {
		return nil, nil, errors.New("lost the connection to the Sprixel app")
	}
	var parsed struct {
		Result json.RawMessage `json:"result"`
		Error  json.RawMessage `json:"error"`
	}
	if json.Unmarshal(data, &parsed) != nil {
		return nil, nil, &ServerError{Status: response.StatusCode, Message: fmt.Sprintf("answered with HTTP status %d and no JSON", response.StatusCode)}
	}
	if parsed.Error != nil {
		var e struct {
			Code    json.RawMessage `json:"code"`
			Message string          `json:"message"`
		}
		json.Unmarshal(parsed.Error, &e)
		code := string(bytes.Trim(e.Code, `"`))
		if code == "-32002" || code == "-32000" || code == "-32601" {
			code = ""
		}
		return nil, nil, &ServerError{Status: response.StatusCode, Code: code, Message: e.Message}
	}
	return parsed.Result, response.Header, nil
}

// Ping checks that the app answers with this token. It opens no session, so
// the app does not ask the person to allow anything.
func (c *Client) Ping(ctx context.Context) error {
	_, _, err := c.post(ctx, "ping", map[string]any{}, false)
	return err
}

// Initialize starts a session as the Sprixel CLI client.
func (c *Client) Initialize(ctx context.Context) error {
	_, header, err := c.post(ctx, "initialize", map[string]any{
		"protocolVersion": "2025-06-18",
		"capabilities":    map[string]any{},
		"clientInfo":      map[string]any{"name": ClientName, "version": c.Version},
	}, false)
	if err != nil {
		return err
	}
	c.session = header.Get("Mcp-Session-Id")
	return nil
}

// Close ends the session of this client in the app. It is safe to call when
// no session was opened, and it never fails the command: the app drops idle
// sessions by itself, so a lost DELETE only delays the cleanup.
func (c *Client) Close() {
	if c.session == "" {
		return
	}
	session := c.session
	c.session = ""
	info, err := runtime.ReadInfo(c.Dir)
	if err != nil {
		return
	}
	token, err := runtime.ReadToken(c.Dir)
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
	if response, err := c.http().Do(request); err == nil {
		response.Body.Close()
	}
}

// Tool is a tool of the app.
type Tool struct {
	Name        string          `json:"name"`
	Description string          `json:"description"`
	InputSchema json.RawMessage `json:"inputSchema"`
}

// ListTools returns the tools. It is empty until the person allows the client.
func (c *Client) ListTools(ctx context.Context) ([]Tool, error) {
	if c.session == "" {
		if err := c.Initialize(ctx); err != nil {
			return nil, err
		}
	}
	result, _, err := c.post(ctx, "tools/list", map[string]any{}, true)
	if err != nil {
		return nil, err
	}
	var list struct {
		Tools []Tool `json:"tools"`
	}
	if err := json.Unmarshal(result, &list); err != nil {
		return nil, errors.New("the app sent a tool list that cannot be read")
	}
	return list.Tools, nil
}

// Content is one piece of a tool result.
type Content struct {
	Type     string `json:"type"`
	Text     string `json:"text,omitempty"`
	Data     string `json:"data,omitempty"`
	MimeType string `json:"mimeType,omitempty"`
}

// Result is what a tool call returns. IsError is the error of the tool itself.
type Result struct {
	Content []Content `json:"content"`
	IsError bool      `json:"isError"`
}

// CallTool runs a tool.
func (c *Client) CallTool(ctx context.Context, name string, args map[string]any) (Result, error) {
	if c.session == "" {
		if err := c.Initialize(ctx); err != nil {
			return Result{}, err
		}
	}
	if args == nil {
		args = map[string]any{}
	}
	result, _, err := c.post(ctx, "tools/call", map[string]any{"name": name, "arguments": args}, true)
	if err != nil {
		return Result{}, err
	}
	var out Result
	if err := json.Unmarshal(result, &out); err != nil {
		return Result{}, errors.New("the app sent a tool result that cannot be read")
	}
	return out, nil
}
