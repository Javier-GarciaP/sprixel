package clients

import (
	"bytes"
	_ "embed"
	"errors"
	"os"
	"path/filepath"
)

// skillText is the skill that teaches Claude Code to use Sprixel. It is part
// of the command, so installing it needs no download.
//
//go:embed skill/SKILL.md
var skillText []byte

// SkillState says whether the skill file is installed and current.
type SkillState int

const (
	SkillMissing SkillState = iota
	// SkillOld: a file exists but it is not the text of this command.
	SkillOld
	SkillCurrent
)

// Skill is the Sprixel skill in the home folder of a user. Sprixel writes only
// its own folder, ~/.claude/skills/sprixel/, and nothing else of Claude Code.
type Skill struct {
	Home string
}

// Dir is the folder of the skill.
func (s Skill) Dir() string { return filepath.Join(s.Home, ".claude", "skills", "sprixel") }

// Path is the skill file.
func (s Skill) Path() string { return filepath.Join(s.Dir(), "SKILL.md") }

// State reads the skill file.
func (s Skill) State() SkillState {
	data, err := os.ReadFile(s.Path())
	switch {
	case err != nil:
		return SkillMissing
	case bytes.Equal(data, skillText):
		return SkillCurrent
	}
	return SkillOld
}

// Install writes the skill file atomically (a temporary file, then a rename),
// so Claude Code never reads half a file. It reports whether it wrote.
func (s Skill) Install() (written bool, err error) {
	if s.Home == "" {
		return false, errors.New("the home folder is unknown")
	}
	if s.State() == SkillCurrent {
		return false, nil
	}
	if err := os.MkdirAll(s.Dir(), 0o755); err != nil {
		return false, err
	}
	temporary, err := os.CreateTemp(s.Dir(), ".SKILL.md.*")
	if err != nil {
		return false, err
	}
	defer os.Remove(temporary.Name())
	if _, err := temporary.Write(skillText); err != nil {
		temporary.Close()
		return false, err
	}
	if err := temporary.Close(); err != nil {
		return false, err
	}
	if err := os.Chmod(temporary.Name(), 0o644); err != nil {
		return false, err
	}
	return true, os.Rename(temporary.Name(), s.Path())
}

// Remove deletes the skill folder. It reports whether there was one.
func (s Skill) Remove() (removed bool, err error) {
	if s.Home == "" {
		return false, nil
	}
	if _, err := os.Stat(s.Dir()); err != nil {
		return false, nil
	}
	return true, os.RemoveAll(s.Dir())
}
