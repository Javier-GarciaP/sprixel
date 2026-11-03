package clients

import (
	"bytes"
	"encoding/json"
	"errors"
	"fmt"
	"os"
	"path/filepath"
	"reflect"
)

// OpenCode is OpenCode. Sprixel edits its global JSON file: it reads the
// whole file, changes only mcp.sprixel and writes the file back atomically.
type OpenCode struct {
	// Home is the home folder of the user.
	Home string
}

func (OpenCode) ID() string    { return "opencode" }
func (OpenCode) Title() string { return "OpenCode" }

const schemaURL = "https://opencode.ai/config.json"

// path is the config file. opencode.json is the default; opencode.jsonc is
// used when only that one exists.
func (o OpenCode) path() string {
	dir := filepath.Join(o.Home, ".config", "opencode")
	json, jsonc := filepath.Join(dir, "opencode.json"), filepath.Join(dir, "opencode.jsonc")
	if _, err := os.Stat(json); err != nil {
		if _, err := os.Stat(jsonc); err == nil {
			return jsonc
		}
	}
	return json
}

func entry(command string) map[string]any {
	return map[string]any{"type": "local", "command": []string{command, "mcp"}, "enabled": true}
}

// load reads the file as generic JSON. A missing file is an empty config.
func (o OpenCode) load() (top map[string]json.RawMessage, existed bool, err error) {
	data, err := os.ReadFile(o.path())
	if errors.Is(err, os.ErrNotExist) {
		return map[string]json.RawMessage{}, false, nil
	}
	if err != nil {
		return nil, false, err
	}
	if len(bytes.TrimSpace(data)) == 0 {
		return map[string]json.RawMessage{}, true, nil
	}
	if err := json.Unmarshal(data, &top); err != nil || top == nil {
		return nil, true, fmt.Errorf("%w (%s). Add the server by hand", ErrHasComments, o.path())
	}
	return top, true, nil
}

// servers returns the mcp object of the config.
func servers(top map[string]json.RawMessage) (map[string]json.RawMessage, error) {
	servers := map[string]json.RawMessage{}
	if raw, ok := top["mcp"]; ok {
		if err := json.Unmarshal(raw, &servers); err != nil || servers == nil {
			return nil, errors.New("the \"mcp\" setting of the OpenCode config is not an object. Fix the file by hand")
		}
	}
	return servers, nil
}

// same says whether the saved entry is already what Sprixel would write.
func same(raw json.RawMessage, want map[string]any) bool {
	var got, expected any
	wantBytes, _ := json.Marshal(want)
	return json.Unmarshal(raw, &got) == nil && json.Unmarshal(wantBytes, &expected) == nil && reflect.DeepEqual(got, expected)
}

func (o OpenCode) State() (State, error) {
	top, _, err := o.load()
	if err != nil {
		return StateNo, err
	}
	list, err := servers(top)
	if err != nil {
		return StateNo, err
	}
	if _, ok := list[ServerName]; ok {
		return StateYes, nil
	}
	return StateNo, nil
}

func (o OpenCode) Add(command string) (Result, error) {
	top, existed, err := o.load()
	if err != nil {
		return Result{}, err
	}
	list, err := servers(top)
	if err != nil {
		return Result{}, err
	}
	want := entry(command)
	outcome := Added
	if old, ok := list[ServerName]; ok {
		if same(old, want) {
			return Result{AlreadyRegistered, o.path()}, nil
		}
		outcome = Updated
	}
	list[ServerName], _ = json.Marshal(want)
	top["mcp"], _ = json.Marshal(list)
	if _, ok := top["$schema"]; !ok && !existed {
		top["$schema"], _ = json.Marshal(schemaURL)
	}
	return Result{outcome, o.path()}, o.save(top)
}

func (o OpenCode) Remove() (Result, error) {
	top, _, err := o.load()
	if err != nil {
		return Result{}, err
	}
	list, err := servers(top)
	if err != nil {
		return Result{}, err
	}
	if _, ok := list[ServerName]; !ok {
		return Result{NotRegistered, o.path()}, nil
	}
	delete(list, ServerName)
	top["mcp"], _ = json.Marshal(list)
	return Result{Removed, o.path()}, o.save(top)
}

func (OpenCode) Snippet(command string) string {
	text, _ := json.MarshalIndent(map[string]any{"mcp": map[string]any{ServerName: entry(command)}}, "", "  ")
	return string(text)
}

// save writes the config next to the old file and renames it over it, so a
// failure never leaves a half written file. The mode of the old file is kept.
func (o OpenCode) save(top map[string]json.RawMessage) error {
	text, err := json.MarshalIndent(top, "", "  ")
	if err != nil {
		return err
	}
	path := o.path()
	if err := os.MkdirAll(filepath.Dir(path), 0o755); err != nil {
		return err
	}
	mode := os.FileMode(0o644)
	if info, err := os.Stat(path); err == nil {
		mode = info.Mode().Perm()
	}
	file, err := os.CreateTemp(filepath.Dir(path), ".opencode-*.tmp")
	if err != nil {
		return err
	}
	defer os.Remove(file.Name()) // only does something if the rename failed
	if _, err := file.Write(append(text, '\n')); err != nil {
		file.Close()
		return err
	}
	if err := file.Sync(); err != nil {
		file.Close()
		return err
	}
	if err := file.Close(); err != nil {
		return err
	}
	if err := os.Chmod(file.Name(), mode); err != nil {
		return err
	}
	return os.Rename(file.Name(), path)
}

// ByID returns a client by the id used in the command.
func ByID(id, home string, run Runner) (Client, bool) {
	switch id {
	case "claude":
		return Claude{Run: run}, true
	case "opencode":
		return OpenCode{Home: home}, true
	}
	return nil, false
}
