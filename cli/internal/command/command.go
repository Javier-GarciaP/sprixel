// Package command implements the sprixel command line.
package command

import (
	"io"

	"github.com/Javier-GarciaP/sprixel/cli/internal/version"
)

// Run executes the command given in args and writes its output to out.
// It returns the exit code.
func Run(args []string, out io.Writer) int {
	if len(args) == 0 {
		io.WriteString(out, "usage: sprixel <command>\ncommands: version\n")
		return 1
	}

	switch args[0] {
	case "version":
		io.WriteString(out, version.Version+"\n")
		return 0
	default:
		io.WriteString(out, "unknown command: "+args[0]+"\n")
		return 1
	}
}
