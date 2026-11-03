package command

import (
	"bufio"
	"fmt"
	"os"
	"path/filepath"
	"strings"

	"github.com/Javier-GarciaP/sprixel/cli/internal/clients"
	"github.com/Javier-GarciaP/sprixel/cli/internal/release"
	"github.com/Javier-GarciaP/sprixel/cli/internal/runtime"
)

const uninstallUsage = "sprixel uninstall [--yes] [--remove-data] [--json]"

func init() {
	commands = append(commands,
		command{name: "uninstall", summary: "Remove Sprixel from this computer (projects stay)", usage: uninstallUsage, run: runUninstall},
	)
}

// ask prints a yes/no question and reads the answer. The default is no: an
// empty answer, anything else, or the end of the input all mean no.
func (c *invocation) ask(in *bufio.Reader, question string) bool {
	fmt.Fprintf(c.env.Stdout, "%s [y/N] ", question)
	line, _ := in.ReadString('\n')
	answer := strings.ToLower(strings.TrimSpace(line))
	return answer == "y" || answer == "yes"
}

func runUninstall(c *invocation, args []string) int {
	yes, removeData := false, false
	for _, a := range args {
		switch a {
		case "--yes", "-y":
			yes = true
		case "--remove-data":
			removeData = true
		default:
			return c.p.Fail(ExitUsage, "usage", "unexpected argument: "+a, "usage: "+uninstallUsage)
		}
	}
	if c.p.json && !yes {
		return c.p.Fail(ExitUsage, "usage", "--json cannot ask questions", "add --yes to confirm the uninstall")
	}

	goos := c.env.goos()
	layout := c.installLayout()
	if !c.isInstalled(layout) {
		return c.p.Fail(ExitFailure, "not_installed",
			"this sprixel was not put here by the installer ("+c.exePath()+"), so it will not remove anything",
			"remove it the way you installed it")
	}
	if _, running := runtime.Running(c.dir); running {
		return c.p.Fail(ExitFailure, "app_running", "the Sprixel app is running", "close the app and run `sprixel uninstall` again")
	}

	registered := c.registeredClients()
	profiles := c.profilesWithPathLine(layout)

	// Show everything first, then ask. Projects are never in the list.
	if !c.p.json {
		out := c.env.Stdout
		fmt.Fprintln(out, "This will remove Sprixel from your computer:")
		fmt.Fprintln(out, "  - the app:     "+layout.App)
		fmt.Fprintln(out, "  - the command: "+layout.Command)
		fmt.Fprintln(out, "  - the license and notices in "+layout.Share)
		if goos == "windows" {
			fmt.Fprintln(out, "  - "+filepath.Dir(layout.Command)+" in your PATH")
		}
		for _, p := range profiles {
			fmt.Fprintln(out, "  - the PATH line added to "+p)
		}
		for _, cl := range registered {
			fmt.Fprintln(out, "  - the Sprixel registration in "+cl.Title())
		}
		if c.skill().State() != clients.SkillMissing {
			fmt.Fprintln(out, "  - the Sprixel skill in "+c.skill().Dir())
		}
		fmt.Fprintln(out, "Your projects are not touched.")
		fmt.Fprintln(out)
	}
	in := bufio.NewReader(c.env.Stdin)
	if !yes && !c.ask(in, "Remove Sprixel?") {
		c.p.Print("Nothing was removed.\n", map[string]any{"uninstalled": false})
		return ExitOK
	}
	if !removeData && !c.p.json {
		fmt.Fprintf(c.env.Stdout, "\nSprixel keeps its local data in %s (the port, the access token and the AI\nclients you approved). Your projects are not in it.\n", c.dir)
		removeData = c.ask(in, "Also remove this local data folder?")
	}

	var done []string
	var notes []string

	// 1. Registrations in AI tools.
	for _, cl := range registered {
		if _, err := cl.Remove(); err != nil {
			notes = append(notes, "could not remove the registration in "+cl.Title()+": "+err.Error())
			continue
		}
		done = append(done, "registration in "+cl.Title())
	}
	if removed, err := c.skill().Remove(); err != nil {
		notes = append(notes, "could not delete the skill folder "+c.skill().Dir()+": "+err.Error())
	} else if removed {
		done = append(done, "skill in "+c.skill().Dir())
	}
	// 2. The PATH entry.
	if goos == "windows" {
		if err := c.removeWindowsPath(filepath.Dir(layout.Command)); err != nil {
			notes = append(notes, "could not remove "+filepath.Dir(layout.Command)+" from your PATH: "+err.Error())
		} else {
			done = append(done, "PATH entry")
		}
	}
	for _, p := range profiles {
		if _, err := release.RemoveProfileLines(p); err != nil {
			notes = append(notes, "could not clean "+p+": "+err.Error())
			continue
		}
		done = append(done, "PATH line in "+p)
	}
	// 3. The local data folder, only if asked.
	if removeData {
		if filepath.Base(c.dir) != ".sprixel" {
			notes = append(notes, "the data folder was not removed: "+c.dir+" is not a .sprixel folder")
		} else if err := os.RemoveAll(c.dir); err != nil {
			notes = append(notes, "could not remove "+c.dir+": "+err.Error())
		} else {
			done = append(done, c.dir)
		}
	}
	// 4. The program files; the command itself goes last.
	files, err := release.Uninstall(goos, layout)
	done = append(done, files...)
	if err != nil {
		return c.p.Fail(ExitFailure, "remove", err.Error(), "close Sprixel and run the command again")
	}

	var text strings.Builder
	text.WriteString("Sprixel was removed.\n")
	for _, n := range notes {
		text.WriteString("warning: " + n + "\n")
	}
	if !removeData {
		text.WriteString("Your local data in " + c.dir + " and your projects were kept.\n")
	}
	c.p.Print(text.String(), map[string]any{"uninstalled": true, "removed": done, "dataRemoved": removeData, "warnings": notes})
	return ExitOK
}

// registeredClients lists the AI tools that have Sprixel registered. A tool
// that is not installed, or cannot be read, is skipped.
func (c *invocation) registeredClients() []clients.Client {
	var list []clients.Client
	for _, id := range clients.IDs {
		client, ok := c.client(id)
		if !ok {
			continue
		}
		if state, err := client.State(); err == nil && state == clients.StateYes {
			list = append(list, client)
		}
	}
	return list
}

// profilesWithPathLine lists the shell profiles that hold the installer's line.
func (c *invocation) profilesWithPathLine(l release.Layout) []string {
	var list []string
	for _, p := range l.Profiles {
		if data, err := os.ReadFile(p); err == nil && strings.Contains(string(data), release.ProfileMarker) {
			list = append(list, p)
		}
	}
	return list
}

// removeWindowsPath takes one folder out of the PATH of the user, using
// PowerShell (the Go standard library cannot edit the registry).
func (c *invocation) removeWindowsPath(dir string) error {
	run := c.env.Runner
	if run == nil {
		run = clients.SystemRunner
	}
	quoted := strings.ReplaceAll(dir, "'", "''")
	script := "$d = '" + quoted + "'; " +
		"$p = [Environment]::GetEnvironmentVariable('Path', 'User'); " +
		"$n = (($p -split ';') | Where-Object { $_ -and $_ -ne $d }) -join ';'; " +
		"[Environment]::SetEnvironmentVariable('Path', $n, 'User')"
	_, err := run("powershell", "-NoProfile", "-NonInteractive", "-Command", script)
	return err
}
