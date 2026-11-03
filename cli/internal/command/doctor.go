package command

import (
	"context"
	"os/exec"
	"strings"

	"github.com/Javier-GarciaP/sprixel/cli/internal/clients"
	"github.com/Javier-GarciaP/sprixel/cli/internal/doctor"
	"github.com/Javier-GarciaP/sprixel/cli/internal/launcher"
	"github.com/Javier-GarciaP/sprixel/cli/internal/version"
)

func init() {
	commands = append(commands, command{
		name:    "doctor",
		summary: "Check the install, the app, the token and the AI tools",
		usage:   "sprixel doctor [--json]",
		run:     runDoctor,
	})
}

func runDoctor(c *invocation, args []string) int {
	if code, bad := noArgs(c, "sprixel doctor [--json]", args); bad {
		return code
	}
	var list []clients.Client
	for _, id := range clients.IDs {
		client, _ := c.client(id)
		list = append(list, client)
	}
	look := c.env.LookPath
	if look == nil {
		look = exec.LookPath
	}
	e := launcher.System(c.env.Home)
	checks := doctor.Run(context.Background(), doctor.Deps{
		Dir:        c.dir,
		Version:    version.Version,
		Command:    c.serverCommand(),
		FindApp:    func() (string, error) { return launcher.Find(e) },
		Clients:    list,
		HasProgram: func(name string) bool { _, err := look(name); return err == nil },
	})

	failed := doctor.Failed(checks)
	var text strings.Builder
	for _, check := range checks {
		label := map[doctor.Status]string{doctor.OK: "OK  ", doctor.Fail: "FAIL", doctor.Skip: "SKIP"}[check.Status]
		text.WriteString(label + "  " + pad(check.Name, 19) + check.Detail + "\n")
		if check.Fix != "" {
			text.WriteString("      fix: " + check.Fix + "\n")
		}
	}
	c.p.Print(text.String(), map[string]any{"ok": !failed, "checks": checks})
	if failed {
		return ExitFailure
	}
	return ExitOK
}
