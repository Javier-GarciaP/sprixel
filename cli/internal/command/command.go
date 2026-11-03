// Package command implements the sprixel command line.
package command

import (
	"io"
	"os"
	goruntime "runtime"
	"sort"
	"strings"

	"github.com/Javier-GarciaP/sprixel/cli/internal/clients"
	"github.com/Javier-GarciaP/sprixel/cli/internal/launcher"
	"github.com/Javier-GarciaP/sprixel/cli/internal/runtime"
	"github.com/Javier-GarciaP/sprixel/cli/internal/version"
)

// Env is everything a command needs from the outside, so tests can replace it.
type Env struct {
	Stdin  io.Reader
	Stdout io.Writer
	Stderr io.Writer
	// Home is the home folder of the user. The data folder is inside it.
	Home string
	// StartApp starts the Sprixel app with the given arguments. When it is
	// nil the real launcher is used. Tests replace it.
	StartApp func(args ...string) (<-chan error, error)
	// Exe is the path of the running command; LookPath finds programs on the
	// PATH; Runner runs other programs (the `claude` command). All three have
	// real defaults and are replaced in tests.
	Exe      string
	LookPath func(string) (string, error)
	Runner   clients.Runner
	// GOOS and GOARCH name the system (update and uninstall pick files by
	// them); Getenv reads environment variables. Empty or nil means the real
	// ones; tests replace them.
	GOOS, GOARCH string
	Getenv       func(string) string
}

func (e Env) goos() string {
	if e.GOOS != "" {
		return e.GOOS
	}
	return goruntime.GOOS
}

func (e Env) goarch() string {
	if e.GOARCH != "" {
		return e.GOARCH
	}
	return goruntime.GOARCH
}

func (e Env) getenv(name string) string {
	if e.Getenv != nil {
		return e.Getenv(name)
	}
	return os.Getenv(name)
}

// invocation is what a command receives.
type invocation struct {
	env Env
	p   *printer
	// dir is the Sprixel data folder (~/.sprixel).
	dir string
}

type command struct {
	name    string
	summary string
	usage   string
	run     func(c *invocation, args []string) int
}

// commands is the list of commands, sorted by name when printed.
var commands = []command{
	{name: "version", summary: "Show the version of the command", usage: "sprixel version [--json]", run: runVersion},
}

// Run executes the command given in args and returns the exit code.
func Run(args []string, env Env) int {
	args, asJSON, help := splitFlags(args)
	p := &printer{out: env.Stdout, err: env.Stderr, json: asJSON}
	c := &invocation{env: env, p: p, dir: runtime.Dir(env.Home)}

	if len(args) == 0 {
		if help {
			io.WriteString(env.Stdout, helpText())
			return ExitOK
		}
		io.WriteString(env.Stderr, helpText())
		return ExitUsage
	}
	cmd := find(args[0])
	if cmd == nil {
		return p.Fail(ExitUsage, "usage", "unknown command: "+args[0], "run `sprixel --help` to see the commands")
	}
	if help {
		io.WriteString(env.Stdout, "usage: "+cmd.usage+"\n\n"+cmd.summary+"\n")
		return ExitOK
	}
	return cmd.run(c, args[1:])
}

// startApp starts the app: it checks that there is a screen, finds the app
// and runs it.
func (c *invocation) startApp(args ...string) (<-chan error, error) {
	if c.env.StartApp != nil {
		return c.env.StartApp(args...)
	}
	e := launcher.System(c.env.Home)
	if err := launcher.CheckDisplay(e); err != nil {
		return nil, err
	}
	path, err := launcher.Find(e)
	if err != nil {
		return nil, err
	}
	return launcher.Start(path, args...)
}

func find(name string) *command {
	for i := range commands {
		if commands[i].name == name {
			return &commands[i]
		}
	}
	return nil
}

func helpText() string {
	var b strings.Builder
	b.WriteString("sprixel " + version.Version + " - the command line of the Sprixel app\n\n")
	b.WriteString("usage: sprixel <command> [arguments]\n\ncommands:\n")
	sorted := append([]command(nil), commands...)
	sort.Slice(sorted, func(i, j int) bool { return sorted[i].name < sorted[j].name })
	for _, c := range sorted {
		b.WriteString("  " + pad(c.name, 10) + c.summary + "\n")
	}
	b.WriteString("\nflags for every command:\n")
	b.WriteString("  --json    print machine-readable JSON only\n")
	b.WriteString("  --help    show help\n")
	b.WriteString("\nexit codes: 0 ok, 1 failure, 2 usage error, 3 app not running, 4 tool error\n")
	return b.String()
}

func pad(s string, n int) string {
	for len(s) < n {
		s += " "
	}
	return s
}

// noArgs reports a usage error when a command that takes no arguments gets some.
func noArgs(c *invocation, usage string, args []string) (int, bool) {
	if len(args) == 0 {
		return 0, false
	}
	return c.p.Fail(ExitUsage, "usage", "unexpected argument: "+args[0], "usage: "+usage), true
}

func runVersion(c *invocation, args []string) int {
	if code, bad := noArgs(c, "sprixel version [--json]", args); bad {
		return code
	}
	c.p.Print(version.Version+"\n", map[string]string{"version": version.Version})
	return ExitOK
}
