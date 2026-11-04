// Package clients registers `sprixel mcp` in the AI tools Sprixel supports:
// Claude Code and OpenCode. Each client has its own file, so a change in one
// tool's format touches only that file (see docs/ai-and-mcp.md for the shapes).
package clients

import "errors"

// ServerName is the name of the MCP server in every client.
const ServerName = "sprixel"

// Outcome says what an Add or a Remove did.
type Outcome int

const (
	// Added: the registration was made now.
	Added Outcome = iota
	// Updated: it existed with another command and was corrected.
	Updated
	// AlreadyRegistered: nothing to do for Add.
	AlreadyRegistered
	// Removed: the registration was removed now.
	Removed
	// NotRegistered: nothing to do for Remove.
	NotRegistered
)

// Result is what a client reports. Where says what was changed: a file or the
// command that was run.
type Result struct {
	Outcome Outcome
	Where   string
}

// State is whether Sprixel is registered in a client.
type State int

const (
	// StateNo: the client works and Sprixel is not registered.
	StateNo State = iota
	// StateYes: Sprixel is registered.
	StateYes
	// StateMissing: the client itself is not installed.
	StateMissing
)

// Errors a client can return. The message always says what to do.
var (
	ErrNotInstalled = errors.New("the claude command was not found. Install Claude Code, or run this on the computer where it is installed")
	ErrHasComments  = errors.New("the OpenCode config file has comments (JSONC), which Sprixel cannot rewrite without losing them")
)

// Client is one AI tool.
type Client interface {
	// ID is the name used in the command: "claude" or "opencode".
	ID() string
	// Title is the name people know: "Claude Code" or "OpenCode".
	Title() string
	// Add registers `command mcp`. It is safe to run twice.
	Add(command string) (Result, error)
	// Remove removes the registration. It leaves everything else alone.
	Remove() (Result, error)
	// State says whether Sprixel is registered.
	State() (State, error)
	// Snippet is the text to add by hand, for when Add cannot do it.
	Snippet(command string) string
}

// IDs lists the supported clients.
var IDs = []string{"claude", "opencode"}
