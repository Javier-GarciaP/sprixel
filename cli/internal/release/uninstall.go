package release

import (
	"errors"
	"os"
	"path/filepath"
	"strings"
)

// ProfileMarker is the comment install.sh writes before the PATH line, so that
// uninstall can remove exactly what the installer added and nothing else.
const ProfileMarker = "# Added by the Sprixel installer"

// RemoveProfileLines deletes the lines the installer added to a shell profile
// (the marker and the export line after it). Other lines are not touched. It
// reports whether the file changed. A missing file is not an error.
func RemoveProfileLines(path string) (bool, error) {
	data, err := os.ReadFile(path)
	if errors.Is(err, os.ErrNotExist) {
		return false, nil
	}
	if err != nil {
		return false, err
	}
	lines := strings.Split(string(data), "\n")
	var keep []string
	changed := false
	for i := 0; i < len(lines); i++ {
		if lines[i] != ProfileMarker {
			keep = append(keep, lines[i])
			continue
		}
		changed = true
		// The installer wrote an empty line, the marker and the export line.
		if n := len(keep); n > 0 && keep[n-1] == "" {
			keep = keep[:n-1]
		}
		if i+1 < len(lines) && strings.HasPrefix(lines[i+1], "export PATH=") {
			i++
		}
	}
	if !changed {
		return false, nil
	}
	info, err := os.Stat(path)
	if err != nil {
		return false, err
	}
	return true, os.WriteFile(path, []byte(strings.Join(keep, "\n")), info.Mode().Perm())
}

// Uninstall removes the program files of an installation: the menu entry of
// the app, the app, the license and notices, and the command last. It never removes projects or the
// data folder (~/.sprixel). It returns what was removed. A path that is
// already gone is skipped.
func Uninstall(goos string, l Layout) ([]string, error) {
	var removed []string
	remove := func(path string, all bool) error {
		if _, err := os.Lstat(path); errors.Is(err, os.ErrNotExist) {
			return nil
		}
		var err error
		if all {
			err = os.RemoveAll(path)
		} else {
			err = os.Remove(path)
		}
		if err != nil && goos == "windows" {
			// A running program cannot be deleted on Windows, but it can be
			// renamed. The leftover .old file is deleted when possible.
			if os.Rename(path, path+".old") == nil {
				os.Remove(path + ".old")
				err = nil
			}
		}
		if err != nil {
			return err
		}
		removed = append(removed, path)
		return nil
	}

	// The menu entry first: it points to the app that is about to go.
	for _, path := range l.MenuFiles {
		if err := remove(path, false); err != nil {
			return removed, err
		}
	}
	if err := remove(l.App, true); err != nil {
		return removed, err
	}
	for _, name := range []string{LicenseFile, NoticesFile} {
		if err := remove(filepath.Join(l.Share, name), false); err != nil {
			return removed, err
		}
	}
	if err := remove(l.Command, false); err != nil {
		return removed, err
	}
	// On Windows the command sits in a bin folder of its own.
	if goos == "windows" && os.Remove(filepath.Dir(l.Command)) == nil {
		removed = append(removed, filepath.Dir(l.Command))
	}
	// The share folder goes only if it is empty now (never RemoveAll).
	if os.Remove(l.Share) == nil {
		removed = append(removed, l.Share)
	}
	return removed, nil
}
