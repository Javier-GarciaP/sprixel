// Package runtime finds the running Sprixel app. The app and the command
// share a small folder, ~/.sprixel, with the port of the server
// (runtime.json) and the access token (token).
package runtime

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"net"
	"os"
	"path/filepath"
	"strconv"
	"strings"
	"time"
)

// ErrNotRunning means there is no runtime.json: the app is closed.
var ErrNotRunning = errors.New("the Sprixel app is not running")

// ErrNoToken means the token file is missing or empty.
var ErrNoToken = errors.New("the token file is missing or empty (start the Sprixel app once to create it)")

// Info is the content of runtime.json.
type Info struct {
	Port    int    `json:"port"`
	PID     int    `json:"pid"`
	Version string `json:"version,omitempty"`
}

// Addr is the address of the server. It is always the local machine.
func (i Info) Addr() string { return "127.0.0.1:" + strconv.Itoa(i.Port) }

// Home is the home folder of the user: HOME on Linux and macOS,
// USERPROFILE on Windows.
func Home() (string, error) { return os.UserHomeDir() }

// Dir is the Sprixel data folder inside a home folder.
func Dir(home string) string { return filepath.Join(home, ".sprixel") }

func runtimePath(dir string) string { return filepath.Join(dir, "runtime.json") }
func tokenPath(dir string) string   { return filepath.Join(dir, "token") }

// ReadInfo reads runtime.json. A missing file is ErrNotRunning; a file that
// cannot be understood is another error.
func ReadInfo(dir string) (Info, error) {
	data, err := os.ReadFile(runtimePath(dir))
	if errors.Is(err, os.ErrNotExist) {
		return Info{}, ErrNotRunning
	}
	if err != nil {
		return Info{}, fmt.Errorf("cannot read runtime.json: %w", err)
	}
	var info Info
	if err := json.Unmarshal(data, &info); err != nil {
		return Info{}, errors.New("runtime.json is damaged (close the app and start it again)")
	}
	if info.Port < 1 || info.Port > 65535 {
		return Info{}, errors.New("runtime.json has no valid port (close the app and start it again)")
	}
	return info, nil
}

// ReadToken reads the access token. The token is a secret: the errors of this
// function never contain it.
func ReadToken(dir string) (string, error) {
	data, err := os.ReadFile(tokenPath(dir))
	if errors.Is(err, os.ErrNotExist) {
		return "", ErrNoToken
	}
	if err != nil {
		return "", fmt.Errorf("cannot read the token file: %w", err)
	}
	token := strings.TrimSpace(string(data))
	if token == "" {
		return "", ErrNoToken
	}
	return token, nil
}

// TokenIsOpen says whether other users can read the token file. known is
// false when the system cannot tell (Windows keeps access lists, not modes).
func TokenIsOpen(dir string) (open bool, known bool, err error) {
	return tokenIsOpen(tokenPath(dir))
}

// Reachable says whether something listens on the port of runtime.json. A
// runtime.json left behind by an app that crashed is not reachable.
func Reachable(info Info) bool {
	conn, err := net.DialTimeout("tcp", info.Addr(), time.Second)
	if err != nil {
		return false
	}
	conn.Close()
	return true
}

// Running reads runtime.json and checks that the server answers.
func Running(dir string) (Info, bool) {
	info, err := ReadInfo(dir)
	if err != nil || !Reachable(info) {
		return Info{}, false
	}
	return info, true
}

// WaitReady waits until the app server is reachable. exited is the channel of
// the app process (it may be nil): a failure stops the wait at once, a clean
// exit does not (on macOS `open` ends well and the app goes on).
func WaitReady(ctx context.Context, dir string, exited <-chan error, timeout, every time.Duration) (Info, error) {
	deadline := time.NewTimer(timeout)
	defer deadline.Stop()
	poll := time.NewTicker(every)
	defer poll.Stop()
	for {
		select {
		case <-ctx.Done():
			return Info{}, errors.New("cancelled while the Sprixel app was starting")
		case err := <-exited:
			if err != nil {
				return Info{}, err
			}
			exited = nil
		case <-deadline.C:
			return Info{}, fmt.Errorf("the Sprixel app did not start in %d seconds. Start it by hand with `sprixel start`", int(timeout.Seconds()))
		case <-poll.C:
			if info, ok := Running(dir); ok {
				return info, nil
			}
		}
	}
}
