package clients

import (
	"context"
	"errors"
	"os/exec"
	"strings"
	"time"
)

// Runner runs a program and returns its output. It is replaced in tests.
type Runner func(name string, args ...string) (output string, err error)

// SystemRunner runs the real program with a time limit.
func SystemRunner(name string, args ...string) (string, error) {
	ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
	defer cancel()
	out, err := exec.CommandContext(ctx, name, args...).CombinedOutput()
	return string(out), err
}

// Claude is Claude Code. Sprixel does not edit its files: it calls
// `claude mcp`, so Claude Code writes its own configuration.
type Claude struct {
	Run Runner
}

func (Claude) ID() string    { return "claude" }
func (Claude) Title() string { return "Claude Code" }

func (c Claude) run(args ...string) (string, error) {
	run := c.Run
	if run == nil {
		run = SystemRunner
	}
	out, err := run("claude", args...)
	if errors.Is(err, exec.ErrNotFound) {
		return out, ErrNotInstalled
	}
	return out, err
}

func (c Claude) registered() (bool, error) {
	out, err := c.run("mcp", "get", ServerName)
	if errors.Is(err, ErrNotInstalled) {
		return false, err
	}
	// `claude mcp get` fails when the server does not exist. The docs do not
	// give its exit codes, so the answer must also name the server.
	return err == nil && strings.Contains(out, ServerName), nil
}

func (c Claude) State() (State, error) {
	yes, err := c.registered()
	switch {
	case errors.Is(err, ErrNotInstalled):
		return StateMissing, nil
	case err != nil:
		return StateNo, err
	case yes:
		return StateYes, nil
	}
	return StateNo, nil
}

func (c Claude) Add(command string) (Result, error) {
	yes, err := c.registered()
	if err != nil {
		return Result{}, err
	}
	where := "claude mcp add --scope user " + ServerName + " -- " + command + " mcp"
	if yes {
		return Result{AlreadyRegistered, where}, nil
	}
	if out, err := c.run("mcp", "add", "--scope", "user", ServerName, "--", command, "mcp"); err != nil {
		return Result{}, errors.New("claude mcp add failed: " + firstLine(out, err))
	}
	return Result{Added, where}, nil
}

func (c Claude) Remove() (Result, error) {
	yes, err := c.registered()
	if err != nil {
		return Result{}, err
	}
	where := "claude mcp remove " + ServerName + " --scope user"
	if !yes {
		return Result{NotRegistered, where}, nil
	}
	if out, err := c.run("mcp", "remove", ServerName, "--scope", "user"); err != nil {
		return Result{}, errors.New("claude mcp remove failed: " + firstLine(out, err))
	}
	return Result{Removed, where}, nil
}

func (Claude) Snippet(command string) string {
	return "claude mcp add --scope user " + ServerName + " -- " + command + " mcp"
}

func firstLine(out string, err error) string {
	if line := strings.TrimSpace(strings.SplitN(strings.TrimSpace(out), "\n", 2)[0]); line != "" {
		return line
	}
	return err.Error()
}
