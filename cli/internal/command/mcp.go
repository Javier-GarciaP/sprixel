package command

import (
	"context"

	"github.com/Javier-GarciaP/sprixel/cli/internal/bridge"
)

func init() {
	commands = append(commands, command{
		name:    "mcp",
		summary: "MCP server over stdio for AI tools; `mcp add|remove|list` registers it",
		usage:   "sprixel mcp [add|remove <claude|opencode> | list] [--json]",
		run:     runMCP,
	})
}

// runMCP runs the bridge, or add, remove and list. Stdout is the MCP stream, so the printer is not
// used here: every message for people goes to stderr.
func runMCP(c *invocation, args []string) int {
	if len(args) > 0 {
		return runMCPSetup(c, args)
	}
	b := bridge.New(bridge.Config{
		Dir:   c.dir,
		In:    c.env.Stdin,
		Out:   c.env.Stdout,
		Log:   c.env.Stderr,
		Start: func() (<-chan error, error) { return c.startApp() },
	})
	if err := b.Run(context.Background()); err != nil {
		return c.p.Fail(ExitFailure, "bridge", err.Error(), "")
	}
	return ExitOK
}
