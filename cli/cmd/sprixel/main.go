// sprixel is the command line companion of the Sprixel app.
package main

import (
	"os"

	"github.com/Javier-GarciaP/sprixel/cli/internal/command"
)

func main() {
	os.Exit(command.Run(os.Args[1:], os.Stdout))
}
