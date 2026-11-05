package release

import (
	"context"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"time"
)

// MenuFilesFor lists the files that the app writes for its entry in the
// applications menu, so `sprixel uninstall` can remove them. macOS has none:
// the app bundle is the entry. appData and localAppData are only used on
// Windows.
func MenuFilesFor(goos, home, appData, localAppData string) []string {
	switch goos {
	case "linux":
		return []string{
			filepath.Join(home, ".local", "share", "applications", "sprixel.desktop"),
			filepath.Join(home, ".local", "share", "icons", "hicolor", "256x256", "apps", "sprixel.png"),
		}
	case "windows":
		if appData == "" {
			appData = filepath.Join(home, "AppData", "Roaming")
		}
		return []string{
			filepath.Join(appData, "Microsoft", "Windows", "Start Menu", "Programs", "Sprixel.lnk"),
			filepath.Join(localAppData, "Sprixel", "menu-entry.txt"),
		}
	}
	return nil
}

// RegisterMenu asks the installed app to write its menu entry
// (`--register-menu`), with `home` as its home folder, and gives it `limit`
// to finish. It returns the last line the app printed. On macOS it does
// nothing.
func RegisterMenu(goos, home string, l Layout, limit time.Duration) (string, error) {
	if goos == "darwin" {
		return "", nil
	}
	ctx, cancel := context.WithTimeout(context.Background(), limit)
	defer cancel()
	cmd := exec.CommandContext(ctx, l.App, "--register-menu")
	cmd.Env = append(os.Environ(), "HOME="+home, "USERPROFILE="+home)
	cmd.WaitDelay = 2 * time.Second
	out, err := cmd.CombinedOutput()
	text := strings.TrimSpace(string(out))
	if err != nil {
		if text == "" {
			return "", fmt.Errorf("the app did not register its menu entry: %w", err)
		}
		return "", fmt.Errorf("the app did not register its menu entry (%s): %w", lastLine(text), err)
	}
	return lastLine(text), nil
}

func lastLine(text string) string {
	lines := strings.Split(text, "\n")
	return strings.TrimSpace(lines[len(lines)-1])
}
