//go:build !windows

package runtime

import "os"

func tokenIsOpen(path string) (bool, bool, error) {
	info, err := os.Stat(path)
	if err != nil {
		return false, true, err
	}
	return info.Mode().Perm()&0o077 != 0, true, nil
}
