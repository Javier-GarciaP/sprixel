package command

import (
	"errors"
	"os"
	"os/exec"
	"path/filepath"
	"strings"

	"github.com/Javier-GarciaP/sprixel/cli/internal/clients"
)

const mcpUsage = "usage: sprixel mcp add <claude|opencode> | sprixel mcp remove <claude|opencode> | sprixel mcp list"

func runMCPSetup(c *invocation, args []string) int {
	switch args[0] {
	case "list":
		if len(args) > 1 {
			return c.p.Fail(ExitUsage, "usage", "unexpected argument: "+args[1], mcpUsage)
		}
		return mcpList(c)
	case "add", "remove":
		if len(args) != 2 {
			return c.p.Fail(ExitUsage, "usage", "say which client: claude or opencode", mcpUsage)
		}
		client, ok := c.client(args[1])
		if !ok {
			return c.p.Fail(ExitUsage, "usage", "unknown client: "+args[1], "the clients are claude and opencode")
		}
		if args[0] == "add" {
			return mcpAdd(c, client)
		}
		return mcpRemove(c, client)
	}
	return c.p.Fail(ExitUsage, "usage", "unknown mcp command: "+args[0], mcpUsage)
}

// skill is the Claude Code skill of Sprixel, in the home of this run.
func (c *invocation) skill() clients.Skill { return clients.Skill{Home: c.env.Home} }

func (c *invocation) client(id string) (clients.Client, bool) {
	return clients.ByID(id, c.env.Home, c.env.Runner)
}

// serverCommand is the program the AI tool will start: "sprixel" when the
// PATH finds this same program, otherwise the full path of this program, so
// the registration works even when sprixel is not on the PATH.
func (c *invocation) serverCommand() string {
	exe := c.env.Exe
	if exe == "" {
		exe, _ = os.Executable()
	}
	look := c.env.LookPath
	if look == nil {
		look = exec.LookPath
	}
	if found, err := look("sprixel"); err == nil && exe != "" {
		a, errA := filepath.EvalSymlinks(found)
		b, errB := filepath.EvalSymlinks(exe)
		if errA == nil && errB == nil && sameFile(a, b) {
			return "sprixel"
		}
	}
	if exe == "" {
		return "sprixel"
	}
	return exe
}

func sameFile(a, b string) bool {
	ia, errA := os.Stat(a)
	ib, errB := os.Stat(b)
	return errA == nil && errB == nil && os.SameFile(ia, ib)
}

func mcpAdd(c *invocation, client clients.Client) int {
	command := c.serverCommand()
	result, err := client.Add(command)
	if err != nil {
		return clientFailure(c, client, command, err)
	}
	undo := "sprixel mcp remove " + client.ID()
	text := ""
	state := "added"
	switch result.Outcome {
	case clients.AlreadyRegistered:
		state = "already_registered"
		text = "Sprixel is already registered in " + client.Title() + ". Nothing changed.\n"
	case clients.Updated:
		state = "updated"
		text = "Sprixel was registered in " + client.Title() + " with another command. It now starts `" + command + " mcp`.\n  changed: " + result.Where + "\n  undo:    " + undo + "\n"
	default:
		text = "Registered Sprixel in " + client.Title() + ". It starts `" + command + " mcp`.\n  changed: " + result.Where + "\n  undo:    " + undo + "\n"
	}
	data := map[string]string{"client": client.ID(), "result": state, "changed": result.Where, "undo": undo}
	if client.ID() == "claude" {
		// The skill is written also when Sprixel was already registered, so a
		// missing or old file is fixed by running the command again.
		skill := c.skill()
		written, err := skill.Install()
		if err != nil {
			return c.p.Fail(ExitFailure, "skill", "Sprixel is registered, but the skill file could not be written: "+err.Error(),
				"check that "+skill.Dir()+" can be written, then run the command again")
		}
		data["skill"] = skill.Path()
		if written {
			text += "  skill:   " + skill.Path() + "\n"
		} else {
			text += "  skill:   " + skill.Path() + " is up to date\n"
		}
	}
	c.p.Print(text, data)
	return ExitOK
}

func mcpRemove(c *invocation, client clients.Client) int {
	result, err := client.Remove()
	if err != nil {
		return clientFailure(c, client, c.serverCommand(), err)
	}
	if result.Outcome == clients.NotRegistered {
		c.p.Print("Sprixel is not registered in "+client.Title()+". Nothing changed.\n",
			map[string]string{"client": client.ID(), "result": "not_registered"})
		return ExitOK
	}
	c.p.Print("Removed Sprixel from "+client.Title()+".\n  changed: "+result.Where+"\n",
		map[string]string{"client": client.ID(), "result": "removed", "changed": result.Where})
	return ExitOK
}

// clientFailure explains why a client could not be changed. When the file
// cannot be edited, the text to add by hand is part of the hint.
func clientFailure(c *invocation, client clients.Client, command string, err error) int {
	hint := ""
	switch {
	case errors.Is(err, clients.ErrNotInstalled):
		hint = "install " + client.Title() + " and run the command again"
	case errors.Is(err, clients.ErrHasComments):
		hint = "add this by hand:\n" + client.Snippet(command)
		if c.p.json {
			hint = strings.ReplaceAll(hint, "\n", " ")
		}
	}
	return c.p.Fail(ExitFailure, "client", err.Error(), hint)
}

func mcpList(c *invocation) int {
	var text strings.Builder
	var rows []map[string]string
	for _, id := range clients.IDs {
		client, _ := c.client(id)
		state, err := client.State()
		label := map[clients.State]string{clients.StateYes: "registered", clients.StateNo: "not registered", clients.StateMissing: "not installed"}[state]
		if err != nil {
			label = "unknown (" + err.Error() + ")"
		}
		text.WriteString(pad(id, 10) + pad(client.Title(), 13) + label + "\n")
		rows = append(rows, map[string]string{"id": id, "name": client.Title(), "state": strings.ReplaceAll(label, " ", "_")})
	}
	c.p.Print(text.String(), map[string]any{"clients": rows})
	return ExitOK
}
