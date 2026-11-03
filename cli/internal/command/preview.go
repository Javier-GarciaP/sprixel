package command

import (
	"crypto/sha256"
	"encoding/base64"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"sort"
	"strings"

	"github.com/Javier-GarciaP/sprixel/cli/internal/runtime"
)

// keepPreviews is how many preview files stay in the folder.
const keepPreviews = 20

// savePreview writes a PNG (base64 text, as the app sends it) in
// ~/.sprixel/previews and returns the path. An AI that only uses the shell
// cannot read base64, but it can open a file. info is the text part of the
// result of render_preview; it names the file: <sprite>-<frame>-<revision>.png
// (render_frames and render_onion: <sprite>-<kind>-<revision>-<hash>.png).
func (c *invocation) savePreview(data string, info string) (string, error) {
	png, err := base64.StdEncoding.DecodeString(data)
	if err != nil || len(png) == 0 {
		return "", fmt.Errorf("the image is not valid base64")
	}
	dir := filepath.Join(runtime.Dir(c.env.Home), "previews")
	if err := os.MkdirAll(dir, 0o700); err != nil {
		return "", err
	}
	path := filepath.Join(dir, previewName(info, png))
	// A temporary file first, then a rename: nobody reads half an image.
	temporary, err := os.CreateTemp(dir, ".preview-*.tmp")
	if err != nil {
		return "", err
	}
	defer os.Remove(temporary.Name())
	if _, err := temporary.Write(png); err != nil {
		temporary.Close()
		return "", err
	}
	if err := temporary.Close(); err != nil {
		return "", err
	}
	if err := os.Chmod(temporary.Name(), 0o600); err != nil {
		return "", err
	}
	if err := os.Rename(temporary.Name(), path); err != nil {
		return "", err
	}
	prunePreviews(dir, path)
	return path, nil
}

// previewName is <sprite>-<frame>-<revision>.png. The sprite name comes from
// the app, so only letters, digits, dot, dash and underscore stay: the name
// can never leave the folder. Without the info it is a short hash of the image.
func previewName(info string, png []byte) string {
	var parsed struct {
		Name     string `json:"name"`
		Kind     string `json:"kind"`
		Frame    int    `json:"frame"`
		Revision uint64 `json:"revision"`
	}
	if json.Unmarshal([]byte(info), &parsed) == nil && parsed.Name != "" && (parsed.Frame > 0 || parsed.Kind == "frames" || parsed.Kind == "onion") {
		safe := strings.Map(func(r rune) rune {
			switch {
			case r >= 'a' && r <= 'z', r >= 'A' && r <= 'Z', r >= '0' && r <= '9', r == '-', r == '_':
				return r
			}
			return '_'
		}, parsed.Name)
		sum := sha256.Sum256(png)
		if parsed.Kind == "frames" || parsed.Kind == "onion" {
			// Several frames: the hash tells two different requests apart.
			return fmt.Sprintf("%s-%s-%d-%s.png", strings.Trim(safe, "_-"), parsed.Kind, parsed.Revision, hex.EncodeToString(sum[:4]))
		}
		return fmt.Sprintf("%s-%d-%d.png", strings.Trim(safe, "_-"), parsed.Frame, parsed.Revision)
	}
	sum := sha256.Sum256(png)
	return "preview-" + hex.EncodeToString(sum[:4]) + ".png"
}

// prunePreviews deletes the oldest .png files so that keepPreviews stay. The
// file just written (keep) is never deleted. Errors are ignored: a file that
// stays is not worth failing the command.
func prunePreviews(dir, keep string) {
	entries, err := os.ReadDir(dir)
	if err != nil {
		return
	}
	type file struct {
		path string
		time int64
	}
	var files []file
	for _, e := range entries {
		if e.IsDir() || !strings.HasSuffix(e.Name(), ".png") {
			continue
		}
		if info, err := e.Info(); err == nil {
			files = append(files, file{filepath.Join(dir, e.Name()), info.ModTime().UnixNano()})
		}
	}
	// Newest first; the same time is broken by the name, so the order is stable.
	sort.Slice(files, func(i, j int) bool {
		if files[i].time != files[j].time {
			return files[i].time > files[j].time
		}
		return files[i].path > files[j].path
	})
	// The new file takes one place; the others share the rest.
	others := 0
	for _, f := range files {
		if f.path == keep {
			continue
		}
		if others++; others > keepPreviews-1 {
			os.Remove(f.path)
		}
	}
}
