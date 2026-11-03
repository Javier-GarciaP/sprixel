package appclient

import (
	"context"
	"errors"
	"os"
	"path/filepath"
	"strings"
	"testing"

	"github.com/Javier-GarciaP/sprixel/cli/internal/testapp"
)

func client(dir string) *Client { return &Client{Dir: dir, Version: "1.0.0"} }

// Checks the whole path: ping, initialize as "Sprixel CLI" with the version of
// the command, list the tools, and call one.
func TestListAndCallThroughTheApp(t *testing.T) {
	app := testapp.Start(t, t.TempDir())
	c := client(app.Dir)
	ctx := context.Background()

	if err := c.Ping(ctx); err != nil {
		t.Fatal(err)
	}
	tools, err := c.ListTools(ctx)
	if err != nil || len(tools) != len(testapp.Tools) {
		t.Fatalf("tools = %d, %v", len(tools), err)
	}
	result, err := c.CallTool(ctx, "list_sprites", nil)
	if err != nil || result.IsError || !strings.Contains(result.Content[0].Text, "goblin") {
		t.Errorf("result = %+v, %v", result, err)
	}
	if names := app.Clients(); len(names) != 1 || names[0] != ClientName {
		t.Errorf("clients = %v", names)
	}
	if app.Calls()[1].ClientVersion != "1.0.0" {
		t.Errorf("version = %q", app.Calls()[1].ClientVersion)
	}
}

// Checks that ping opens no session: the app must not ask the person to allow
// a client just because of "status" or "doctor".
func TestPingDoesNotInitialize(t *testing.T) {
	app := testapp.Start(t, t.TempDir())
	if err := client(app.Dir).Ping(context.Background()); err != nil {
		t.Fatal(err)
	}
	if len(app.Clients()) != 0 {
		t.Error("ping started a session")
	}
}

// Checks that a closed app, and a runtime.json left by a crashed app, are both
// ErrNotRunning.
func TestNotRunning(t *testing.T) {
	dir := t.TempDir()
	if err := client(dir).Ping(context.Background()); !errors.Is(err, ErrNotRunning) {
		t.Errorf("no file: %v", err)
	}
	os.WriteFile(filepath.Join(dir, "token"), []byte(testapp.Token), 0o600)
	os.WriteFile(filepath.Join(dir, "runtime.json"), []byte(`{"port":1,"pid":1}`), 0o600)
	if err := client(dir).Ping(context.Background()); !errors.Is(err, ErrNotRunning) {
		t.Errorf("stale file: %v", err)
	}
}

// Checks that a wrong token is an explained refusal and the message does not
// repeat the token.
func TestWrongTokenIsRefused(t *testing.T) {
	app := testapp.Start(t, t.TempDir())
	os.WriteFile(filepath.Join(app.Dir, "token"), []byte("secret-wrong-token"), 0o600)

	err := client(app.Dir).Ping(context.Background())

	var refused *ServerError
	if !errors.As(err, &refused) || refused.Code != "unauthorized" {
		t.Fatalf("err = %v", err)
	}
	if strings.Contains(err.Error(), "secret-wrong-token") {
		t.Error("the error repeats the token")
	}
}

// Checks that a client that is not allowed sees no tools, and that a call
// gives the app's own "awaiting_approval" result.
func TestNotApprovedClient(t *testing.T) {
	app := testapp.Start(t, t.TempDir())
	app.SetApproved(false)
	c := client(app.Dir)

	tools, err := c.ListTools(context.Background())
	if err != nil || len(tools) != 0 {
		t.Errorf("tools = %v, %v", tools, err)
	}
	result, err := c.CallTool(context.Background(), "list_sprites", nil)
	if err != nil || !result.IsError || !strings.Contains(result.Content[0].Text, "awaiting_approval") {
		t.Errorf("result = %+v, %v", result, err)
	}
}
