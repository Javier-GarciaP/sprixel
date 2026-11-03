//go:build !windows

package launcher

import (
	"os/exec"
	"syscall"
)

// detach puts the app in its own session, so closing the terminal or the AI
// tool that started it does not close the app.
func detach(command *exec.Cmd) {
	command.SysProcAttr = &syscall.SysProcAttr{Setsid: true}
}
