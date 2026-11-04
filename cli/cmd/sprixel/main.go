// sprixel is the command line companion of the Sprixel app.
package main

import (
	"fmt"
	"os"

	"github.com/Javier-GarciaP/sprixel/cli/internal/command"
	"github.com/Javier-GarciaP/sprixel/cli/internal/runtime"
)

func main() {
	home, err := runtime.Home()
	if err != nil {
		fmt.Fprintln(os.Stderr, "error: cannot find your home folder:", err)
		os.Exit(command.ExitFailure)
	}
	os.Exit(command.Run(os.Args[1:], command.Env{
		Stdin:  os.Stdin,
		Stdout: os.Stdout,
		Stderr: os.Stderr,
		Home:   home,
	}))
}
