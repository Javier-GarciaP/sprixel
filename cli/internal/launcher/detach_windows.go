//go:build windows

package launcher

import (
	"os/exec"
	"syscall"
)

// detach starts the app apart from the console of the command.
func detach(command *exec.Cmd) {
	const detachedProcess = 0x00000008
	const newProcessGroup = 0x00000200
	command.SysProcAttr = &syscall.SysProcAttr{CreationFlags: detachedProcess | newProcessGroup, HideWindow: false}
}
