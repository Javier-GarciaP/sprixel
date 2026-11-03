// Package doctor checks that Sprixel is installed and connected: the app, the
// version, the token, the server and the AI tools. Every check says what is
// wrong and how to fix it.
package doctor

import (
	"context"
	"errors"
	"fmt"
	"strings"

	"github.com/Javier-GarciaP/sprixel/cli/internal/appclient"
	"github.com/Javier-GarciaP/sprixel/cli/internal/clients"
	"github.com/Javier-GarciaP/sprixel/cli/internal/runtime"
)

// Status is the result of one check.
type Status string

const (
	OK   Status = "ok"
	Fail Status = "fail"
	// Skip is a check that does not apply (for example, the client is not installed).
	Skip Status = "skip"
)

// Check is one line of the report.
type Check struct {
	Name   string `json:"name"`
	Status Status `json:"status"`
	Detail string `json:"detail"`
	// Fix says how to solve a failure. Empty when the check is not a failure.
	Fix string `json:"fix,omitempty"`
}

// Deps is everything the checks read. Tests replace the parts.
type Deps struct {
	// Dir is the Sprixel data folder.
	Dir string
	// Version is the version of this command.
	Version string
	// Command is the command that AI tools will start ("sprixel" or a path).
	Command string
	// FindApp returns the path of the installed app.
	FindApp func() (string, error)
	// Clients are the AI tools to check.
	Clients []clients.Client
	// HasProgram says whether a program is installed (opencode).
	HasProgram func(name string) bool
}

// Run runs every check, in a fixed order.
func Run(ctx context.Context, d Deps) []Check {
	var checks []Check
	add := func(c Check) { checks = append(checks, c) }

	info, running := runtime.Running(d.Dir)
	add(installation(d, running))
	add(appRunning(d, running))
	add(versions(d, info, running))
	token, tokenCheck := tokenFile(d)
	add(tokenCheck)
	add(tokenPermissions(d, token != ""))
	add(serverAnswer(ctx, d, running, token != ""))
	for _, client := range d.Clients {
		add(registration(d, client))
	}
	return checks
}

// Failed says whether any check failed.
func Failed(checks []Check) bool {
	for _, c := range checks {
		if c.Status == Fail {
			return true
		}
	}
	return false
}

func installation(d Deps, running bool) Check {
	path, err := d.FindApp()
	switch {
	case err == nil:
		return Check{Name: "installation", Status: OK, Detail: "the app is at " + path}
	case running:
		return Check{Name: "installation", Status: OK, Detail: "the app is running (its install place is not one Sprixel knows)"}
	}
	return Check{Name: "installation", Status: Fail, Detail: "the Sprixel app was not found",
		Fix: "install Sprixel (see the README), or set SPRIXEL_APP to the path of the app"}
}

func appRunning(d Deps, running bool) Check {
	if running {
		return Check{Name: "app", Status: OK, Detail: "the app is running"}
	}
	return Check{Name: "app", Status: Fail, Detail: "the app is not running", Fix: "run `sprixel start`"}
}

func versions(d Deps, info runtime.Info, running bool) Check {
	switch {
	case !running:
		return Check{Name: "version", Status: Skip, Detail: "the app is not running, so its version is unknown"}
	case info.Version == "":
		return Check{Name: "version", Status: Skip, Detail: "the app did not report its version"}
	case info.Version != d.Version:
		return Check{Name: "version", Status: Fail,
			Detail: fmt.Sprintf("the app is %s and the command is %s", info.Version, d.Version),
			Fix:    "install the same version of both (see the README)"}
	}
	return Check{Name: "version", Status: OK, Detail: "app and command are " + d.Version}
}

// tokenFile checks the token file and returns the token. The token itself is
// never part of a message.
func tokenFile(d Deps) (string, Check) {
	token, err := runtime.ReadToken(d.Dir)
	if errors.Is(err, runtime.ErrNoToken) {
		return "", Check{Name: "token", Status: Fail, Detail: "the token file is missing or empty",
			Fix: "start the Sprixel app once: it creates the token"}
	}
	if err != nil {
		return "", Check{Name: "token", Status: Fail, Detail: err.Error(), Fix: "check the files in " + d.Dir}
	}
	return token, Check{Name: "token", Status: OK, Detail: "the token file exists"}
}

func tokenPermissions(d Deps, hasToken bool) Check {
	if !hasToken {
		return Check{Name: "token permissions", Status: Skip, Detail: "there is no token file"}
	}
	open, known, err := runtime.TokenIsOpen(d.Dir)
	switch {
	case err != nil:
		return Check{Name: "token permissions", Status: Fail, Detail: err.Error(), Fix: "check the files in " + d.Dir}
	case !known:
		return Check{Name: "token permissions", Status: Skip, Detail: "this system keeps access lists that the command does not read"}
	case open:
		return Check{Name: "token permissions", Status: Fail, Detail: "other users can read the token file",
			Fix: "run `chmod 600 " + d.Dir + "/token`, or delete the file and restart the app to make a new token"}
	}
	return Check{Name: "token permissions", Status: OK, Detail: "only your user can read the token"}
}

func serverAnswer(ctx context.Context, d Deps, running, hasToken bool) Check {
	if !running {
		return Check{Name: "server", Status: Skip, Detail: "the app is not running"}
	}
	if !hasToken {
		return Check{Name: "server", Status: Skip, Detail: "there is no token"}
	}
	client := &appclient.Client{Dir: d.Dir, Version: d.Version}
	err := client.Ping(ctx)
	var refused *appclient.ServerError
	switch {
	case err == nil:
		return Check{Name: "server", Status: OK, Detail: "the server answers with the token"}
	case errors.As(err, &refused) && refused.Code == "unauthorized":
		return Check{Name: "server", Status: Fail, Detail: "the server refused the token",
			Fix: "close the app and start it again with `sprixel start`"}
	}
	return Check{Name: "server", Status: Fail, Detail: err.Error(), Fix: "close the app and start it again with `sprixel start`"}
}

func registration(d Deps, client clients.Client) Check {
	name := client.ID()
	state, err := client.State()
	switch {
	case err != nil:
		return Check{Name: name, Status: Fail, Detail: "cannot read the " + client.Title() + " setup: " + firstLine(err.Error()),
			Fix: "fix the setup of " + client.Title() + ", or add Sprixel by hand (see docs/ai-and-mcp.md)"}
	case state == clients.StateYes:
		return Check{Name: name, Status: OK, Detail: "Sprixel is registered in " + client.Title()}
	case state == clients.StateMissing:
		return Check{Name: name, Status: Skip, Detail: client.Title() + " is not installed"}
	case client.ID() == "opencode" && d.HasProgram != nil && !d.HasProgram("opencode"):
		return Check{Name: name, Status: Skip, Detail: client.Title() + " is not installed"}
	}
	return Check{Name: name, Status: Fail, Detail: "Sprixel is not registered in " + client.Title(),
		Fix: "run `sprixel mcp add " + client.ID() + "`"}
}

func firstLine(text string) string {
	return strings.TrimSpace(strings.SplitN(text, "\n", 2)[0])
}
