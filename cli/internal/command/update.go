package command

import (
	"fmt"
	"os"
	"path/filepath"
	"time"

	"github.com/Javier-GarciaP/sprixel/cli/internal/release"
	"github.com/Javier-GarciaP/sprixel/cli/internal/runtime"
	"github.com/Javier-GarciaP/sprixel/cli/internal/version"
)

const updateUsage = "sprixel update [--check] [--json]"

func init() {
	commands = append(commands,
		command{name: "update", summary: "Install the latest version of Sprixel", usage: updateUsage, run: runUpdate},
	)
}

// installLayout finds the folders of the installers for this system.
func (c *invocation) installLayout() release.Layout {
	l := release.LayoutFor(c.env.goos(), c.env.Home, c.env.getenv("LOCALAPPDATA"))
	l.MenuFiles = release.MenuFilesFor(c.env.goos(), c.env.Home, c.env.getenv("APPDATA"), c.env.getenv("LOCALAPPDATA"))
	return l
}

// exePath is the running command, with links resolved.
func (c *invocation) exePath() string {
	exe := c.env.Exe
	if exe == "" {
		exe, _ = os.Executable()
	}
	if real, err := filepath.EvalSymlinks(exe); err == nil {
		return real
	}
	return exe
}

// isInstalled reports whether the running command is the one the installer
// put in place. Update and uninstall refuse to touch any other copy (for
// example a build from source in bin/).
func (c *invocation) isInstalled(l release.Layout) bool {
	return sameFile(c.exePath(), l.Command)
}

func runUpdate(c *invocation, args []string) int {
	checkOnly := false
	for _, a := range args {
		if a != "--check" {
			return c.p.Fail(ExitUsage, "usage", "unexpected argument: "+a, "usage: "+updateUsage)
		}
		checkOnly = true
	}
	target, err := release.TargetFor(c.env.goos(), c.env.goarch())
	if err != nil {
		return c.p.Fail(ExitFailure, "unsupported", err.Error(), "")
	}
	layout := c.installLayout()
	if !c.isInstalled(layout) {
		return c.p.Fail(ExitFailure, "not_installed",
			"this sprixel was not put here by the installer ("+c.exePath()+"), so it will not replace it",
			"run the install script again, or update it the way you installed it")
	}

	current := version.Version
	base := release.BaseURL(c.env.getenv, "")
	latest, err := release.LatestVersion(base)
	if err != nil {
		return c.p.Fail(ExitFailure, "download", err.Error(), "check your connection and try again")
	}
	if !release.Newer(latest, current) {
		c.p.Print(fmt.Sprintf("Sprixel %s is up to date.\n", current),
			map[string]any{"updated": false, "available": false, "current": current, "latest": latest})
		return ExitOK
	}
	if checkOnly {
		c.p.Print(fmt.Sprintf("A new version is available: %s (you have %s). Run `sprixel update` to install it.\n", latest, current),
			map[string]any{"updated": false, "available": true, "current": current, "latest": latest})
		return ExitOK
	}

	files, err := release.Download(base, target)
	if err != nil {
		return c.p.Fail(ExitFailure, "download", err.Error(), "nothing was changed")
	}
	if err := release.Apply(c.env.goos(), target, layout, files); err != nil {
		return c.p.Fail(ExitFailure, "install", err.Error(), "run the install script to repair the installation")
	}

	text := fmt.Sprintf("Updated Sprixel from %s to %s.\n", current, latest)
	// The new app writes its own menu entry. A problem is only a warning.
	menuWarning := ""
	if _, err := release.RegisterMenu(c.env.goos(), c.env.Home, layout, 30*time.Second); err != nil {
		menuWarning = "Sprixel could not be added to the applications menu (" + err.Error() + "). Open the app once, or run: " + layout.App + " --register-menu"
		text += "warning: " + menuWarning + "\n"
	}
	_, running := runtime.Running(c.dir)
	if running {
		text += "The app is running: close it and open it again to use the new version.\n"
	}
	c.p.Print(text, map[string]any{"updated": true, "available": false, "current": current, "latest": latest, "from": current, "to": latest, "restartApp": running, "menuWarning": menuWarning})
	return ExitOK
}
