//go:build windows

package runtime

// Windows uses access lists, which the app sets with icacls. The command does
// not read them, so the check is skipped.
func tokenIsOpen(path string) (bool, bool, error) { return false, false, nil }
