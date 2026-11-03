package clients

import (
	"os"
	"path/filepath"
	"strings"
	"testing"
)

// Checks that the embedded text is a real skill: it has the name Claude Code
// needs in its header and teaches the order of the tools and the rules.
func TestSkillTextHasTheLessons(t *testing.T) {
	text := string(skillText)
	if !strings.HasPrefix(text, "---\nname: sprixel\ndescription: ") {
		t.Errorf("the header is wrong: %q", text[:40])
	}
	for _, want := range []string{"get_project", "get_palette", "create_sprite", "open_project",
		"apply_grid", "set_pixels", "draw_shape", "render_preview", "permission_denied", "stale_revision",
		"image saved:", "OPEN that file"} {
		if !strings.Contains(text, want) {
			t.Errorf("the skill does not mention %s", want)
		}
	}
}

// Checks the scenarios "First add" and "Already registered": Install writes the
// file in a temporary home, writes a missing or old file again, and does not
// write when the file is already current. No temporary file stays behind.
func TestSkillInstall(t *testing.T) {
	skill := Skill{Home: t.TempDir()}
	if skill.State() != SkillMissing {
		t.Fatal("a new home must have no skill")
	}
	if written, err := skill.Install(); err != nil || !written {
		t.Fatalf("first install = %v, %v", written, err)
	}
	data, _ := os.ReadFile(skill.Path())
	if string(data) != string(skillText) || skill.State() != SkillCurrent {
		t.Error("the file is not the embedded text")
	}
	if written, _ := skill.Install(); written {
		t.Error("a current skill was written again")
	}

	os.WriteFile(skill.Path(), []byte("old"), 0o644)
	if skill.State() != SkillOld {
		t.Error("a different file must be old")
	}
	if written, err := skill.Install(); err != nil || !written || skill.State() != SkillCurrent {
		t.Errorf("update = %v, %v", written, err)
	}
	entries, _ := os.ReadDir(skill.Dir())
	if len(entries) != 1 {
		t.Errorf("the folder has %d files, want only SKILL.md", len(entries))
	}
}

// Checks that Remove deletes only the skill folder and reports when there was
// nothing to remove.
func TestSkillRemove(t *testing.T) {
	home := t.TempDir()
	other := filepath.Join(home, ".claude", "skills", "other", "SKILL.md")
	os.MkdirAll(filepath.Dir(other), 0o755)
	os.WriteFile(other, []byte("x"), 0o644)
	skill := Skill{Home: home}

	if removed, _ := skill.Remove(); removed {
		t.Error("removed a skill that was not there")
	}
	skill.Install()
	if removed, err := skill.Remove(); !removed || err != nil {
		t.Fatalf("remove = %v, %v", removed, err)
	}
	if _, err := os.Stat(skill.Dir()); !os.IsNotExist(err) {
		t.Error("the folder is still there")
	}
	if _, err := os.Stat(other); err != nil {
		t.Error("another skill was touched")
	}
}
