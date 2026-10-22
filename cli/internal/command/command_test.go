package command

import (
	"bytes"
	"testing"

	"github.com/Javier-GarciaP/sprixel/cli/internal/version"
)

// Checks that "version" prints the version with a newline and returns 0.
func TestVersionPrintsVersionAndReturnsZero(t *testing.T) {
	var out bytes.Buffer
	code := Run([]string{"version"}, &out)

	if code != 0 {
		t.Errorf("exit code = %d, want 0", code)
	}
	want := version.Version + "\n"
	if got := out.String(); got != want {
		t.Errorf("output = %q, want %q", got, want)
	}
}

// Checks that an unknown command returns 1.
func TestUnknownCommandReturnsOne(t *testing.T) {
	var out bytes.Buffer
	code := Run([]string{"nope"}, &out)

	if code != 1 {
		t.Errorf("exit code = %d, want 1", code)
	}
}
