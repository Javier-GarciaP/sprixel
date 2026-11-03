package command

// Exit codes of the sprixel command. They are part of the public interface:
// scripts depend on them (see docs/usage.md).
const (
	// ExitOK means the command did what was asked.
	ExitOK = 0
	// ExitFailure is any other failure (a failed doctor check, a config file
	// that could not be written).
	ExitFailure = 1
	// ExitUsage means the command line is wrong.
	ExitUsage = 2
	// ExitNotRunning means the Sprixel app is not running (or not reachable).
	ExitNotRunning = 3
	// ExitToolError means the app ran a tool and the tool answered with an error.
	ExitToolError = 4
)
