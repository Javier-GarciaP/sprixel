package launcher

import (
	"errors"
	"os"
	"path/filepath"
	"time"
)

const lockName = "start.lock"

// ClaimStart says whether this process may start the app. Several commands
// (every AI tool that has the Sprixel MCP server) can find the app closed at
// the same moment, and without this each one opened its own window. The first
// one creates start.lock and wins; the others must wait for the app to be
// ready. A claim older than maxAge is from a process that died: it is taken
// over. release removes the claim.
func ClaimStart(dir string, maxAge time.Duration) (release func(), ok bool) {
	path := filepath.Join(dir, lockName)
	if err := os.MkdirAll(dir, 0o700); err != nil {
		return func() {}, true // cannot lock: do not block the start
	}
	for attempt := 0; attempt < 2; attempt++ {
		file, err := os.OpenFile(path, os.O_CREATE|os.O_EXCL|os.O_WRONLY, 0o600)
		if err == nil {
			file.Close()
			return func() { os.Remove(path) }, true
		}
		if !errors.Is(err, os.ErrExist) {
			return func() {}, true
		}
		info, statErr := os.Stat(path)
		if statErr == nil && time.Since(info.ModTime()) < maxAge {
			return nil, false
		}
		os.Remove(path) // old or vanished: try once more
	}
	return nil, false
}
