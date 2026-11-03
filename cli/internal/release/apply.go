package release

import (
	"archive/tar"
	"archive/zip"
	"bytes"
	"compress/gzip"
	"errors"
	"fmt"
	"io"
	"os"
	"path/filepath"
	"strings"
)

// Apply puts the downloaded files in place. The files must already be
// checked by Download. It replaces the command, the app and the notices; it
// never touches anything else (projects, ~/.sprixel, the PATH).
func Apply(goos string, t Target, l Layout, files map[string][]byte) error {
	windows := goos == "windows"
	app := files[t.AppName()]

	// The app first: if it fails, the working command is still there.
	switch t.OS {
	case "macos":
		if err := replaceApp(l.App, app); err != nil {
			return fmt.Errorf("cannot install the app: %w", err)
		}
	case "windows":
		exe, err := readZipFile(app, "Sprixel.exe")
		if err != nil {
			return fmt.Errorf("cannot install the app: %w", err)
		}
		if err := replaceFile(l.App, exe, 0o755, windows); err != nil {
			return fmt.Errorf("cannot install the app: %w", err)
		}
	default:
		if err := replaceFile(l.App, app, 0o755, windows); err != nil {
			return fmt.Errorf("cannot install the app: %w", err)
		}
	}
	if err := replaceFile(l.Command, files[t.CLIName()], 0o755, windows); err != nil {
		return fmt.Errorf("cannot install the command: %w", err)
	}
	if err := os.MkdirAll(l.Share, 0o755); err != nil {
		return err
	}
	for _, name := range []string{LicenseFile, NoticesFile} {
		if err := replaceFile(filepath.Join(l.Share, name), files[name], 0o644, windows); err != nil {
			return fmt.Errorf("cannot write %s: %w", name, err)
		}
	}
	return nil
}

// replaceFile writes data to a temporary name and renames it over path, so a
// running program is replaced without a half-written file. Windows cannot
// overwrite a running program but can rename it, so the old file is moved
// aside first.
func replaceFile(path string, data []byte, mode os.FileMode, windows bool) error {
	if err := os.MkdirAll(filepath.Dir(path), 0o755); err != nil {
		return err
	}
	next := path + ".new"
	if err := os.WriteFile(next, data, mode); err != nil {
		return err
	}
	old := path + ".old"
	if windows {
		os.Remove(old)
		if err := os.Rename(path, old); err != nil && !errors.Is(err, os.ErrNotExist) {
			os.Remove(next)
			return err
		}
	}
	if err := os.Rename(next, path); err != nil {
		os.Remove(next)
		return err
	}
	os.Remove(old) // may fail while the old program still runs: harmless
	return nil
}

// replaceApp unpacks Sprixel.app next to the old one, then swaps them.
func replaceApp(dest string, archive []byte) error {
	parent := filepath.Dir(dest)
	if err := os.MkdirAll(parent, 0o755); err != nil {
		return err
	}
	stage, err := os.MkdirTemp(parent, ".sprixel-app-")
	if err != nil {
		return err
	}
	defer os.RemoveAll(stage)
	if err := untar(archive, stage); err != nil {
		return err
	}
	unpacked := filepath.Join(stage, "Sprixel.app")
	if info, err := os.Stat(unpacked); err != nil || !info.IsDir() {
		return errors.New("the app archive has no Sprixel.app")
	}
	old := dest + ".old"
	os.RemoveAll(old)
	if err := os.Rename(dest, old); err != nil && !errors.Is(err, os.ErrNotExist) {
		return err
	}
	if err := os.Rename(unpacked, dest); err != nil {
		os.Rename(old, dest) // put the old app back
		return err
	}
	os.RemoveAll(old)
	return nil
}

// untar unpacks a .tar.gz into dir. Names that leave dir are refused.
func untar(archive []byte, dir string) error {
	gz, err := gzip.NewReader(bytes.NewReader(archive))
	if err != nil {
		return fmt.Errorf("the app archive is damaged: %w", err)
	}
	tr := tar.NewReader(gz)
	for {
		h, err := tr.Next()
		if err == io.EOF {
			return nil
		}
		if err != nil {
			return fmt.Errorf("the app archive is damaged: %w", err)
		}
		target := filepath.Join(dir, filepath.FromSlash(h.Name))
		if target != dir && !strings.HasPrefix(target, dir+string(filepath.Separator)) {
			return fmt.Errorf("the app archive has an unsafe path: %s", h.Name)
		}
		switch h.Typeflag {
		case tar.TypeDir:
			err = os.MkdirAll(target, 0o755)
		case tar.TypeReg:
			err = writeFrom(target, tr, h.FileInfo().Mode().Perm())
		case tar.TypeSymlink:
			// A link must stay inside the app.
			resolved := filepath.Join(filepath.Dir(target), filepath.FromSlash(h.Linkname))
			if filepath.IsAbs(h.Linkname) || !strings.HasPrefix(resolved, dir+string(filepath.Separator)) {
				return fmt.Errorf("the app archive has an unsafe link: %s", h.Name)
			}
			if err = os.MkdirAll(filepath.Dir(target), 0o755); err == nil {
				err = os.Symlink(h.Linkname, target)
			}
		}
		if err != nil {
			return err
		}
	}
}

func writeFrom(path string, r io.Reader, mode os.FileMode) error {
	if err := os.MkdirAll(filepath.Dir(path), 0o755); err != nil {
		return err
	}
	f, err := os.OpenFile(path, os.O_CREATE|os.O_WRONLY|os.O_TRUNC, mode)
	if err != nil {
		return err
	}
	if _, err := io.Copy(f, r); err != nil {
		f.Close()
		return err
	}
	return f.Close()
}

func readZipFile(archive []byte, name string) ([]byte, error) {
	zr, err := zip.NewReader(bytes.NewReader(archive), int64(len(archive)))
	if err != nil {
		return nil, fmt.Errorf("the app archive is damaged: %w", err)
	}
	for _, f := range zr.File {
		if f.Name == name {
			rc, err := f.Open()
			if err != nil {
				return nil, err
			}
			defer rc.Close()
			return io.ReadAll(rc)
		}
	}
	return nil, fmt.Errorf("the app archive has no %s", name)
}
